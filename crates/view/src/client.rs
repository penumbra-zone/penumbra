use std::{collections::BTreeMap, future::Future, pin::Pin};

use anyhow::Result;
use futures::{FutureExt, Stream, StreamExt, TryStreamExt};
use penumbra_chain::params::{ChainParameters, FmdParameters};
use penumbra_crypto::asset::{DenomMetadata, Id};
use penumbra_crypto::keys::AccountGroupId;
use penumbra_crypto::{asset, keys::AddressIndex, note, Nullifier};
use penumbra_crypto::{stake::IdentityKey, Address, Amount};
use penumbra_proto::view::v1alpha1::{
    self as pb, view_protocol_service_client::ViewProtocolServiceClient, WitnessRequest,
};

use penumbra_transaction::AuthorizationData;
use penumbra_transaction::{plan::TransactionPlan, Transaction, WitnessData};

use tonic::codegen::Bytes;
use tracing::instrument;

use crate::{SpendableNoteRecord, StatusStreamResponse, SwapRecord, TransactionInfo};

/// The view protocol is used by a view client, who wants to do some
/// transaction-related actions, to request data from a view service, which is
/// responsible for synchronizing and scanning the public chain state with one
/// or more full viewing keys.
///
/// This trait is a wrapper around the proto-generated [`ViewProtocolServiceClient`]
/// that serves two goals:
///
/// 1. It can use domain types rather than proto-generated types, avoiding conversions;
/// 2. It's easier to write as a trait bound than the `CustodyProtocolClient`,
///   which requires complex bounds on its inner type to
///   enforce that it is a tower `Service`.
#[allow(clippy::type_complexity)]
pub trait ViewClient {
    /// Get the current status of chain sync.
    fn status(
        &mut self,
        account_group_id: AccountGroupId,
    ) -> Pin<Box<dyn Future<Output = Result<pb::StatusResponse>> + Send + 'static>>;

