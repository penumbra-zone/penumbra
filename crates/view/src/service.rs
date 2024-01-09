use std::{
    collections::{BTreeMap, BTreeSet},
    pin::Pin,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Context};
use ark_std::UniformRand;
use async_stream::try_stream;
use camino::Utf8Path;
use decaf377::Fq;
use futures::stream::{StreamExt, TryStreamExt};
use rand::Rng;
use rand_core::OsRng;
use tokio::sync::{watch, RwLock};
use tokio_stream::wrappers::WatchStream;
use tonic::{async_trait, transport::Channel, Request, Response, Status};
use tracing::instrument;
use url::Url;

use penumbra_asset::{asset, Value};
use penumbra_dex::{
    lp::{
        position::{self, Position},
        Reserves,
    },
    swap_claim::SwapClaimPlan,
    TradingPair,
};
use penumbra_fee::Fee;
use penumbra_keys::{
    keys::{AddressIndex, FullViewingKey},
    Address,
};
use penumbra_num::Amount;
use penumbra_proto::view::v1alpha1::{WalletIdRequest, WalletIdResponse};
use penumbra_proto::{
    util::tendermint_proxy::v1alpha1::{
        tendermint_proxy_service_client::TendermintProxyServiceClient, BroadcastTxSyncRequest,
        GetStatusRequest,
    },
    view::v1alpha1::{
        self as pb,
        view_protocol_service_client::ViewProtocolServiceClient,
        view_protocol_service_server::{ViewProtocolService, ViewProtocolServiceServer},
        AppParametersResponse, FmdParametersResponse, GasPricesResponse, NoteByCommitmentResponse,
        StatusResponse, SwapByCommitmentResponse, TransactionPlannerResponse, WitnessResponse,
    },
    DomainType,
};
use penumbra_stake::rate::RateData;
use penumbra_tct::{Proof, StateCommitment};
use penumbra_transaction::{
    plan::TransactionPlan, txhash::TransactionId, AuthorizationData, Transaction,
    TransactionPerspective, WitnessData,
};

use crate::{Planner, Storage, Worker};

/// A service that synchronizes private chain state and responds to queries
/// about it.
///
/// The [`ViewService`] implements the Tonic-derived [`ViewProtocol`] trait,
/// so it can be used as a gRPC server, or called directly.  It spawns a task
/// internally that performs synchronization and scanning.  The
/// [`ViewService`] can be cloned; each clone will read from the same shared
/// state, but there will only be a single scanning task.
#[derive(Clone)]
pub struct ViewService {
    storage: Storage,
    // A shared error slot for errors bubbled up by the worker. This is a regular Mutex
    // rather than a Tokio Mutex because it should be uncontended.
    error_slot: Arc<Mutex<Option<anyhow::Error>>>,
    // A copy of the SCT used by the worker task.
    state_commitment_tree: Arc<RwLock<penumbra_tct::Tree>>,
    // The Url for the pd gRPC endpoint on remote node.
    node: Url,
    /// Used to watch for changes to the sync height.
    sync_height_rx: watch::Receiver<u64>,
}

impl ViewService {
    /// Convenience method that calls [`Storage::load_or_initialize`] and then [`Self::new`].
    pub async fn load_or_initialize(
        storage_path: Option<impl AsRef<Utf8Path>>,
        fvk: &FullViewingKey,
        node: Url,
    ) -> anyhow::Result<Self> {
        let storage = Storage::load_or_initialize(storage_path, fvk, node.clone()).await?;

        Self::new(storage, node).await
    }

    /// Constructs a new [`ViewService`], spawning a sync task internally.
    ///
    /// The sync task uses the provided `client` to sync with the chain.
    ///
    /// To create multiple [`ViewService`]s, clone the [`ViewService`] returned
    /// by this method, rather than calling it multiple times.  That way, each clone
    /// will be backed by the same scanning task, rather than each spawning its own.
    pub async fn new(storage: Storage, node: Url) -> anyhow::Result<Self> {
        let (worker, sct, error_slot, sync_height_rx) =
            Worker::new(storage.clone(), node.clone()).await?;

        tokio::spawn(worker.run());

        Ok(Self {
            storage,
            error_slot,
            sync_height_rx,
            state_commitment_tree: sct,
            node,
        })
    }

    async fn check_worker(&self) -> Result<(), tonic::Status> {
        // If the shared error slot is set, then an error has occurred in the worker
        // that we should bubble up.
        if self
            .error_slot
            .lock()
            .map_err(|e| {
                tonic::Status::unavailable(format!("unable to lock worker error slot {:#}", e))
            })?
            .is_some()
        {
            return Err(tonic::Status::new(
                tonic::Code::Internal,
                format!(
                    "Worker failed: {}",
                    self.error_slot
                        .lock()
                        .map_err(|e| {
                            tonic::Status::unavailable(format!(
                                "unable to lock worker error slot {:#}",
                                e
                            ))
                        })?
                        .as_ref()
                        .ok_or_else(|| {
                            tonic::Status::unavailable("unable to get ref to worker error slot")
                        })?
                ),
            ));
        }

        // TODO: check whether the worker is still alive, else fail, when we have a way to do that
        // (if the worker is to crash without setting the error_slot, the service should die as well)

        Ok(())
    }

