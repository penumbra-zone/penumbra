use std::{
    collections::{BTreeMap, BTreeSet},
    pin::Pin,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Context};
use async_stream::try_stream;
use camino::Utf8Path;
use decaf377::Fq;
use futures::stream::{self, StreamExt, TryStreamExt};
use penumbra_sdk_auction::auction::dutch::actions::view::{
    ActionDutchAuctionScheduleView, ActionDutchAuctionWithdrawView,
};
use rand::Rng;
use rand_core::OsRng;
use tap::{Tap, TapFallible};
use tokio::sync::{watch, RwLock};
use tokio_stream::wrappers::WatchStream;
use tonic::transport::channel::ClientTlsConfig;
use tonic::transport::channel::Endpoint;
use tonic::{async_trait, transport::Channel, Request, Response, Status};
use tracing::{instrument, Instrument};
use url::Url;

use penumbra_sdk_asset::{asset, asset::Metadata, Value};
use penumbra_sdk_dex::{
    lp::{
        position::{self, Position},
        Reserves,
    },
    swap_claim::SwapClaimPlan,
    TradingPair,
};
use penumbra_sdk_fee::Fee;
use penumbra_sdk_keys::{
    keys::WalletId,
    keys::{AddressIndex, FullViewingKey},
    Address, AddressView,
};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{
    util::tendermint_proxy::v1::{
        tendermint_proxy_service_client::TendermintProxyServiceClient, BroadcastTxSyncRequest,
        GetStatusRequest, GetStatusResponse, SyncInfo,
    },
    view::v1::{
        self as pb,
        broadcast_transaction_response::{BroadcastSuccess, Confirmed, Status as BroadcastStatus},
        view_service_client::ViewServiceClient,
        view_service_server::{ViewService, ViewServiceServer},
        AppParametersResponse, AssetMetadataByIdRequest, AssetMetadataByIdResponse,
        BroadcastTransactionResponse, FmdParametersResponse, GasPricesResponse,
        NoteByCommitmentResponse, StatusResponse, SwapByCommitmentResponse,
        TransactionPlannerResponse, WalletIdRequest, WalletIdResponse, WitnessResponse,
    },
    DomainType,
};
use penumbra_sdk_stake::{rate::RateData, IdentityKey};
use penumbra_sdk_tct::{Proof, StateCommitment};
use penumbra_sdk_transaction::{
    AuthorizationData, Transaction, TransactionPerspective, TransactionPlan, WitnessData,
};

use crate::{worker::Worker, Planner, SpendableNoteRecord, Storage};

/// A [`futures::Stream`] of broadcast transaction responses.
///
/// See [`ViewService::broadcast_transaction()`].
type BroadcastTransactionStream = Pin<
    Box<dyn futures::Stream<Item = Result<pb::BroadcastTransactionResponse, tonic::Status>> + Send>,
>;

/// A service that synchronizes private chain state and responds to queries
/// about it.
///
/// The [`ViewServer`] implements the Tonic-derived [`ViewService`] trait,
/// so it can be used as a gRPC server, or called directly.  It spawns a task
/// internally that performs synchronization and scanning.  The
/// [`ViewServer`] can be cloned; each clone will read from the same shared
/// state, but there will only be a single scanning task.
#[derive(Clone)]
pub struct ViewServer {
    storage: Storage,
    // A shared error slot for errors bubbled up by the worker. This is a regular Mutex
    // rather than a Tokio Mutex because it should be uncontended.
    error_slot: Arc<Mutex<Option<anyhow::Error>>>,
    // A copy of the SCT used by the worker task.
    state_commitment_tree: Arc<RwLock<penumbra_sdk_tct::Tree>>,
    // The Url for the pd gRPC endpoint on remote node.
    node: Url,
    /// Used to watch for changes to the sync height.
    sync_height_rx: watch::Receiver<u64>,
}

impl ViewServer {
    /// Convenience method that calls [`Storage::load_or_initialize`] and then [`Self::new`].
    pub async fn load_or_initialize(
        storage_path: Option<impl AsRef<Utf8Path>>,
        registry_path: Option<impl AsRef<Utf8Path>>,
        fvk: &FullViewingKey,
        node: Url,
    ) -> anyhow::Result<Self> {
        let storage = Storage::load_or_initialize(storage_path, fvk, node.clone())
            .tap(|_| tracing::trace!("loading or initializing storage"))
            .await?
            .tap(|_| tracing::debug!("storage is ready"));

        if let Some(registry_path) = registry_path {
            storage.load_asset_metadata(registry_path).await?;
        }

        Self::new(storage, node)
            .tap(|_| tracing::trace!("constructing view server"))
            .await
            .tap(|_| tracing::debug!("constructed view server"))
    }