    /// Stream status updates on chain sync until it completes.
    fn status_stream(
        &mut self,
        account_group_id: AccountGroupId,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Pin<Box<dyn Stream<Item = Result<StatusStreamResponse>> + Send + 'static>>,
                    >,
                > + Send
                + 'static,
        >,
    >;

    /// Get a copy of the chain parameters.
    fn chain_params(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<ChainParameters>> + Send + 'static>>;

    /// Get a copy of the FMD parameters.
    fn fmd_parameters(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<FmdParameters>> + Send + 'static>>;

    /// Queries for notes.
    fn notes(
        &mut self,
        request: pb::NotesRequest,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<SpendableNoteRecord>>> + Send + 'static>>;

    /// Queries for notes for voting.
    fn notes_for_voting(
        &mut self,
        request: pb::NotesForVotingRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<Vec<(SpendableNoteRecord, IdentityKey)>>> + Send + 'static>,
    >;

    /// Queries for account balance by address
    fn balance_by_address(
        &mut self,
        address: Address,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<(Id, Amount)>>> + Send + 'static>>;

    /// Queries for a specific note by commitment, returning immediately if it is not found.
    fn note_by_commitment(
        &mut self,
        account_group_id: AccountGroupId,
        note_commitment: note::StateCommitment,
    ) -> Pin<Box<dyn Future<Output = Result<SpendableNoteRecord>> + Send + 'static>>;

    /// Queries for a specific swap by commitment, returning immediately if it is not found.
    fn swap_by_commitment(
        &mut self,
        account_group_id: AccountGroupId,
        swap_commitment: penumbra_tct::StateCommitment,
    ) -> Pin<Box<dyn Future<Output = Result<SwapRecord>> + Send + 'static>>;

    /// Queries for a specific nullifier's status, returning immediately if it is not found.
    fn nullifier_status(
        &mut self,
        account_group_id: AccountGroupId,
        nullifier: Nullifier,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'static>>;

    /// Waits for a specific nullifier to be detected, returning immediately if it is already
    /// present, but waiting otherwise.
    fn await_nullifier(
        &mut self,
        account_group_id: AccountGroupId,
        nullifier: Nullifier,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;

    /// Queries for a specific note by commitment, waiting until the note is detected if it is not found.
    ///
    /// This is useful for waiting for a note to be detected by the view service.
    fn await_note_by_commitment(
        &mut self,
        account_group_id: AccountGroupId,
        note_commitment: note::StateCommitment,
    ) -> Pin<Box<dyn Future<Output = Result<SpendableNoteRecord>> + Send + 'static>>;

    /// Returns authentication paths for the given note commitments.
    ///
    /// This method takes a batch of input commitments, rather than just one, so
    /// that the client can get a consistent set of authentication paths to a
    /// common root.  (Otherwise, if a client made multiple requests, the wallet
    /// service could have advanced the state commitment tree state between queries).
    fn witness(
        &mut self,
        account_group_id: AccountGroupId,
        plan: &TransactionPlan,
    ) -> Pin<Box<dyn Future<Output = Result<WitnessData>> + Send + 'static>>;

    /// Returns a transaction built from the provided TransactionPlan and AuthorizationData
    fn witness_and_build(
        &mut self,
        plan: TransactionPlan,
        auth_data: AuthorizationData,
    ) -> Pin<Box<dyn Future<Output = Result<Transaction>> + Send + 'static>>;

    /// Queries for all known assets.
    fn assets(&mut self) -> Pin<Box<dyn Future<Output = Result<asset::Cache>> + Send + 'static>>;

    /// Generates a full perspective for a selected transaction using a full viewing key
    fn transaction_info_by_hash(
        &mut self,
        id: penumbra_transaction::Id,
    ) -> Pin<Box<dyn Future<Output = Result<TransactionInfo>> + Send + 'static>>;

    /// Queries for transactions in a range of block heights
    fn transaction_info(
        &mut self,
        start_height: Option<u64>,
        end_height: Option<u64>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<TransactionInfo>>> + Send + 'static>>;

    fn broadcast_transaction(
        &mut self,
        transaction: Transaction,
        await_detection: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(penumbra_transaction::Id, u64)>> + Send + 'static>>;

    /// Return unspent notes, grouped by address index and then by asset id.
    #[instrument(skip(self, account_group_id))]
    fn unspent_notes_by_address_and_asset(
        &mut self,
        account_group_id: AccountGroupId,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        BTreeMap<AddressIndex, BTreeMap<asset::Id, Vec<SpendableNoteRecord>>>,
                    >,
                > + Send
                + 'static,
        >,
    > {
        let notes = self.notes(pb::NotesRequest {
            account_group_id: Some(account_group_id.into()),
            include_spent: false,
            ..Default::default()
        });
        async move {
            let notes = notes.await?;
            tracing::trace!(?notes);

            let mut notes_by_address_and_asset = BTreeMap::new();

            for note_record in notes {
                notes_by_address_and_asset
                    .entry(note_record.address_index)
                    .or_insert_with(BTreeMap::new)
                    .entry(note_record.note.asset_id())
                    .or_insert_with(Vec::new)
                    .push(note_record);
            }
            tracing::trace!(?notes_by_address_and_asset);

            Ok(notes_by_address_and_asset)
        }
        .boxed()
    }

    /// Return unspent notes, grouped by denom and then by address index.
    #[instrument(skip(self, account_group_id))]
    fn unspent_notes_by_asset_and_address(
        &mut self,
        account_group_id: AccountGroupId,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        BTreeMap<asset::Id, BTreeMap<AddressIndex, Vec<SpendableNoteRecord>>>,
                    >,
                > + Send
                + 'static,
        >,
    > {
        let notes = self.notes(pb::NotesRequest {
            account_group_id: Some(account_group_id.into()),
            include_spent: false,
            ..Default::default()
        });

        async move {
            let notes = notes.await?;
            tracing::trace!(?notes);

            let mut notes_by_asset_and_address = BTreeMap::new();

            for note_record in notes {
                notes_by_asset_and_address
                    .entry(note_record.note.asset_id())
                    .or_insert_with(BTreeMap::new)
                    .entry(note_record.address_index)
                    .or_insert_with(Vec::new)
                    .push(note_record);
            }
            tracing::trace!(?notes_by_asset_and_address);

            Ok(notes_by_asset_and_address)
        }
        .boxed()
    }

    fn address_by_index(
        &mut self,
        address_index: AddressIndex,
    ) -> Pin<Box<dyn Future<Output = Result<Address>> + Send + 'static>>;
}

// We need to tell `async_trait` not to add a `Send` bound to the boxed
// futures it generates, because the underlying `CustodyProtocolClient` isn't `Sync`,
// but its `authorize` method takes `&mut self`. This would normally cause a huge
// amount of problems, because non-`Send` futures don't compose well, but as long
// as we're calling the method within an async block on a local mutable variable,
// it should be fine.
impl<T> ViewClient for ViewProtocolServiceClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody> + Clone + Send + 'static,
    T::ResponseBody: tonic::codegen::Body<Data = Bytes> + Send + 'static,
    T::Error: Into<tonic::codegen::StdError>,
    T::Future: Send + 'static,
    <T::ResponseBody as tonic::codegen::Body>::Error: Into<tonic::codegen::StdError> + Send,
{
    fn status(
        &mut self,
        account_group_id: AccountGroupId,
    ) -> Pin<Box<dyn Future<Output = Result<pb::StatusResponse>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let status = self2.status(tonic::Request::new(pb::StatusRequest {
                account_group_id: Some(account_group_id.into()),
                ..Default::default()
            }));
            let status = status.await?.into_inner();
            Ok(status)
        }
        .boxed()
    }

    fn status_stream(
        &mut self,
        account_group_id: AccountGroupId,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Pin<Box<dyn Stream<Item = Result<StatusStreamResponse>> + Send + 'static>>,
                    >,
                > + Send
                + 'static,
        >,
    > {
        let mut self2 = self.clone();
        async move {
            let stream = self2.status_stream(tonic::Request::new(pb::StatusStreamRequest {
                account_group_id: Some(account_group_id.into()),
                ..Default::default()
            }));
            let stream = stream.await?.into_inner();

            Ok(stream
                .map_err(|e| anyhow::anyhow!("view service error: {}", e))
                .and_then(|msg| async move { StatusStreamResponse::try_from(msg) })
                .boxed())
        }
        .boxed()
    }

    fn chain_params(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<ChainParameters>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            // We have to manually invoke the method on the type, because it has the
            // same name as the one we're implementing.
            let rsp = ViewProtocolServiceClient::chain_parameters(
                &mut self2,
                tonic::Request::new(pb::ChainParametersRequest {}),
            );
            rsp.await?.into_inner().try_into()
        }
        .boxed()
    }

    fn fmd_parameters(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<FmdParameters>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let parameters = ViewProtocolServiceClient::fmd_parameters(
                &mut self2,
                tonic::Request::new(pb::FmdParametersRequest {}),
            );
            let parameters = parameters.await?.into_inner().parameters;

            parameters
                .ok_or_else(|| anyhow::anyhow!("empty FmdParametersRequest message"))?
                .try_into()
        }
        .boxed()
    }

    fn notes(
        &mut self,
        request: pb::NotesRequest,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<SpendableNoteRecord>>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let req = self2.notes(tonic::Request::new(request));
            let pb_notes: Vec<_> = req.await?.into_inner().try_collect().await?;

            pb_notes
                .into_iter()
                .map(|note_rsp| {
                    let note_record = note_rsp
                        .note_record
                        .ok_or_else(|| anyhow::anyhow!("empty NotesResponse message"));

                    match note_record {
                        Ok(note) => note.try_into(),
                        Err(e) => Err(e),
                    }
                })
                .collect()
        }
        .boxed()
    }

    fn notes_for_voting(
        &mut self,
        request: pb::NotesForVotingRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<Vec<(SpendableNoteRecord, IdentityKey)>>> + Send + 'static>,
    > {
        let mut self2 = self.clone();
        async move {
            let req = self2.notes_for_voting(tonic::Request::new(request));
            let pb_notes: Vec<_> = req.await?.into_inner().try_collect().await?;

            pb_notes
                .into_iter()
                .map(|note_rsp| {
                    let note_record = note_rsp
                        .note_record
                        .ok_or_else(|| anyhow::anyhow!("empty NotesForVotingResponse message"))?
                        .try_into()?;

                    let identity_key = note_rsp
                        .identity_key
                        .ok_or_else(|| anyhow::anyhow!("empty NotesForVotingResponse message"))?
                        .try_into()?;

                    Ok((note_record, identity_key))
                })
                .collect()
        }
        .boxed()
    }

    fn note_by_commitment(
        &mut self,
        account_group_id: AccountGroupId,
        note_commitment: note::StateCommitment,
    ) -> Pin<Box<dyn Future<Output = Result<SpendableNoteRecord>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let note_commitment_response = ViewProtocolServiceClient::note_by_commitment(
                &mut self2,
                tonic::Request::new(pb::NoteByCommitmentRequest {
                    account_group_id: Some(account_group_id.into()),
                    note_commitment: Some(note_commitment.into()),
                    await_detection: false,
                    ..Default::default()
                }),
            );
            let note_commitment_response = note_commitment_response.await?.into_inner();

            note_commitment_response
                .spendable_note
                .ok_or_else(|| anyhow::anyhow!("empty NoteByCommitmentResponse message"))?
                .try_into()
        }
        .boxed()
    }

    fn balance_by_address(
        &mut self,
        address: Address,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<(Id, Amount)>>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let req = ViewProtocolServiceClient::balance_by_address(
                &mut self2,
                tonic::Request::new(pb::BalanceByAddressRequest {
                    address: Some(address.into()),
                }),
            );

            let balances: Vec<_> = req.await?.into_inner().try_collect().await?;

            balances
                .into_iter()
                .map(|rsp| {
                    let asset = rsp
                        .asset
                        .ok_or_else(|| anyhow::anyhow!("empty asset type"))?
                        .try_into()?;
                    let amount = rsp
                        .amount
                        .ok_or_else(|| anyhow::anyhow!("empty amount type"))?
                        .try_into()?;
                    Ok((asset, amount))
                })
                .collect()
        }
        .boxed()
    }

    fn swap_by_commitment(
        &mut self,
        account_group_id: AccountGroupId,
        swap_commitment: penumbra_tct::StateCommitment,
    ) -> Pin<Box<dyn Future<Output = Result<SwapRecord>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let swap_commitment_response = ViewProtocolServiceClient::swap_by_commitment(
                &mut self2,
                tonic::Request::new(pb::SwapByCommitmentRequest {
                    account_group_id: Some(account_group_id.into()),
                    swap_commitment: Some(swap_commitment.into()),
                    await_detection: false,
                    ..Default::default()
                }),
            );
            let swap_commitment_response = swap_commitment_response.await?.into_inner();

            swap_commitment_response
                .swap
                .ok_or_else(|| anyhow::anyhow!("empty SwapByCommitmentResponse message"))?
                .try_into()
        }
        .boxed()
    }

    /// Queries for a specific note by commitment, waiting until the note is detected if it is not found.
    ///
    /// This is useful for waiting for a note to be detected by the view service.
    fn await_note_by_commitment(
        &mut self,
        account_group_id: AccountGroupId,
        note_commitment: note::StateCommitment,
    ) -> Pin<Box<dyn Future<Output = Result<SpendableNoteRecord>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let spendable_note = ViewProtocolServiceClient::note_by_commitment(
                &mut self2,
                tonic::Request::new(pb::NoteByCommitmentRequest {
                    account_group_id: Some(account_group_id.into()),
                    note_commitment: Some(note_commitment.into()),
                    await_detection: true,
                    ..Default::default()
                }),
            );
            let spendable_note = spendable_note.await?.into_inner().spendable_note;

            spendable_note
                .ok_or_else(|| anyhow::anyhow!("empty NoteByCommitmentRequest message"))?
                .try_into()
        }
        .boxed()
    }

    /// Queries for a specific nullifier's status, returning immediately if it is not found.
    fn nullifier_status(
        &mut self,
        account_group_id: AccountGroupId,
        nullifier: Nullifier,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let rsp = ViewProtocolServiceClient::nullifier_status(
                &mut self2,
                tonic::Request::new(pb::NullifierStatusRequest {
                    account_group_id: Some(account_group_id.into()),
                    nullifier: Some(nullifier.into()),
                    await_detection: false,
                    ..Default::default()
                }),
            );
            Ok(rsp.await?.into_inner().spent)
        }
        .boxed()
    }

    /// Waits for a specific nullifier to be detected, returning immediately if it is already
    /// present, but waiting otherwise.
    fn await_nullifier(
        &mut self,
        account_group_id: AccountGroupId,
        nullifier: Nullifier,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let rsp = ViewProtocolServiceClient::nullifier_status(
                &mut self2,
                tonic::Request::new(pb::NullifierStatusRequest {
                    account_group_id: Some(account_group_id.into()),
                    nullifier: Some(nullifier.into()),
                    await_detection: true,
                    ..Default::default()
                }),
            );
            rsp.await?;
            Ok(())
        }
        .boxed()
    }