    #[instrument(skip(self, transaction), fields(id = %transaction.id()))]
    async fn broadcast_transaction(
        &self,
        transaction: Transaction,
        await_detection: bool,
    ) -> anyhow::Result<TransactionId> {
        use penumbra_app::ActionHandler;

        // 1. Pre-check the transaction for (stateless) validity.
        transaction
            .check_stateless(())
            .await
            .context("transaction pre-submission checks failed")?;

        // 2. Broadcast the transaction to the network.
        // Note that "synchronous" here means "wait for the tx to be accepted by
        // the fullnode", not "wait for the tx to be included on chain.
        let mut fullnode_client = self.tendermint_proxy_client().await?;
        let node_rsp = fullnode_client
            .broadcast_tx_sync(BroadcastTxSyncRequest {
                params: transaction.encode_to_vec(),
                req_id: OsRng.gen(),
            })
            .await?
            .into_inner();
        tracing::info!(?node_rsp);
        if node_rsp.code != 0 {
            anyhow::bail!(
                "Error submitting transaction: code {}, log: {}",
                node_rsp.code,
                node_rsp.log,
            );
        }

        // 3. Optionally wait for the transaction to be detected by the view service.
        let nullifier = if await_detection {
            // This needs to be only *spend* nullifiers because the nullifier detection
            // is broken for swaps, https://github.com/penumbra-zone/penumbra/issues/1749
            //
            // in the meantime, inline the definition from `Transaction`
            transaction
                .actions()
                .filter_map(|action| match action {
                    penumbra_transaction::Action::Spend(spend) => Some(spend.body.nullifier),
                    /*
                    penumbra_transaction::Action::SwapClaim(swap_claim) => {
                        Some(swap_claim.body.nullifier)
                    }
                     */
                    _ => None,
                })
                .next()
        } else {
            None
        };

        if let Some(nullifier) = nullifier {
            tracing::info!(?nullifier, "waiting for detection of nullifier");
            let detection = self.storage.nullifier_status(nullifier, true);
            tokio::time::timeout(std::time::Duration::from_secs(20), detection)
                .await
                .context("timeout waiting to detect nullifier of submitted transaction")?
                .context("error while waiting for detection of submitted transaction")?;
        }

        Ok(transaction.id())
    }

    async fn tendermint_proxy_client(
        &self,
    ) -> anyhow::Result<TendermintProxyServiceClient<Channel>> {
        let client = TendermintProxyServiceClient::connect(self.node.to_string()).await?;

        Ok(client)
    }

    /// Return the latest block height known by the fullnode or its peers, as
    /// well as whether the fullnode is caught up with that height.
    #[instrument(skip(self))]
    pub async fn latest_known_block_height(&self) -> anyhow::Result<(u64, bool)> {
        let mut client = self.tendermint_proxy_client().await?;

        let rsp = client.get_status(GetStatusRequest {}).await?.into_inner();

        //tracing::debug!("{:#?}", rsp);

        let sync_info = rsp
            .sync_info
            .ok_or_else(|| anyhow::anyhow!("could not parse sync_info in gRPC response"))?;

        let latest_block_height = sync_info.latest_block_height;

        let node_catching_up = sync_info.catching_up;

        // There is a `max_peer_block_height` available in TM 0.35, however it should not be used
        // as it does not seem to reflect the consensus height. Since clients use `latest_known_block_height`
        // to determine the height to attempt syncing to, a validator reporting a non-consensus height
        // can cause a DoS to clients attempting to sync if `max_peer_block_height` is used.
        let latest_known_block_height = latest_block_height;

        tracing::debug!(
            ?latest_block_height,
            ?node_catching_up,
            ?latest_known_block_height
        );

        Ok((latest_known_block_height, node_catching_up))
    }

    #[instrument(skip(self))]
    pub async fn status(&self) -> anyhow::Result<StatusResponse> {
        let full_sync_height = self.storage.last_sync_height().await?.unwrap_or(0);

        let (latest_known_block_height, node_catching_up) =
            self.latest_known_block_height().await?;

        let height_diff = latest_known_block_height
            .checked_sub(full_sync_height)
            .ok_or_else(|| anyhow!("sync height ahead of node height"))?;

        let catching_up = match (node_catching_up, height_diff) {
            // We're synced to the same height as the node
            (false, 0) => false,
            // We're one block behind, and will learn about it soon, close enough
            (false, 1) => false,
            // We're behind the node
            (false, _) => true,
            // The node is behind the network
            (true, _) => true,
        };

        Ok(StatusResponse {
            full_sync_height,
            catching_up,
            partial_sync_height: full_sync_height, // Set these as the same for backwards compatibility following adding the partial_sync_height
        })
    }
}

#[async_trait]
impl ViewProtocolService for ViewService {
    type NotesStream =
        Pin<Box<dyn futures::Stream<Item = Result<pb::NotesResponse, tonic::Status>> + Send>>;
    type NotesForVotingStream = Pin<
        Box<dyn futures::Stream<Item = Result<pb::NotesForVotingResponse, tonic::Status>> + Send>,
    >;
    type AssetsStream =
        Pin<Box<dyn futures::Stream<Item = Result<pb::AssetsResponse, tonic::Status>> + Send>>;
    type StatusStreamStream = Pin<
        Box<dyn futures::Stream<Item = Result<pb::StatusStreamResponse, tonic::Status>> + Send>,
    >;
    type TransactionInfoStream = Pin<
        Box<dyn futures::Stream<Item = Result<pb::TransactionInfoResponse, tonic::Status>> + Send>,
    >;
    type BalancesStream =
        Pin<Box<dyn futures::Stream<Item = Result<pb::BalancesResponse, tonic::Status>> + Send>>;
    type OwnedPositionIdsStream = Pin<
        Box<dyn futures::Stream<Item = Result<pb::OwnedPositionIdsResponse, tonic::Status>> + Send>,
    >;
    type UnclaimedSwapsStream = Pin<
        Box<dyn futures::Stream<Item = Result<pb::UnclaimedSwapsResponse, tonic::Status>> + Send>,
    >;