    /// Constructs a new [`ViewService`], spawning a sync task internally.
    ///
    /// The sync task uses the provided `client` to sync with the chain.
    ///
    /// To create multiple [`ViewService`]s, clone the [`ViewService`] returned
    /// by this method, rather than calling it multiple times.  That way, each clone
    /// will be backed by the same scanning task, rather than each spawning its own.
    pub async fn new(storage: Storage, node: Url) -> anyhow::Result<Self> {
        let span = tracing::error_span!(parent: None, "view");
        let channel = Self::get_pd_channel(node.clone()).await?;

        let (worker, state_commitment_tree, error_slot, sync_height_rx) =
            Worker::new(storage.clone(), channel)
                .instrument(span.clone())
                .tap(|_| tracing::trace!("constructing view server worker"))
                .await?
                .tap(|_| tracing::debug!("constructed view server worker"));

        tokio::spawn(worker.run().instrument(span))
            .tap(|_| tracing::debug!("spawned view server worker"));

        Ok(Self {
            storage,
            error_slot,
            sync_height_rx,
            state_commitment_tree,
            node,
        })
    }

    /// Obtain a Tonic [Channel] to a remote `pd` endpoint.
    ///
    /// Provided as a convenience method for bootstrapping a connection.
    /// Handles configuring TLS if the URL is HTTPS. Also adds a tracing span
    /// to the working [Channel].
    pub async fn get_pd_channel(node: Url) -> anyhow::Result<Channel> {
        let endpoint = get_pd_endpoint(node).await?;
        let span = tracing::error_span!(parent: None, "view");
        let c: Channel = endpoint
            .connect()
            .instrument(span.clone())
            .await
            .with_context(|| "could not connect to grpc server")
            .tap_err(|error| tracing::error!(?error, "could not connect to grpc server"))?;

        Ok(c)
    }

    /// Checks if the view server worker has encountered an error.
    ///
    /// This function returns a gRPC [`tonic::Status`] containing the view server worker error if
    /// any exists, otherwise it returns `Ok(())`.
    #[instrument(level = "debug", skip_all)]
    async fn check_worker(&self) -> Result<(), tonic::Status> {
        // If the shared error slot is set, then an error has occurred in the worker
        // that we should bubble up.
        tracing::debug!("checking view server worker");
        if let Some(error) = self
            .error_slot
            .lock()
            .tap_err(|error| tracing::error!(?error, "unable to lock worker error slot"))
            .map_err(|e| {
                tonic::Status::unavailable(format!("unable to lock worker error slot {:#}", e))
            })?
            .as_ref()
        {
            return Err(tonic::Status::new(
                tonic::Code::Internal,
                format!("Worker failed: {error}"),
            ));
        }

        // TODO: check whether the worker is still alive, else fail, when we have a way to do that
        // (if the worker is to crash without setting the error_slot, the service should die as well)

        Ok(()).tap(|_| tracing::trace!("view server worker is healthy"))
    }