    fn witness(
        &mut self,
        account_group_id: AccountGroupId,
        plan: &TransactionPlan,
    ) -> Pin<Box<dyn Future<Output = Result<WitnessData>> + Send + 'static>> {
        // TODO: delete this code and move it into the view service.
        // The caller shouldn't have to massage the transaction plan to make the request.

        // Get the witness data from the view service only for non-zero amounts of value,
        // since dummy spends will have a zero amount.
        let note_commitments = plan
            .spend_plans()
            .filter(|plan| plan.note.amount() != 0u64.into())
            .map(|spend| spend.note.commit().into())
            .chain(
                plan.swap_claim_plans()
                    .map(|swap_claim| swap_claim.swap_plaintext.swap_commitment().into()),
            )
            .chain(
                plan.delegator_vote_plans()
                    .map(|vote_plan| vote_plan.staked_note.commit().into()),
            )
            .collect();

        let request = WitnessRequest {
            account_group_id: Some(account_group_id.into()),
            note_commitments,
            transaction_plan: Some(plan.clone().into()),
            ..Default::default()
        };

        let mut self2 = self.clone();
        async move {
            let rsp = self2.witness(tonic::Request::new(request));

            let witness_data = rsp
                .await?
                .into_inner()
                .witness_data
                .ok_or_else(|| anyhow::anyhow!("empty WitnessResponse message"))?
                .try_into()?;

            Ok(witness_data)
        }
        .boxed()
    }

    fn assets(&mut self) -> Pin<Box<dyn Future<Output = Result<asset::Cache>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            // We have to manually invoke the method on the type, because it has the
            // same name as the one we're implementing.
            let rsp = ViewProtocolServiceClient::assets(
                &mut self2,
                tonic::Request::new(pb::AssetsRequest {
                    ..Default::default()
                }),
            );

            let pb_assets: Vec<_> = rsp.await?.into_inner().try_collect().await?;

            let assets = pb_assets
                .into_iter()
                .map(DenomMetadata::try_from)
                .collect::<Result<Vec<DenomMetadata>, anyhow::Error>>()?;

            Ok(assets.into_iter().collect())
        }
        .boxed()
    }

    fn transaction_info_by_hash(
        &mut self,
        id: penumbra_transaction::Id,
    ) -> Pin<Box<dyn Future<Output = Result<TransactionInfo>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let rsp = ViewProtocolServiceClient::transaction_info_by_hash(
                &mut self2,
                tonic::Request::new(pb::TransactionInfoByHashRequest {
                    id: Some(id.into()),
                }),
            )
            .await?
            .into_inner()
            .tx_info
            .ok_or_else(|| anyhow::anyhow!("empty TransactionInfoByHashResponse message"))?;

            let tx_info = TransactionInfo {
                height: rsp
                    .height
                    .ok_or_else(|| anyhow::anyhow!("missing height"))?,
                id: rsp
                    .id
                    .ok_or_else(|| anyhow::anyhow!("missing id"))?
                    .try_into()?,
                transaction: rsp
                    .transaction
                    .ok_or_else(|| anyhow::anyhow!("missing transaction"))?
                    .try_into()?,
                perspective: rsp
                    .perspective
                    .ok_or_else(|| anyhow::anyhow!("missing perspective"))?
                    .try_into()?,
                view: rsp
                    .view
                    .ok_or_else(|| anyhow::anyhow!("missing view"))?
                    .try_into()?,
            };

            Ok(tx_info)
        }
        .boxed()
    }

    fn transaction_info(
        &mut self,
        start_height: Option<u64>,
        end_height: Option<u64>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<TransactionInfo>>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let rsp = self2.transaction_info(tonic::Request::new(pb::TransactionInfoRequest {
                start_height,
                end_height,
            }));
            let pb_txs: Vec<_> = rsp.await?.into_inner().try_collect().await?;

            pb_txs
                .into_iter()
                .map(|rsp| {
                    let tx_rsp = rsp
                        .tx_info
                        .ok_or_else(|| anyhow::anyhow!("empty TransactionInfoResponse message"))?;

                    let tx_info = TransactionInfo {
                        height: tx_rsp
                            .height
                            .ok_or_else(|| anyhow::anyhow!("missing height"))?,
                        transaction: tx_rsp
                            .transaction
                            .ok_or_else(|| {
                                anyhow::anyhow!("empty TransactionInfoResponse message")
                            })?
                            .try_into()?,
                        id: tx_rsp
                            .id
                            .ok_or_else(|| anyhow::anyhow!("missing id"))?
                            .try_into()?,
                        perspective: tx_rsp
                            .perspective
                            .ok_or_else(|| anyhow::anyhow!("missing perspective"))?
                            .try_into()?,
                        view: tx_rsp
                            .view
                            .ok_or_else(|| anyhow::anyhow!("missing view"))?
                            .try_into()?,
                    };

                    Ok(tx_info)
                })
                .collect()
        }
        .boxed()
    }

    fn broadcast_transaction(
        &mut self,
        transaction: Transaction,
        await_detection: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(penumbra_transaction::Id, u64)>> + Send + 'static>>
    {
        let mut self2 = self.clone();
        async move {
            let rsp = ViewProtocolServiceClient::broadcast_transaction(
                &mut self2,
                tonic::Request::new(pb::BroadcastTransactionRequest {
                    transaction: Some(transaction.into()),
                    await_detection,
                }),
            )
            .await?
            .into_inner();

            let id = rsp
                .id
                .ok_or_else(|| anyhow::anyhow!("response id is empty"))?
                .try_into()?;

            Ok((id, rsp.detection_height))
        }
        .boxed()
    }

    fn address_by_index(
        &mut self,
        address_index: AddressIndex,
    ) -> Pin<Box<dyn Future<Output = Result<Address>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let address = self2.address_by_index(tonic::Request::new(pb::AddressByIndexRequest {
                address_index: Some(address_index.into()),
                ..Default::default()
            }));
            let address = address
                .await?
                .into_inner()
                .address
                .ok_or_else(|| anyhow::anyhow!("No address available for this address index"))?
                .try_into()?;
            Ok(address)
        }
        .boxed()
    }

    fn witness_and_build(
        &mut self,
        transaction_plan: TransactionPlan,
        authorization_data: AuthorizationData,
    ) -> Pin<Box<dyn Future<Output = Result<Transaction>> + Send + 'static>> {
        let request = pb::WitnessAndBuildRequest {
            transaction_plan: Some(transaction_plan.into()),
            authorization_data: Some(authorization_data.into()),
        };
        let mut self2 = self.clone();
        async move {
            let rsp = self2.witness_and_build(tonic::Request::new(request));

            let tx = rsp
                .await?
                .into_inner()
                .transaction
                .ok_or_else(|| anyhow::anyhow!("empty WitnessAndBuildResponse message"))?
                .try_into()?;

            Ok(tx)
        }
        .boxed()
    }
}