    async fn broadcast_transaction(
        &self,
        request: tonic::Request<pb::BroadcastTransactionRequest>,
    ) -> Result<tonic::Response<pb::BroadcastTransactionResponse>, tonic::Status> {
        let pb::BroadcastTransactionRequest {
            transaction,
            await_detection,
        } = request.into_inner();

        let transaction: Transaction = transaction
            .ok_or_else(|| tonic::Status::invalid_argument("missing transaction"))?
            .try_into()
            .map_err(|e: anyhow::Error| e.context("could not decode transaction"))
            .map_err(|e| tonic::Status::invalid_argument(format!("{:#}", e)))?;

        let id = self
            .broadcast_transaction(transaction, await_detection)
            .await
            .map_err(|e| {
                tonic::Status::internal(format!("could not broadcast transaction: {:#}", e))
            })?;

        let detection_height = if await_detection {
            // We already awaited detection, so we expect to know about the transaction:
            self.storage
                .transaction_by_hash(&id.0)
                .await
                .map_err(|e| tonic::Status::internal(format!("error querying storage: {:#}", e)))?
                .map(|(height, _tx)| height)
                // If we didn't find it for some reason, return 0 for unknown.
                // TODO: how does this change if we detach extended transaction fetch from scanning?
                .unwrap_or(0)
        } else {
            0
        };

        Ok(tonic::Response::new(pb::BroadcastTransactionResponse {
            id: Some(id.into()),
            detection_height,
        }))
    }

    async fn transaction_planner(
        &self,
        request: tonic::Request<pb::TransactionPlannerRequest>,
    ) -> Result<tonic::Response<pb::TransactionPlannerResponse>, tonic::Status> {
        let prq = request.into_inner();

        let app_params =
            self.storage.app_params().await.map_err(|e| {
                tonic::Status::internal(format!("could not get app params: {:#}", e))
            })?;

        let mut planner = Planner::new(OsRng);
        planner
            .fee(
                match prq.fee {
                    Some(x) => x,
                    None => Fee::default().into(),
                }
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse fee: {e:#}"))
                })?,
            )
            .expiry_height(prq.expiry_height);