    #[instrument(skip(self, transaction), fields(id = %transaction.id()))]
    fn broadcast_transaction(
        &self,
        transaction: Transaction,
        await_detection: bool,
    ) -> BroadcastTransactionStream {
        let self2 = self.clone();
        try_stream! {
                // 1. Broadcast the transaction to the network.
                // Note that "synchronous" here means "wait for the tx to be accepted by
                // the fullnode", not "wait for the tx to be included on chain.
                let mut fullnode_client = self2.tendermint_proxy_client().await
                            .map_err(|e| {
                                tonic::Status::unavailable(format!(
                                    "couldn't connect to fullnode: {:#?}",
                                    e
                                ))
                            })?
                        ;
                let node_rsp = fullnode_client
                    .broadcast_tx_sync(BroadcastTxSyncRequest {
                        params: transaction.encode_to_vec(),
                        req_id: OsRng.gen(),
                    })
                    .await
                    .map_err(|e| {
                        tonic::Status::unavailable(format!(
                            "error broadcasting tx: {:#?}",
                            e
                        ))
                    })?
                    .into_inner();
                tracing::info!(?node_rsp);
                match node_rsp.code {
                    0 => Ok(()),
                    _ => Err(tonic::Status::new(
                        tonic::Code::Internal,
                        format!(
                            "Error submitting transaction: code {}, log: {}",
                            node_rsp.code,
                            node_rsp.log,
                        ),
                    )),
                }?;

                // The transaction was submitted so we provide a status update
                yield BroadcastTransactionResponse{ status: Some(BroadcastStatus::BroadcastSuccess(BroadcastSuccess{id:Some(transaction.id().into())}))};

                // 2. Optionally wait for the transaction to be detected by the view service.
                let nullifier = if await_detection {
                    // This needs to be only *spend* nullifiers because the nullifier detection
                    // is broken for swaps, https://github.com/penumbra-zone/penumbra/issues/1749
                    //
                    // in the meantime, inline the definition from `Transaction`
                    transaction
                        .actions()
                        .filter_map(|action| match action {
                            penumbra_sdk_transaction::Action::Spend(spend) => Some(spend.body.nullifier),
                            /*
                            penumbra_sdk_transaction::Action::SwapClaim(swap_claim) => {
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
                    let detection = self2.storage.nullifier_status(nullifier, true);
                    tokio::time::timeout(std::time::Duration::from_secs(20), detection)
                        .await
                        .map_err(|_| {
                            tonic::Status::unavailable(
                                "timeout waiting to detect nullifier of submitted transaction"
                            )
                        })?
                        .map_err(|_| {
                            tonic::Status::unavailable(
                                "error while waiting for detection of submitted transaction"
                            )
                        })?;
                }

                let detection_height = self2.storage
                    .transaction_by_hash(&transaction.id().0)
                    .await
                    .map_err(|e| tonic::Status::internal(format!("error querying storage: {:#}", e)))?
                    .map(|(height, _tx)| height)
                    // If we didn't find it for some reason, return 0 for unknown.
                    // TODO: how does this change if we detach extended transaction fetch from scanning?
                    .unwrap_or(0);
                yield BroadcastTransactionResponse{ status: Some(BroadcastStatus::Confirmed(Confirmed{id:Some(transaction.id().into()), detection_height}))};
            }.boxed()
    }

    #[instrument(level = "trace", skip(self))]
    async fn tendermint_proxy_client(
        &self,
    ) -> anyhow::Result<TendermintProxyServiceClient<Channel>> {
        TendermintProxyServiceClient::connect(self.node.to_string())
            .tap(|_| tracing::debug!("connecting to tendermint proxy"))
            .await
            .tap_err(|error| tracing::error!(?error, "failed to connect to tendermint proxy"))
            .map_err(anyhow::Error::from)
    }

    /// Return the latest block height known by the fullnode or its peers, as
    /// well as whether the fullnode is caught up with that height.
    #[instrument(skip(self))]
    pub async fn latest_known_block_height(&self) -> anyhow::Result<(u64, bool)> {
        let mut client = self.tendermint_proxy_client().await?;

        let GetStatusResponse { sync_info, .. } = client
            .get_status(GetStatusRequest {})
            .tap(|_| tracing::debug!("querying current status"))
            .await
            .tap_err(|error| tracing::debug!(?error, "failed to query current status"))?
            .into_inner();

        let SyncInfo {
            latest_block_height,
            catching_up,
            ..
        } = sync_info
            .ok_or_else(|| anyhow::anyhow!("could not parse sync_info in gRPC response"))?;

        // There is a `max_peer_block_height` available in TM 0.35, however it should not be used
        // as it does not seem to reflect the consensus height. Since clients use `latest_known_block_height`
        // to determine the height to attempt syncing to, a validator reporting a non-consensus height
        // can cause a DoS to clients attempting to sync if `max_peer_block_height` is used.
        let latest_known_block_height = latest_block_height;

        tracing::debug!(
            ?latest_block_height,
            ?catching_up,
            ?latest_known_block_height,
            "found latest known block height"
        );

        Ok((latest_known_block_height, catching_up))
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
impl ViewService for ViewServer {
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
    type BroadcastTransactionStream = BroadcastTransactionStream;
    type WitnessAndBuildStream = Pin<
        Box<dyn futures::Stream<Item = Result<pb::WitnessAndBuildResponse, tonic::Status>> + Send>,
    >;
    type AuthorizeAndBuildStream = Pin<
        Box<
            dyn futures::Stream<Item = Result<pb::AuthorizeAndBuildResponse, tonic::Status>> + Send,
        >,
    >;
    type DelegationsByAddressIndexStream = Pin<
        Box<
            dyn futures::Stream<Item = Result<pb::DelegationsByAddressIndexResponse, tonic::Status>>
                + Send,
        >,
    >;
    type UnbondingTokensByAddressIndexStream = Pin<
        Box<
            dyn futures::Stream<
                    Item = Result<pb::UnbondingTokensByAddressIndexResponse, tonic::Status>,
                > + Send,
        >,
    >;
    type AuctionsStream =
        Pin<Box<dyn futures::Stream<Item = Result<pb::AuctionsResponse, tonic::Status>> + Send>>;
    type LatestSwapsStream =
        Pin<Box<dyn futures::Stream<Item = Result<pb::LatestSwapsResponse, tonic::Status>> + Send>>;
    type LqtVotingNotesStream = Pin<
        Box<dyn futures::Stream<Item = Result<pb::LqtVotingNotesResponse, tonic::Status>> + Send>,
    >;
    type TournamentVotesStream = Pin<
        Box<dyn futures::Stream<Item = Result<pb::TournamentVotesResponse, tonic::Status>> + Send>,
    >;
    type LpPositionBundleStream = Pin<
        Box<dyn futures::Stream<Item = Result<pb::LpPositionBundleResponse, tonic::Status>> + Send>,
    >;
    type LpStrategyCatalogStream = Pin<
        Box<
            dyn futures::Stream<Item = Result<pb::LpStrategyCatalogResponse, tonic::Status>> + Send,
        >,
    >;

    #[instrument(skip_all, level = "trace")]
    async fn auctions(
        &self,
        request: tonic::Request<pb::AuctionsRequest>,
    ) -> Result<tonic::Response<Self::AuctionsStream>, tonic::Status> {
        use penumbra_sdk_proto::core::component::auction::v1 as pb_auction;
        use penumbra_sdk_proto::core::component::auction::v1::query_service_client::QueryServiceClient as AuctionQueryServiceClient;

        let parameters = request.into_inner();
        let query_latest_state = parameters.query_latest_state;
        let include_inactive = parameters.include_inactive;

        let account_filter = parameters
            .account_filter
            .to_owned()
            .map(AddressIndex::try_from)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|_| tonic::Status::invalid_argument("invalid account filter"))?;

        let all_auctions = self
            .storage
            .fetch_auctions_by_account(account_filter, include_inactive)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let client = if query_latest_state {
            Some(
                AuctionQueryServiceClient::connect(self.node.to_string())
                    .await
                    .map_err(|e| tonic::Status::internal(e.to_string()))?,
            )
        } else {
            None
        };

        let responses = futures::future::join_all(all_auctions.into_iter().map(
            |(auction_id, note_record, local_seq)| {
                let maybe_client = client.clone();
                async move {
                    let (any_state, positions) = if let Some(mut client2) = maybe_client {
                        let extra_data = client2
                            .auction_state_by_id(pb_auction::AuctionStateByIdRequest {
                                id: Some(auction_id.into()),
                            })
                            .await
                            .map_err(|e| tonic::Status::internal(e.to_string()))?
                            .into_inner();
                        (extra_data.auction, extra_data.positions)
                    } else {
                        (None, vec![])
                    };

                    Result::<_, tonic::Status>::Ok(pb::AuctionsResponse {
                        id: Some(auction_id.into()),
                        note_record: Some(note_record.into()),
                        auction: any_state,
                        positions,
                        local_seq,
                    })
                }
            },
        ))
        .await;

        let stream = stream::iter(responses)
            .map_err(|e| tonic::Status::internal(format!("error getting auction: {e}")))
            .boxed();

        Ok(Response::new(stream))
    }

    #[instrument(skip_all, level = "trace")]
    async fn broadcast_transaction(
        &self,
        request: tonic::Request<pb::BroadcastTransactionRequest>,
    ) -> Result<tonic::Response<Self::BroadcastTransactionStream>, tonic::Status> {
        let pb::BroadcastTransactionRequest {
            transaction,
            await_detection,
        } = request.into_inner();

        let transaction: Transaction = transaction
            .ok_or_else(|| tonic::Status::invalid_argument("missing transaction"))?
            .try_into()
            .map_err(|e: anyhow::Error| e.context("could not decode transaction"))
            .map_err(|e| tonic::Status::invalid_argument(format!("{:#}", e)))?;

        let stream = self.broadcast_transaction(transaction, await_detection);

        Ok(tonic::Response::new(stream))
    }

    #[instrument(skip_all, level = "trace")]
    async fn transaction_planner(
        &self,
        request: tonic::Request<pb::TransactionPlannerRequest>,
    ) -> Result<tonic::Response<pb::TransactionPlannerResponse>, tonic::Status> {
        let prq = request.into_inner();

        let app_params =
            self.storage.app_params().await.map_err(|e| {
                tonic::Status::internal(format!("could not get app params: {:#}", e))
            })?;

        let gas_prices =
            self.storage.gas_prices().await.map_err(|e| {
                tonic::Status::internal(format!("could not get gas prices: {:#}", e))
            })?;

        // TODO: need to support passing the fee _in_ to this API via the TransactionPlannerRequest
        // meaning the requester should fetch the gas prices and estimate cost/allow the user to modify
        // fee paid
        let mut planner = Planner::new(OsRng);
        planner.set_gas_prices(gas_prices);
        planner.expiry_height(prq.expiry_height);

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
                epoch_duration: app_params.sct_params.epoch_duration,
                proof_blinding_r: Fq::rand(&mut OsRng),
                proof_blinding_s: Fq::rand(&mut OsRng),
            });
        }

        let current_epoch = if prq.undelegations.is_empty() && prq.delegations.is_empty() {
            None
        } else {
            Some(
                prq.epoch
                    .ok_or_else(|| {
                        tonic::Status::invalid_argument(
                            "Missing current epoch in TransactionPlannerRequest",
                        )
                    })?
                    .try_into()
                    .map_err(|e| {
                        tonic::Status::invalid_argument(format!(
                            "Could not parse current epoch: {e:#}"
                        ))
                    })?,
            )
        };

        for delegation in prq.delegations {
            let amount: Amount = delegation
                .amount
                .ok_or_else(|| tonic::Status::invalid_argument("Missing amount"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse amount: {e:#}"))
                })?;

            let rate_data: RateData = delegation
                .rate_data
                .ok_or_else(|| tonic::Status::invalid_argument("Missing rate data"))?
                .try_into()
                .map_err(|e| {
                    tonic::Status::invalid_argument(format!("Could not parse rate data: {e:#}"))
                })?;

            planner.delegate(
                current_epoch.expect("checked that current epoch is present"),
                amount,
                rate_data,
            );
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

            planner.undelegate(
                current_epoch.expect("checked that current epoch is present"),
                value.amount,
                rate_data,
            );
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

            planner.position_withdraw(position_id, reserves, trading_pair, 0);
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

        let mut client_of_self = ViewServiceClient::new(ViewServiceServer::new(self.clone()));

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

    #[instrument(skip_all, level = "trace")]
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

    #[instrument(skip_all, level = "trace")]
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
    async fn transparent_address(
        &self,
        _request: tonic::Request<pb::TransparentAddressRequest>,
    ) -> Result<tonic::Response<pb::TransparentAddressResponse>, tonic::Status> {
        let fvk =
            self.storage.full_viewing_key().await.map_err(|_| {
                tonic::Status::failed_precondition("Error retrieving full viewing key")
            })?;

        let encoding = fvk.incoming().transparent_address();
        let address: Address = encoding
            .parse()
            .map_err(|_| tonic::Status::internal("could not parse newly generated address"))?;

        Ok(tonic::Response::new(pb::TransparentAddressResponse {
            address: Some(address.into()),
            encoding,
        }))
    }

    #[instrument(skip_all, level = "trace")]
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

    #[instrument(skip_all, level = "trace")]
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
            use penumbra_sdk_transaction::Action;
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
            use penumbra_sdk_dex::{swap::SwapView, swap_claim::SwapClaimView};
            use penumbra_sdk_transaction::view::action_view::{
                ActionView, DelegatorVoteView, OutputView, SpendView,
            };
            match action_view {
                ActionView::Spend(SpendView::Visible { note, .. }) => {
                    let address = note.address();
                    address_views.insert(address.clone(), fvk.view_address(address));
                    asset_ids.insert(note.asset_id());
                }
                ActionView::Output(OutputView::Visible { note, .. }) => {
                    let address = note.address();
                    address_views.insert(address.clone(), fvk.view_address(address.clone()));
                    asset_ids.insert(note.asset_id());

                    // Also add an AddressView for the return address in the memo.
                    let memo = tx.decrypt_memo(&fvk).map_err(|_| {
                        tonic::Status::internal("Error decrypting memo for OutputView")
                    })?;
                    address_views.insert(memo.return_address(), fvk.view_address(address));
                }
                ActionView::Swap(SwapView::Visible { swap_plaintext, .. }) => {
                    let address = swap_plaintext.claim_address.clone();
                    address_views.insert(address.clone(), fvk.view_address(address));
                    asset_ids.insert(swap_plaintext.trading_pair.asset_1());
                    asset_ids.insert(swap_plaintext.trading_pair.asset_2());
                }
                ActionView::SwapClaim(SwapClaimView::Visible {
                    output_1, output_2, ..
                }) => {
                    // Both will be sent to the same address so this only needs to be added once
                    let address = output_1.address();
                    address_views.insert(address.clone(), fvk.view_address(address));
                    asset_ids.insert(output_1.asset_id());
                    asset_ids.insert(output_2.asset_id());
                }
                ActionView::DelegatorVote(DelegatorVoteView::Visible { note, .. }) => {
                    let address = note.address();
                    address_views.insert(address.clone(), fvk.view_address(address));
                    asset_ids.insert(note.asset_id());
                }
                ActionView::ActionDutchAuctionWithdraw(ActionDutchAuctionWithdrawView {
                    action: _,
                    reserves,
                }) => {
                    // previous comment: /* no-op for now - i'm not totally sure we have all the necessary data to attribute specific note openings to this view */
                    // to this cronokirby replied: well, we can however at least fill in some asset ids!
                    for value in reserves {
                        asset_ids.insert(value.asset_id());
                    }
                }
                // We can populate asset ids for the assets involved in the auction
                ActionView::ActionDutchAuctionSchedule(ActionDutchAuctionScheduleView {
                    action,
                    ..
                }) => {
                    let description = &action.description;
                    asset_ids.insert(description.input.asset_id);
                    asset_ids.insert(description.output_id);
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
        let summary = txv.summary();

        let response = pb::TransactionInfoByHashResponse {
            tx_info: Some(pb::TransactionInfo {
                height,
                id: Some(tx.id().into()),
                perspective: Some(txp.into()),
                transaction: Some(tx.into()),
                view: Some(txv.into()),
                summary: Some(summary.into()),
            }),
        };

        Ok(tonic::Response::new(response))
    }

    #[instrument(skip_all, level = "trace")]
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

    #[allow(deprecated)]
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

        let self2 = self.clone();
        let stream = try_stream! {
            // retrieve balance and address views
            for element in result {
                let metadata: Metadata = self2
                    .asset_metadata_by_id(Request::new(pb::AssetMetadataByIdRequest {
                        asset_id: Some(element.id.into()),
                    }))
                    .await?
                    .into_inner()
                    .denom_metadata
                    .context("denom metadata not found")?
                    .try_into()?;

                 let value = Value {
                    asset_id: element.id,
                    amount: element.amount.into(),
                };

                let value_view = value.view_with_denom(metadata)?;

                let address: Address = self2
                  .address_by_index(Request::new(pb::AddressByIndexRequest {
                       address_index: account_filter.map(Into::into),
                   }))
                   .await?
                    .into_inner()
                    .address
                    .context("address not found")?
                    .try_into()?;

                 let wallet_id: WalletId = self2
                            .wallet_id(Request::new(pb::WalletIdRequest {}))
                            .await?
                            .into_inner()
                            .wallet_id
                            .context("wallet id not found")?
                            .try_into()?;

                let address_view = AddressView::Decoded {
                    address,
                    index: element.address_index,
                    wallet_id,
                };

                yield pb::BalancesResponse {
                    account_address: Some(address_view.into()),
                    balance_view: Some(value_view.into()),
                    balance: None,
                    account: None,
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

    #[instrument(skip_all, level = "trace")]
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

    #[instrument(skip_all, level = "trace")]
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

    #[instrument(skip_all, level = "trace")]
    async fn status(
        &self,
        _: tonic::Request<pb::StatusRequest>,
    ) -> Result<tonic::Response<pb::StatusResponse>, tonic::Status> {
        self.check_worker().await?;

        Ok(tonic::Response::new(self.status().await.map_err(|e| {
            tonic::Status::internal(format!("error: {e}"))
        })?))
    }

    #[instrument(skip_all, level = "trace")]
    async fn status_stream(
        &self,
        _: tonic::Request<pb::StatusStreamRequest>,
    ) -> Result<tonic::Response<Self::StatusStreamStream>, tonic::Status> {
        self.check_worker().await?;

        let (latest_known_block_height, _) = self
            .latest_known_block_height()
            .await
            .tap_err(|error| {
                tracing::debug!(
                    ?error,
                    "unable to fetch latest known block height from fullnode"
                )
            })
            .map_err(|e| {
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

    #[instrument(skip_all, level = "trace")]
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

    #[instrument(skip_all, level = "trace")]
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

    #[instrument(skip_all, level = "trace")]
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

    #[instrument(skip_all, level = "trace")]
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

    #[instrument(skip_all, level = "trace")]
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
        let tx_plan: TransactionPlan =
            request
                .get_ref()
                .to_owned()
                .transaction_plan
                .map_or(TransactionPlan::default(), |x| {
                    x.try_into()
                        .expect("TransactionPlan should exist in request")
                });

        let requested_note_commitments: Vec<StateCommitment> = tx_plan
            .spend_plans()
            .filter(|plan| plan.note.amount() != 0u64.into())
            .map(|spend| spend.note.commit().into())
            .chain(
                tx_plan
                    .swap_claim_plans()
                    .map(|swap_claim| swap_claim.swap_plaintext.swap_commitment().into()),
            )
            .chain(
                tx_plan
                    .delegator_vote_plans()
                    .map(|vote_plan| vote_plan.staked_note.commit().into()),
            )
            .collect();

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

    #[instrument(skip_all, level = "trace")]
    async fn witness_and_build(
        &self,
        request: tonic::Request<pb::WitnessAndBuildRequest>,
    ) -> Result<tonic::Response<Self::WitnessAndBuildStream>, tonic::Status> {
        let pb::WitnessAndBuildRequest {
            transaction_plan,
            authorization_data,
        } = request.into_inner();

        let transaction_plan: TransactionPlan = transaction_plan
            .ok_or_else(|| tonic::Status::invalid_argument("missing transaction plan"))?
            .try_into()
            .map_err(|e: anyhow::Error| e.context("could not decode transaction plan"))
            .map_err(|e| tonic::Status::invalid_argument(format!("{:#}", e)))?;

        let authorization_data: AuthorizationData = authorization_data
            .ok_or_else(|| tonic::Status::invalid_argument("missing authorization data"))?
            .try_into()
            .map_err(|e: anyhow::Error| e.context("could not decode authorization data"))
            .map_err(|e| tonic::Status::invalid_argument(format!("{:#}", e)))?;

        let witness_request = pb::WitnessRequest {
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
                // TODO: calling `.build` should provide some mechanism to get progress
                // updates
                .build(&fvk, &witness_data, &authorization_data)
                .map_err(|_| tonic::Status::failed_precondition("Error building transaction"))?
                .into(),
        );

        let stream = try_stream! {
            yield pb::WitnessAndBuildResponse {
                status: Some(pb::witness_and_build_response::Status::Complete(
                    pb::witness_and_build_response::Complete { transaction },
                )),
            }
        };

        Ok(tonic::Response::new(
            stream
                .map_err(|e: anyhow::Error| {
                    tonic::Status::unavailable(format!("error witnessing transaction: {e}"))
                })
                .boxed(),
        ))
    }

    #[instrument(skip_all, level = "trace")]
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

    #[instrument(skip_all, level = "trace")]
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
            alt_gas_prices: Vec::new(),
        };

        Ok(tonic::Response::new(response))
    }

    #[instrument(skip_all, level = "trace")]
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

    #[instrument(skip_all, level = "trace")]
    async fn owned_position_ids(
        &self,
        request: tonic::Request<pb::OwnedPositionIdsRequest>,
    ) -> Result<tonic::Response<Self::OwnedPositionIdsStream>, tonic::Status> {
        self.check_worker().await?;

        let pb::OwnedPositionIdsRequest {
            position_state,
            trading_pair,
            subaccount,
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

        let subaccount: Option<AddressIndex> = subaccount
            .map(|a| a.try_into())
            .transpose()
            .map_err(|e: anyhow::Error| e.context("could not decode subaccount"))
            .map_err(|e| tonic::Status::invalid_argument(format!("{:#}", e)))?;

        let ids = self
            .storage
            .owned_position_ids(position_state, trading_pair, subaccount)
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error getting position ids: {e}")))?;

        let stream = try_stream! {
            for id in ids {
                yield pb::OwnedPositionIdsResponse{
                    position_id: Some(id.into()),
                    // We null out the randomizer, reflecting that we didn't use it.
                    subaccount: subaccount.map(|s|
                        AddressIndex { account: s.account, ..Default::default()}.into()
                    ),
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

    #[instrument(skip_all, level = "trace")]
    async fn authorize_and_build(
        &self,
        _request: tonic::Request<pb::AuthorizeAndBuildRequest>,
    ) -> Result<tonic::Response<Self::AuthorizeAndBuildStream>, tonic::Status> {
        unimplemented!("authorize_and_build")
    }

    #[instrument(skip_all, level = "trace")]
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

    #[instrument(skip_all, level = "trace")]
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

    #[instrument(skip_all, level = "trace")]
    async fn asset_metadata_by_id(
        &self,
        request: Request<AssetMetadataByIdRequest>,
    ) -> Result<Response<AssetMetadataByIdResponse>, Status> {
        let asset_id = request
            .into_inner()
            .asset_id
            .ok_or_else(|| Status::invalid_argument("missing asset id"))?
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("{e:#}")))?;

        let metadata = self
            .storage
            .asset_by_id(&asset_id)
            .await
            .map_err(|e| Status::internal(format!("Error retrieving asset by id: {e:#}")))?;

        Ok(Response::new(AssetMetadataByIdResponse {
            denom_metadata: metadata.map(Into::into),
        }))
    }

    #[instrument(skip_all, level = "trace")]
    async fn delegations_by_address_index(
        &self,
        _request: tonic::Request<pb::DelegationsByAddressIndexRequest>,
    ) -> Result<tonic::Response<Self::DelegationsByAddressIndexStream>, tonic::Status> {
        unimplemented!("delegations_by_address_index")
    }

    #[instrument(skip_all, level = "trace")]
    async fn unbonding_tokens_by_address_index(
        &self,
        _request: tonic::Request<pb::UnbondingTokensByAddressIndexRequest>,
    ) -> Result<tonic::Response<Self::UnbondingTokensByAddressIndexStream>, tonic::Status> {
        unimplemented!("unbonding_tokens_by_address_index currently only implemented on web")
    }

    #[instrument(skip_all, level = "trace")]
    async fn latest_swaps(
        &self,
        _request: tonic::Request<pb::LatestSwapsRequest>,
    ) -> Result<tonic::Response<Self::LatestSwapsStream>, tonic::Status> {
        unimplemented!("latest_swaps currently only implemented on web")
    }

    #[instrument(skip_all, level = "trace")]
    async fn tournament_votes(
        &self,
        _request: tonic::Request<pb::TournamentVotesRequest>,
    ) -> Result<tonic::Response<Self::TournamentVotesStream>, tonic::Status> {
        unimplemented!("tournament_votes currently only implemented on web")
    }

    #[instrument(skip_all, level = "trace")]
    async fn lqt_voting_notes(
        &self,
        request: tonic::Request<pb::LqtVotingNotesRequest>,
    ) -> Result<tonic::Response<Self::LqtVotingNotesStream>, tonic::Status> {
        async fn inner(
            this: &ViewServer,
            epoch: u64,
            filter: Option<AddressIndex>,
        ) -> anyhow::Result<Vec<(SpendableNoteRecord, IdentityKey)>> {
            let (_, start_height) = this.storage.get_epoch(epoch).await?;
            let start_height =
                start_height.ok_or_else(|| anyhow!("missing height for epoch {epoch}"))?;
            let notes = this.storage.notes_for_voting(filter, start_height).await?;
            Ok(notes)
        }

        let request = request.into_inner();
        let epoch = request.epoch_index;
        let filter = request
            .account_filter
            .map(|x| AddressIndex::try_from(x))
            .transpose()
            .map_err(|_| tonic::Status::invalid_argument("invalid account filter"))?;
        let notes = inner(self, epoch, filter).await.map_err(|e| {
            tonic::Status::internal(format!("error fetching voting notes: {:#}", e))
        })?;
        let stream = tokio_stream::iter(notes.into_iter().map(|(note, _)| {
            Result::<_, tonic::Status>::Ok(pb::LqtVotingNotesResponse {
                note_record: Some(note.into()),
                already_voted: false,
            })
        }));
        Ok(tonic::Response::new(stream.boxed()))
    }

    #[instrument(skip_all, level = "trace")]
    async fn lp_position_bundle(
        &self,
        _request: tonic::Request<pb::LpPositionBundleRequest>,
    ) -> Result<tonic::Response<Self::LpPositionBundleStream>, tonic::Status> {
        unimplemented!("lp_position_bundle currently only implemented on web")
    }

    #[instrument(skip_all, level = "trace")]
    async fn lp_strategy_catalog(
        &self,
        _request: tonic::Request<pb::LpStrategyCatalogRequest>,
    ) -> Result<tonic::Response<Self::LpStrategyCatalogStream>, tonic::Status> {
        unimplemented!("lp_strategy_catalog currently only implemented on web")
    }
}

/// Convert a pd node URL to a Tonic `Endpoint`.
///
/// Required in order to configure TLS for HTTPS endpoints.
async fn get_pd_endpoint(node: Url) -> anyhow::Result<Endpoint> {
    let endpoint = match node.scheme() {
        "http" => Channel::from_shared(node.to_string())?,
        "https" => Channel::from_shared(node.to_string())?
            .tls_config(ClientTlsConfig::new().with_webpki_roots())?,
        other => anyhow::bail!("unknown url scheme {other}"),
    };
    Ok(endpoint)
}