        for output in prq.outputs {
            let address: Address = output
                .address
                .ok_or_else(|| tonic::Status::invalid_argument("Missing address"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse address: {e:#}"))
                })?;

            let value: Value = output
                .value
                .ok_or_else(|| tonic::Status::invalid_argument("Missing value"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse value: {e:#}"))
                })?;

            planner.output(value, address);
        }

        for swap in prq.swaps {
            let value: Value = swap
                .value
                .ok_or_else(|| tonic::Status::invalid_argument("Missing value"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse value: {e:#}"))
                })?;

            let target_asset: asset::Id = swap
                .target_asset
                .ok_or_else(|| tonic::Status::invalid_argument("Missing target asset"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse target asset: {e:#}"))
                })?;

            let fee: Fee = swap
                .fee
                .ok_or_else(|| tonic::Status::invalid_argument("Missing fee"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse fee: {e:#}"))
                })?;

            let claim_address: Address = swap
                .claim_address
                .ok_or_else(|| tonic::Status::invalid_argument("Missing claim address"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse claim address: {e:#}"))
                })?;

            planner
                .swap(value, target_asset, fee, claim_address)
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not plan swap: {e:#}"))
                })?;
        }

        for swap_claim in prq.swap_claims {
            let swap_commitment: StateCommitment = swap_claim
                .swap_commitment
                .ok_or_else(|| tonic::Status::invalid_argument("Missing swap commitment"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!(
                        "Could not parse swap commitment: {e:#}"
                    ))
                })?;
            let swap_record = self
                .storage
                // TODO: should there be a timeout on detection here instead?
                .swap_by_commitment(swap_commitment, false)
                .await
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!(
                        "Could not fetch swap by commitment: {e:#}"
                    ))
                })?;

            planner.swap_claim(SwapClaimPlan {
                swap_plaintext: swap_record.swap,
                position: swap_record.position,
                output_data: swap_record.output_data,
                epoch_duration: app_params.chain_params.epoch_duration,
                proof_blinding_r: Fq::rand(&mut OsRng),
                proof_blinding_s: Fq::rand(&mut OsRng),
            });
        }

        for delegation in prq.delegations {
            let amount: Amount = delegation
                .amount
                .ok_or_else(|| tonic::Status::invalid_argument("Missing amount"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse amount: {e:#}"))
                })?;

            let amount_u128: u128 = amount.into();

            let rate_data: RateData = delegation
                .rate_data
                .ok_or_else(|| tonic::Status::invalid_argument("Missing rate data"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse rate data: {e:#}"))
                })?;

            planner.delegate(amount_u128, rate_data);
        }

        for undelegation in prq.undelegations {
            let value: Value = undelegation
                .value
                .ok_or_else(|| tonic::Status::invalid_argument("Missing value"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse value: {e:#}"))
                })?;

            let rate_data: RateData = undelegation
                .rate_data
                .ok_or_else(|| tonic::Status::invalid_argument("Missing rate data"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse rate data: {e:#}"))
                })?;

            planner.undelegate(value.amount, rate_data);
        }

        for position_open in prq.position_opens {
            let position: Position = position_open
                .position
                .ok_or_else(|| tonic::Status::invalid_argument("Missing position"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse position: {e:#}"))
                })?;

            planner.position_open(position);
        }

        for position_close in prq.position_closes {
            let position_id: position::Id = position_close
                .position_id
                .ok_or_else(|| tonic::Status::invalid_argument("Missing position_id"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse position ID: {e:#}"))
                })?;

            planner.position_close(position_id);
        }

        for position_withdraw in prq.position_withdraws {
            let position_id: position::Id = position_withdraw
                .position_id
                .ok_or_else(|| tonic::Status::invalid_argument("Missing position_id"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse position ID: {e:#}"))
                })?;

            let reserves: Reserves = position_withdraw
                .reserves
                .ok_or_else(|| tonic::Status::invalid_argument("Missing reserves"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse reserves: {e:#}"))
                })?;

            let trading_pair: TradingPair = position_withdraw
                .trading_pair
                .ok_or_else(|| tonic::Status::invalid_argument("Missing pair"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse pair: {e:#}"))
                })?;

            planner.position_withdraw(position_id, reserves, trading_pair);
        }

        // Insert any ICS20 withdrawals.
        for ics20_withdrawal in prq.ics20_withdrawals {
            planner.ics20_withdrawal(
                ics20_withdrawal
                    .try_into()
                    .map_err(|e| tonic::Status::invalid_argument(format!("{e:#}")))?,
            );
        }

        // Finally, insert all the requested IBC actions.
        for ibc_action in prq.ibc_relay_actions {
            planner.ibc_action(
                ibc_action
                    .try_into()
                    .map_err(|e| tonic::Status::invalid_argument(format!("{e:#}")))?,
            );
        }

        let mut client_of_self =
            ViewProtocolServiceClient::new(ViewProtocolServiceServer::new(self.clone()));

        let source = prq
            .source
            // If the request specified a source of funds, pass it to the planner...
            .map(|addr_index| addr_index.account)
            // ... or just use the default account if not.
            .unwrap_or(0u32);

        let plan = planner
            .plan(&mut client_of_self, source.into())
            .await
            .context("could not plan requested transaction")
            .map_err(|e| tonic::Status::invalid_argument(format!("{e:#}")))?;

        Ok(tonic::Response::new(TransactionPlannerResponse {
            plan: Some(plan.into()),
        }))
    }

    async fn address_by_index(
        &self,
        request: tonic::Request<pb::AddressByIndexRequest>,
    ) -> Result<tonic::Response<pb::AddressByIndexResponse>, tonic::Status> {
        let fvk =
            self.storage.full_viewing_key().await.map_err(|_| {
                tonic::Status::failed_precondition("Error retrieving full viewing key")
            })?;

        let address_index = request
            .into_inner()
            .address_index
            .ok_or_else(|| tonic::Status::invalid_argument("Missing address index"))?
            .try_into()
            .map_err(|e| {
                tonic::Status::invalid_argument(format!("Could not parse address index: {e:#}"))
            })?;

        Ok(tonic::Response::new(pb::AddressByIndexResponse {
            address: Some(fvk.payment_address(address_index).0.into()),
        }))
    }

    async fn index_by_address(
        &self,
        request: tonic::Request<pb::IndexByAddressRequest>,
    ) -> Result<tonic::Response<pb::IndexByAddressResponse>, tonic::Status> {
        let fvk =
            self.storage.full_viewing_key().await.map_err(|_| {
                tonic::Status::failed_precondition("Error retrieving full viewing key")
            })?;

        let address: Address = request
            .into_inner()
            .address
            .ok_or_else(|| tonic::Status::invalid_argument("Missing address"))?
            .try_into()
            .map_err(|e| {
                tonic::Status::invalid_argument(format!("Could not parse address: {e:#}"))
            })?;

        Ok(tonic::Response::new(pb::IndexByAddressResponse {
            address_index: fvk.address_index(&address).map(Into::into),
        }))
    }

    async fn ephemeral_address(
        &self,
        request: tonic::Request<pb::EphemeralAddressRequest>,
    ) -> Result<tonic::Response<pb::EphemeralAddressResponse>, tonic::Status> {
        let fvk =
            self.storage.full_viewing_key().await.map_err(|_| {
                tonic::Status::failed_precondition("Error retrieving full viewing key")
            })?;

        let address_index = request
            .into_inner()
            .address_index
            .ok_or_else(|| tonic::Status::invalid_argument("Missing address index"))?
            .try_into()
            .map_err(|e| {
                tonic::Status::invalid_argument(format!("Could not parse address index: {e:#}"))
            })?;

        Ok(tonic::Response::new(pb::EphemeralAddressResponse {
            address: Some(fvk.ephemeral_address(OsRng, address_index).0.into()),
        }))
    }

    async fn transaction_info_by_hash(
        &self,
        request: tonic::Request<pb::TransactionInfoByHashRequest>,
    ) -> Result<tonic::Response<pb::TransactionInfoByHashResponse>, tonic::Status> {
        self.check_worker().await?;

        let request = request.into_inner();

        let fvk =
            self.storage.full_viewing_key().await.map_err(|_| {
                tonic::Status::failed_precondition("Error retrieving full viewing key")
            })?;

        let maybe_tx = self
            .storage
            .transaction_by_hash(
                &request
                    .id
                    .clone()
                    .ok_or_else(|| {
                        tonic::Status::invalid_argument(
                            "missing transaction ID in TransactionInfoByHashRequest",
                        )
                    })?
                    .inner,
            )
            .await
            .map_err(|_| {
                tonic::Status::failed_precondition(format!(
                    "Error retrieving transaction by hash {}",
                    hex::encode(request.id.expect("transaction id is present").inner)
                ))
            })?;

        let Some((height, tx)) = maybe_tx else {
            return Ok(tonic::Response::new(
                pb::TransactionInfoByHashResponse::default(),
            ));
        };

        // First, create a TxP with the payload keys visible to our FVK and no other data.
        let mut txp = TransactionPerspective {
            payload_keys: tx
                .payload_keys(&fvk)
                .map_err(|_| tonic::Status::failed_precondition("Error generating payload keys"))?,
            ..Default::default()
        };

        // Next, extend the TxP with the openings of commitments known to our view server
        // but not included in the transaction body, for instance spent notes or swap claim outputs.
        for action in tx.actions() {
            use penumbra_transaction::Action;
            match action {
                Action::Spend(spend) => {
                    let nullifier = spend.body.nullifier;
                    // An error here indicates we don't know the nullifier, so we omit it from the Perspective.
                    if let Ok(spendable_note_record) =
                        self.storage.note_by_nullifier(nullifier, false).await
                    {
                        txp.spend_nullifiers
                            .insert(nullifier, spendable_note_record.note);
                    }
                }
                Action::SwapClaim(claim) => {
                    let output_1_record = self
                        .storage
                        .note_by_commitment(claim.body.output_1_commitment, false)
                        .await
                        .map_err(|e| {
                            tonic::Status::internal(format!(
                                "Error retrieving first SwapClaim output note record: {:#}",
                                e
                            ))
                        })?;
                    let output_2_record = self
                        .storage
                        .note_by_commitment(claim.body.output_2_commitment, false)
                        .await
                        .map_err(|e| {
                            tonic::Status::internal(format!(
                                "Error retrieving second SwapClaim output note record: {:#}",
                                e
                            ))
                        })?;

                    txp.advice_notes
                        .insert(claim.body.output_1_commitment, output_1_record.note);
                    txp.advice_notes
                        .insert(claim.body.output_2_commitment, output_2_record.note);
                }
                _ => {}
            }
        }

        // Now, generate a stub TxV from our minimal TxP, and inspect it to see what data we should
        // augment the minimal TxP with to provide additional context (e.g., filling in denoms for
        // visible asset IDs).
        let min_view = tx.view_from_perspective(&txp);
        let mut address_views = BTreeMap::new();
        let mut asset_ids = BTreeSet::new();
        for action_view in min_view.action_views() {
            use penumbra_dex::{swap::SwapView, swap_claim::SwapClaimView};
            use penumbra_transaction::view::action_view::{
                ActionView, DelegatorVoteView, OutputView, SpendView,
            };
            match action_view {
                ActionView::Spend(SpendView::Visible { note, .. }) => {
                    let address = note.address();
                    address_views.insert(address, fvk.view_address(address));
                    asset_ids.insert(note.asset_id());
                }
                ActionView::Output(OutputView::Visible { note, .. }) => {
                    let address = note.address();
                    address_views.insert(address, fvk.view_address(address));
                    asset_ids.insert(note.asset_id());

                    // Also add an AddressView for the return address in the memo.
                    let memo = tx.decrypt_memo(&fvk).map_err(|_| {
                        tonic::Status::internal("Error decrypting memo for OutputView")
                    })?;
                    address_views.insert(memo.return_address(), fvk.view_address(address));
                }
                ActionView::Swap(SwapView::Visible { swap_plaintext, .. }) => {
                    let address = swap_plaintext.claim_address;
                    address_views.insert(address, fvk.view_address(address));
                    asset_ids.insert(swap_plaintext.trading_pair.asset_1());
                    asset_ids.insert(swap_plaintext.trading_pair.asset_2());
                }
                ActionView::SwapClaim(SwapClaimView::Visible {
                    output_1, output_2, ..
                }) => {
                    // Both will be sent to the same address so this only needs to be added once
                    let address = output_1.address();
                    address_views.insert(address, fvk.view_address(address));
                    asset_ids.insert(output_1.asset_id());
                    asset_ids.insert(output_2.asset_id());
                }
                ActionView::DelegatorVote(DelegatorVoteView::Visible { note, .. }) => {
                    let address = note.address();
                    address_views.insert(address, fvk.view_address(address));
                    asset_ids.insert(note.asset_id());
                }
                _ => {}
            }
        }

        // Now, extend the TxV with information helpful to understand the data it can view:

        let mut denoms = Vec::new();

        for id in asset_ids {
            if let Some(asset) = self.storage.asset_by_id(&id).await.map_err(|e| {
                tonic::Status::internal(format!("Error retrieving asset by id: {:#}", e))
            })? {
                denoms.push(asset);
            }
        }

        txp.denoms.extend(denoms);

        txp.address_views = address_views.into_values().collect();

        // Finally, compute the full TxV from the full TxP:
        let txv = tx.view_from_perspective(&txp);

        let response = pb::TransactionInfoByHashResponse {
            tx_info: Some(pb::TransactionInfo {
                height,
                id: Some(tx.id().into()),
                perspective: Some(txp.into()),
                transaction: Some(tx.into()),
                view: Some(txv.into()),
            }),
        };

        Ok(tonic::Response::new(response))
    }

    async fn swap_by_commitment(
        &self,
        request: tonic::Request<pb::SwapByCommitmentRequest>,
    ) -> Result<tonic::Response<pb::SwapByCommitmentResponse>, tonic::Status> {
        self.check_worker().await?;

        let request = request.into_inner();

        let swap_commitment = request
            .swap_commitment
            .ok_or_else(|| {
                tonic::Status::failed_precondition("Missing swap commitment in request")
            })?
            .try_into()
            .map_err(|_| {
                tonic::Status::failed_precondition("Invalid swap commitment in request")
            })?;

        let swap = pb::SwapRecord::from(
            self.storage
                .swap_by_commitment(swap_commitment, request.await_detection)
                .await
                .map_err(|e| tonic::Status::internal(format!("error: {e}")))?,
        );

        Ok(tonic::Response::new(SwapByCommitmentResponse {
            swap: Some(swap),
        }))
    }

    #[instrument(skip(self, request))]
    async fn balances(
        &self,
        request: tonic::Request<pb::BalancesRequest>,
    ) -> Result<tonic::Response<Self::BalancesStream>, tonic::Status> {
        let request = request.into_inner();

        let account_filter = request.account_filter.and_then(|x| {
            AddressIndex::try_from(x)
                .map_err(|_| {
                    tonic::Status::failed_precondition("Invalid swap commitment in request")
                })
                .map_or(None, |x| x.into())
        });

        let asset_id_filter = request.asset_id_filter.and_then(|x| {
            asset::Id::try_from(x)
                .map_err(|_| {
                    tonic::Status::failed_precondition("Invalid swap commitment in request")
                })
                .map_or(None, |x| x.into())
        });

        let result = self
            .storage
            .balances(account_filter, asset_id_filter)
            .await
            .map_err(|e| tonic::Status::internal(format!("error: {e}")))?;

        tracing::debug!(?account_filter, ?asset_id_filter, ?result);

        let stream = try_stream! {
            for element in result {
                yield pb::BalancesResponse {
                    account: account_filter.clone().map(Into::into),
                    balance: Some(Value {
                        asset_id: element.0.into(),
                        amount: element.1.into(),
                    }.into()),

                }
            }
        };

        Ok(tonic::Response::new(
            stream
                .map_err(|e: anyhow::Error| {
                    tonic::Status::unavailable(format!("error getting balances: {e}"))
                })
                .boxed(),
        ))
    }

    async fn note_by_commitment(
        &self,
        request: tonic::Request<pb::NoteByCommitmentRequest>,
    ) -> Result<tonic::Response<pb::NoteByCommitmentResponse>, tonic::Status> {
        self.check_worker().await?;

        let request = request.into_inner();

        let note_commitment = request
            .note_commitment
            .ok_or_else(|| {
                tonic::Status::failed_precondition("Missing note commitment in request")
            })?
            .try_into()
            .map_err(|_| {
                tonic::Status::failed_precondition("Invalid note commitment in request")
            })?;

        let spendable_note = pb::SpendableNoteRecord::from(
            self.storage
                .note_by_commitment(note_commitment, request.await_detection)
                .await
                .map_err(|e| tonic::Status::internal(format!("error: {e}")))?,
        );

        Ok(tonic::Response::new(NoteByCommitmentResponse {
            spendable_note: Some(spendable_note),
        }))
    }

    async fn nullifier_status(
        &self,
        request: tonic::Request<pb::NullifierStatusRequest>,
    ) -> Result<tonic::Response<pb::NullifierStatusResponse>, tonic::Status> {
        self.check_worker().await?;

        let request = request.into_inner();

        let nullifier = request
            .nullifier
            .ok_or_else(|| tonic::Status::failed_precondition("Missing nullifier in request"))?
            .try_into()
            .map_err(|_| tonic::Status::failed_precondition("Invalid nullifier in request"))?;

        Ok(tonic::Response::new(pb::NullifierStatusResponse {
            spent: self
                .storage
                .nullifier_status(nullifier, request.await_detection)
                .await
                .map_err(|e| tonic::Status::internal(format!("error: {e}")))?,
        }))
    }

    async fn status(
        &self,
        _: tonic::Request<pb::StatusRequest>,
    ) -> Result<tonic::Response<pb::StatusResponse>, tonic::Status> {
        self.check_worker().await?;

        Ok(tonic::Response::new(self.status().await.map_err(|e| {
            tonic::Status::internal(format!("error: {e}"))
        })?))
    }

    async fn status_stream(
        &self,
        _: tonic::Request<pb::StatusStreamRequest>,
    ) -> Result<tonic::Response<Self::StatusStreamStream>, tonic::Status> {
        self.check_worker().await?;

        let (latest_known_block_height, _) =
            self.latest_known_block_height().await.map_err(|e| {
                tonic::Status::unknown(format!(
                    "unable to fetch latest known block height from fullnode: {e}"
                ))
            })?;

        // Create a stream of sync height updates from our worker, and send them to the client
        // until we've reached the latest known block height at the time the request was made.
        let mut sync_height_stream = WatchStream::new(self.sync_height_rx.clone());
        let stream = try_stream! {
            while let Some(sync_height) = sync_height_stream.next().await {
                yield pb::StatusStreamResponse {
                    latest_known_block_height,
                    full_sync_height: sync_height,
                    partial_sync_height: sync_height, // Set these as the same for backwards compatibility following adding the partial_sync_height
                };
                if sync_height >= latest_known_block_height {
                    break;
                }
            }
        };

        Ok(tonic::Response::new(stream.boxed()))
    }

    async fn notes(
        &self,
        request: tonic::Request<pb::NotesRequest>,
    ) -> Result<tonic::Response<Self::NotesStream>, tonic::Status> {
        self.check_worker().await?;

        let request = request.into_inner();

        let include_spent = request.include_spent;
        let asset_id = request
            .asset_id
            .to_owned()
            .map(asset::Id::try_from)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|_| tonic::Status::invalid_argument("invalid asset id"))?;
        let address_index = request
            .address_index
            .to_owned()
            .map(AddressIndex::try_from)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|_| tonic::Status::invalid_argument("invalid address index"))?;

        let amount_to_spend = request
            .amount_to_spend
            .map(Amount::try_from)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|_| tonic::Status::invalid_argument("invalid amount to spend"))?;

        let notes = self
            .storage
            .notes(include_spent, asset_id, address_index, amount_to_spend)
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error fetching notes: {e}")))?;

        let stream = try_stream! {
            for note in notes {
                yield pb::NotesResponse {
                    note_record: Some(note.into()),
                }
            }
        };

        Ok(tonic::Response::new(
            stream
                .map_err(|e: anyhow::Error| {
                    tonic::Status::unavailable(format!("error getting notes: {e}"))
                })
                .boxed(),
        ))
    }

    async fn notes_for_voting(
        &self,
        request: tonic::Request<pb::NotesForVotingRequest>,
    ) -> Result<tonic::Response<Self::NotesForVotingStream>, tonic::Status> {
        self.check_worker().await?;

        let address_index = request
            .get_ref()
            .address_index
            .to_owned()
            .map(AddressIndex::try_from)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|_| tonic::Status::invalid_argument("invalid address index"))?;

        let votable_at_height = request.get_ref().votable_at_height;

        let notes = self
            .storage
            .notes_for_voting(address_index, votable_at_height)
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error fetching notes: {e}")))?;

        let stream = try_stream! {
            for (note, identity_key) in notes {
                yield pb::NotesForVotingResponse {
                    note_record: Some(note.into()),
                    identity_key: Some(identity_key.into()),
                }
            }
        };

        Ok(tonic::Response::new(
            stream
                .map_err(|e: anyhow::Error| {
                    tonic::Status::unavailable(format!("error getting notes: {e}"))
                })
                .boxed(),
        ))
    }

    async fn assets(
        &self,
        request: tonic::Request<pb::AssetsRequest>,
    ) -> Result<tonic::Response<Self::AssetsStream>, tonic::Status> {
        self.check_worker().await?;

        let pb::AssetsRequest {
            filtered,
            include_specific_denominations,
            include_delegation_tokens,
            include_unbonding_tokens,
            include_lp_nfts,
            include_proposal_nfts,
            include_voting_receipt_tokens,
        } = request.get_ref();

        // Fetch assets from storage.
        let assets = if !filtered {
            self.storage
                .all_assets()
                .await
                .map_err(|e| tonic::Status::unavailable(format!("error fetching assets: {e}")))?
        } else {
            let mut assets = vec![];
            for denom in include_specific_denominations {
                if let Some(denom) = asset::REGISTRY.parse_denom(&denom.denom) {
                    assets.push(denom);
                }
            }
            for (include, pattern) in [
                (include_delegation_tokens, "_delegation\\_%"),
                (include_unbonding_tokens, "_unbonding\\_%"),
                (include_lp_nfts, "lpnft\\_%"),
                (include_proposal_nfts, "proposal\\_%"),
                (include_voting_receipt_tokens, "voted\\_on\\_%"),
            ] {
                if *include {
                    assets.extend(
                        self.storage
                            .assets_matching(pattern.to_string())
                            .await
                            .map_err(|e| {
                                tonic::Status::unavailable(format!("error fetching assets: {e}"))
                            })?,
                    );
                }
            }
            assets
        };

        let stream = try_stream! {
            for asset in assets {
                yield
                    pb::AssetsResponse {
                        denom_metadata: Some(asset.into()),
                    }
            }
        };

        Ok(tonic::Response::new(
            stream
                .map_err(|e: anyhow::Error| {
                    tonic::Status::unavailable(format!("error getting assets: {e}"))
                })
                .boxed(),
        ))
    }

    async fn transaction_info(
        &self,
        request: tonic::Request<pb::TransactionInfoRequest>,
    ) -> Result<tonic::Response<Self::TransactionInfoStream>, tonic::Status> {
        self.check_worker().await?;
        // Unpack optional start/end heights.
        let start_height = if request.get_ref().start_height == 0 {
            None
        } else {
            Some(request.get_ref().start_height)
        };
        let end_height = if request.get_ref().end_height == 0 {
            None
        } else {
            Some(request.get_ref().end_height)
        };

        // Fetch transactions from storage.
        let txs = self
            .storage
            .transactions(start_height, end_height)
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error fetching transactions: {e}")))?;

        let self2 = self.clone();
        let stream = try_stream! {
            for tx in txs {

                let rsp = self2.transaction_info_by_hash(tonic::Request::new(pb::TransactionInfoByHashRequest {
                    id: Some(tx.2.id().into()),
                })).await?.into_inner();

                yield pb::TransactionInfoResponse {
                    tx_info: rsp.tx_info,
                }
            }
        };

        Ok(tonic::Response::new(
            stream
                .map_err(|e: anyhow::Error| {
                    tonic::Status::unavailable(format!("error getting transactions: {e}"))
                })
                .boxed(),
        ))
    }

    async fn witness(
        &self,
        request: tonic::Request<pb::WitnessRequest>,
    ) -> Result<tonic::Response<WitnessResponse>, tonic::Status> {
        self.check_worker().await?;

        // Acquire a read lock for the SCT that will live for the entire request,
        // so that all auth paths are relative to the same SCT root.
        let sct = self.state_commitment_tree.read().await;

        // Read the SCT root
        let anchor = sct.root();

        // Obtain an auth path for each requested note commitment
        let requested_note_commitments = request
            .get_ref()
            .note_commitments
            .iter()
            .map(|nc| StateCommitment::try_from(nc.clone()))
            .collect::<Result<Vec<StateCommitment>, _>>()
            .map_err(|_| {
                tonic::Status::new(
                    tonic::Code::InvalidArgument,
                    "Unable to deserialize note commitment",
                )
            })?;

        tracing::debug!(?requested_note_commitments);

        let auth_paths: Vec<Proof> = requested_note_commitments
            .iter()
            .map(|nc| {
                sct.witness(*nc).ok_or_else(|| {
                    tonic::Status::new(tonic::Code::InvalidArgument, "Note commitment missing")
                })
            })
            .collect::<Result<Vec<Proof>, tonic::Status>>()?;

        // Release the read lock on the SCT
        drop(sct);

        let mut witness_data = WitnessData {
            anchor,
            state_commitment_proofs: auth_paths
                .into_iter()
                .map(|proof| (proof.commitment(), proof))
                .collect(),
        };

        tracing::debug!(?witness_data);

        let tx_plan: TransactionPlan =
            request
                .get_ref()
                .to_owned()
                .transaction_plan
                .map_or(TransactionPlan::default(), |x| {
                    x.try_into()
                        .expect("TransactionPlan should exist in request")
                });

        // Now we need to augment the witness data with dummy proofs such that
        // note commitments corresponding to dummy spends also have proofs.
        for nc in tx_plan
            .spend_plans()
            .filter(|plan| plan.note.amount() == 0u64.into())
            .map(|plan| plan.note.commit())
        {
            witness_data.add_proof(nc, Proof::dummy(&mut OsRng, nc));
        }

        let witness_response = WitnessResponse {
            witness_data: Some(witness_data.into()),
        };
        Ok(tonic::Response::new(witness_response))
    }

    async fn witness_and_build(
        &self,
        request: tonic::Request<pb::WitnessAndBuildRequest>,
    ) -> Result<tonic::Response<pb::WitnessAndBuildResponse>, tonic::Status> {
        let pb::WitnessAndBuildRequest {
            transaction_plan,
            authorization_data,
        } = request.into_inner();

        let transaction_plan: TransactionPlan = transaction_plan
            .ok_or_else(|| tonic::Status::invalid_argument("missing transaction plan"))?
            .try_into()
            .map_err(|e: anyhow::Error| e.context("could not decode transaction plan"))
            .map_err(|e| tonic::Status::invalid_argument(format!("{:#}", e)))?;

        // Get the witness data from the view service only for non-zero amounts of value,
        // since dummy spends will have a zero amount.
        let note_commitments = transaction_plan
            .spend_plans()
            .filter(|plan| plan.note.amount() != 0u64.into())
            .map(|spend| spend.note.commit().into())
            .chain(
                transaction_plan
                    .swap_claim_plans()
                    .map(|swap_claim| swap_claim.swap_plaintext.swap_commitment().into()),
            )
            .chain(
                transaction_plan
                    .delegator_vote_plans()
                    .map(|vote_plan| vote_plan.staked_note.commit().into()),
            )
            .collect();

        let authorization_data: AuthorizationData = authorization_data
            .ok_or_else(|| tonic::Status::invalid_argument("missing authorization data"))?
            .try_into()
            .map_err(|e: anyhow::Error| e.context("could not decode authorization data"))
            .map_err(|e| tonic::Status::invalid_argument(format!("{:#}", e)))?;

        let witness_request = pb::WitnessRequest {
            note_commitments,
            transaction_plan: Some(transaction_plan.clone().into()),
        };

        let witness_data: WitnessData = self
            .witness(tonic::Request::new(witness_request))
            .await?
            .into_inner()
            .witness_data
            .ok_or_else(|| tonic::Status::invalid_argument("missing witness data"))?
            .try_into()
            .map_err(|e: anyhow::Error| e.context("could not decode witness data"))
            .map_err(|e| tonic::Status::invalid_argument(format!("{:#}", e)))?;

        let fvk =
            self.storage.full_viewing_key().await.map_err(|_| {
                tonic::Status::failed_precondition("Error retrieving full viewing key")
            })?;

        let transaction = Some(
            transaction_plan
                .build(&fvk, &witness_data, &authorization_data)
                .map_err(|_| tonic::Status::failed_precondition("Error building transaction"))?
                .into(),
        );

        Ok(tonic::Response::new(pb::WitnessAndBuildResponse {
            transaction,
        }))
    }

    async fn app_parameters(
        &self,
        _request: tonic::Request<pb::AppParametersRequest>,
    ) -> Result<tonic::Response<pb::AppParametersResponse>, tonic::Status> {
        self.check_worker().await?;

        let parameters =
            self.storage.app_params().await.map_err(|e| {
                tonic::Status::unavailable(format!("error getting app params: {e}"))
            })?;

        let response = AppParametersResponse {
            parameters: Some(parameters.into()),
        };

        Ok(tonic::Response::new(response))
    }

    async fn gas_prices(
        &self,
        _request: tonic::Request<pb::GasPricesRequest>,
    ) -> Result<tonic::Response<pb::GasPricesResponse>, tonic::Status> {
        self.check_worker().await?;

        let gas_prices =
            self.storage.gas_prices().await.map_err(|e| {
                tonic::Status::unavailable(format!("error getting gas prices: {e}"))
            })?;

        let response = GasPricesResponse {
            gas_prices: Some(gas_prices.into()),
        };

        Ok(tonic::Response::new(response))
    }

    async fn fmd_parameters(
        &self,
        _request: tonic::Request<pb::FmdParametersRequest>,
    ) -> Result<tonic::Response<pb::FmdParametersResponse>, tonic::Status> {
        self.check_worker().await?;

        let parameters =
            self.storage.fmd_parameters().await.map_err(|e| {
                tonic::Status::unavailable(format!("error getting FMD params: {e}"))
            })?;

        let response = FmdParametersResponse {
            parameters: Some(parameters.into()),
        };

        Ok(tonic::Response::new(response))
    }

    async fn owned_position_ids(
        &self,
        request: tonic::Request<pb::OwnedPositionIdsRequest>,
    ) -> Result<tonic::Response<Self::OwnedPositionIdsStream>, tonic::Status> {
        self.check_worker().await?;

        let pb::OwnedPositionIdsRequest {
            position_state,
            trading_pair,
        } = request.into_inner();

        let position_state: Option<position::State> = position_state
            .map(|state| state.try_into())
            .transpose()
            .map_err(|e: anyhow::Error| e.context("could not decode position state"))
            .map_err(|e| tonic::Status::invalid_argument(format!("{:#}", e)))?;

        let trading_pair: Option<TradingPair> = trading_pair
            .map(|pair| pair.try_into())
            .transpose()
            .map_err(|e: anyhow::Error| e.context("could not decode trading pair"))
            .map_err(|e| tonic::Status::invalid_argument(format!("{:#}", e)))?;

        let ids = self
            .storage
            .owned_position_ids(position_state, trading_pair)
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error getting position ids: {e}")))?;

        let stream = try_stream! {
            for id in ids {
                yield pb::OwnedPositionIdsResponse{
                    position_id: Some(id.into()),
                }
            }
        };

        Ok(tonic::Response::new(
            stream
                .map_err(|e: anyhow::Error| {
                    tonic::Status::unavailable(format!("error getting position ids: {e}"))
                })
                .boxed(),
        ))
    }

    async fn authorize_and_build(
        &self,
        _request: tonic::Request<pb::AuthorizeAndBuildRequest>,
    ) -> Result<tonic::Response<pb::AuthorizeAndBuildResponse>, tonic::Status> {
        unimplemented!("authorize_and_build")
    }

    async fn unclaimed_swaps(
        &self,
        _: tonic::Request<pb::UnclaimedSwapsRequest>,
    ) -> Result<tonic::Response<Self::UnclaimedSwapsStream>, tonic::Status> {
        self.check_worker().await?;

        let swaps = self.storage.unclaimed_swaps().await.map_err(|e| {
            tonic::Status::unavailable(format!("error fetching unclaimed swaps: {e}"))
        })?;

        let stream = try_stream! {
            for swap in swaps {
                yield pb::UnclaimedSwapsResponse{
                    swap: Some(swap.into()),
                }
            }
        };

        Ok(tonic::Response::new(
            stream
                .map_err(|e: anyhow::Error| {
                    tonic::Status::unavailable(format!("error getting unclaimed swaps: {e}"))
                })
                .boxed(),
        ))
    }

    async fn wallet_id(
        &self,
        _: Request<WalletIdRequest>,
    ) -> Result<Response<WalletIdResponse>, Status> {
        let fvk = self.storage.full_viewing_key().await.map_err(|e| {
            Status::failed_precondition(format!("Error retrieving full viewing key: {e}"))
        })?;

        Ok(Response::new(WalletIdResponse {
            wallet_id: Some(fvk.wallet_id().into()),
        }))
    }
}
