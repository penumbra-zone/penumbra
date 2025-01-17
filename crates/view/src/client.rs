use std::{collections::BTreeMap, future::Future, pin::Pin};

use anyhow::Result;
use futures::{FutureExt, Stream, StreamExt, TryStreamExt};
use pbjson_types::Any;
use penumbra_auction::auction::AuctionId;
use tonic::{codegen::Bytes, Streaming};
use tracing::instrument;

use penumbra_app::params::AppParameters;
use penumbra_asset::{
    asset::{self, Id, Metadata},
    ValueView,
};
use penumbra_dex::{
    lp::position::{self, Position},
    TradingPair,
};
use penumbra_fee::GasPrices;
use penumbra_keys::{keys::AddressIndex, Address};
use penumbra_num::Amount;
use penumbra_proto::view::v1::{
    self as pb, view_service_client::ViewServiceClient, BalancesResponse,
    BroadcastTransactionResponse, WitnessRequest,
};
use penumbra_sct::Nullifier;
use penumbra_shielded_pool::{fmd, note};
use penumbra_stake::IdentityKey;
use penumbra_transaction::{
    txhash::TransactionId, AuthorizationData, Transaction, TransactionPlan, WitnessData,
};

use crate::{SpendableNoteRecord, StatusStreamResponse, SwapRecord, TransactionInfo};

pub(crate) type BroadcastStatusStream = Pin<
    Box<dyn Future<Output = Result<Streaming<BroadcastTransactionResponse>, anyhow::Error>> + Send>,
>;

/// The view protocol is used by a view client, who wants to do some
/// transaction-related actions, to request data from a view service, which is
/// responsible for synchronizing and scanning the public chain state with one
/// or more full viewing keys.
///
/// This trait is a wrapper around the proto-generated [`ViewServiceClient`]
/// that serves two goals:
///
/// 1. It can use domain types rather than proto-generated types, avoiding conversions;
/// 2. It's easier to write as a trait bound than the `CustodyProtocolClient`,
///   which requires complex bounds on its inner type to
///   enforce that it is a tower `Service`.
#[allow(clippy::type_complexity)]
pub trait ViewClient {
    /// Query the auction state
    fn auctions(
        &mut self,
        account_filter: Option<AddressIndex>,
        include_inactive: bool,
        query_latest_state: bool,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<(
                            AuctionId,
                            SpendableNoteRecord,
                            u64,
                            Option<Any>,
                            Vec<Position>,
                        )>,
                    >,
                > + Send
                + 'static,
        >,
    >;

    /// Get the current status of chain sync.
    fn status(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<pb::StatusResponse>> + Send + 'static>>;

    /// Stream status updates on chain sync until it completes.
    fn status_stream(
        &mut self,
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

    /// Get a copy of the app parameters.
    fn app_params(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<AppParameters>> + Send + 'static>>;

    /// Get a copy of the gas prices.
    fn gas_prices(&mut self) -> Pin<Box<dyn Future<Output = Result<GasPrices>> + Send + 'static>>;

    /// Get a copy of the FMD parameters.
    fn fmd_parameters(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<fmd::Parameters>> + Send + 'static>>;

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
    fn balances(
        &mut self,
        address_index: AddressIndex,
        asset_id: Option<asset::Id>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<(Id, Amount)>>> + Send + 'static>>;

    /// Queries for a specific note by commitment, returning immediately if it is not found.
    fn note_by_commitment(
        &mut self,
        note_commitment: note::StateCommitment,
    ) -> Pin<Box<dyn Future<Output = Result<SpendableNoteRecord>> + Send + 'static>>;

    /// Queries for a specific swap by commitment, returning immediately if it is not found.
    fn swap_by_commitment(
        &mut self,
        swap_commitment: penumbra_tct::StateCommitment,
    ) -> Pin<Box<dyn Future<Output = Result<SwapRecord>> + Send + 'static>>;

    /// Queries for a specific nullifier's status, returning immediately if it is not found.
    fn nullifier_status(
        &mut self,
        nullifier: Nullifier,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'static>>;

    /// Waits for a specific nullifier to be detected, returning immediately if it is already
    /// present, but waiting otherwise.
    fn await_nullifier(
        &mut self,
        nullifier: Nullifier,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;

    /// Queries for a specific note by commitment, waiting until the note is detected if it is not found.
    ///
    /// This is useful for waiting for a note to be detected by the view service.
    fn await_note_by_commitment(
        &mut self,
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

    /// Queries for liquidity positions owned by the full viewing key.
    fn owned_position_ids(
        &mut self,
        position_state: Option<position::State>,
        trading_pair: Option<TradingPair>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<position::Id>>> + Send + 'static>>;

    /// Generates a full perspective for a selected transaction using a full viewing key
    fn transaction_info_by_hash(
        &mut self,
        id: TransactionId,
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
    ) -> BroadcastStatusStream;

    /// Return unspent notes, grouped by address index and then by asset id.
    #[instrument(skip(self))]
    fn unspent_notes_by_address_and_asset(
        &mut self,
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

    /// Return unspent notes, grouped by account ID (combining ephemeral addresses for the account) and then by asset id.
    #[instrument(skip(self))]
    fn unspent_notes_by_account_and_asset(
        &mut self,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<BTreeMap<u32, BTreeMap<asset::Id, Vec<SpendableNoteRecord>>>>,
                > + Send
                + 'static,
        >,
    > {
        let notes = self.notes(pb::NotesRequest {
            include_spent: false,
            ..Default::default()
        });
        async move {
            let notes = notes.await?;
            tracing::trace!(?notes);

            let mut notes_by_account_and_asset = BTreeMap::new();

            for note_record in notes {
                notes_by_account_and_asset
                    .entry(note_record.address_index.account)
                    .or_insert_with(BTreeMap::new)
                    .entry(note_record.note.asset_id())
                    .or_insert_with(Vec::new)
                    .push(note_record);
            }
            tracing::trace!(?notes_by_account_and_asset);

            Ok(notes_by_account_and_asset)
        }
        .boxed()
    }

    /// Return unspent notes, grouped by denom and then by address index.
    #[instrument(skip(self))]
    fn unspent_notes_by_asset_and_address(
        &mut self,
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

    /// Queries for the index of a provided address, returning `None` if not
    /// controlled by the view service's FVK.
    fn index_by_address(
        &mut self,
        address: Address,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AddressIndex>>> + Send + 'static>>;

    /// Queries for unclaimed Swaps.
    fn unclaimed_swaps(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<SwapRecord>>> + Send + 'static>>;
}

// We need to tell `async_trait` not to add a `Send` bound to the boxed
// futures it generates, because the underlying `CustodyProtocolClient` isn't `Sync`,
// but its `authorize` method takes `&mut self`. This would normally cause a huge
// amount of problems, because non-`Send` futures don't compose well, but as long
// as we're calling the method within an async block on a local mutable variable,
// it should be fine.
impl<T> ViewClient for ViewServiceClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody> + Clone + Send + 'static,
    T::ResponseBody: tonic::codegen::Body<Data = Bytes> + Send + 'static,
    T::Error: Into<tonic::codegen::StdError>,
    T::Future: Send + 'static,
    <T::ResponseBody as tonic::codegen::Body>::Error: Into<tonic::codegen::StdError> + Send,
{
    fn status(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<pb::StatusResponse>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let status = self2.status(tonic::Request::new(pb::StatusRequest {}));
            let status = status.await?.into_inner();
            Ok(status)
        }
        .boxed()
    }

    fn status_stream(
        &mut self,
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
            let stream = self2.status_stream(tonic::Request::new(pb::StatusStreamRequest {}));
            let stream = stream.await?.into_inner();

            Ok(stream
                .map_err(|e| anyhow::anyhow!("view service error: {}", e))
                .and_then(|msg| async move { StatusStreamResponse::try_from(msg) })
                .boxed())
        }
        .boxed()
    }

    fn app_params(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<AppParameters>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            // We have to manually invoke the method on the type, because it has the
            // same name as the one we're implementing.
            let rsp = ViewServiceClient::app_parameters(
                &mut self2,
                tonic::Request::new(pb::AppParametersRequest {}),
            );
            rsp.await?.into_inner().try_into()
        }
        .boxed()
    }

    fn gas_prices(&mut self) -> Pin<Box<dyn Future<Output = Result<GasPrices>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            // We have to manually invoke the method on the type, because it has the
            // same name as the one we're implementing.
            let rsp = ViewServiceClient::gas_prices(
                &mut self2,
                tonic::Request::new(pb::GasPricesRequest {}),
            );
            rsp.await?
                .into_inner()
                .gas_prices
                .ok_or_else(|| anyhow::anyhow!("empty GasPricesResponse message"))?
                .try_into()
        }
        .boxed()
    }

    fn fmd_parameters(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<fmd::Parameters>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let parameters = ViewServiceClient::fmd_parameters(
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
        note_commitment: note::StateCommitment,
    ) -> Pin<Box<dyn Future<Output = Result<SpendableNoteRecord>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let note_commitment_response = ViewServiceClient::note_by_commitment(
                &mut self2,
                tonic::Request::new(pb::NoteByCommitmentRequest {
                    note_commitment: Some(note_commitment.into()),
                    await_detection: false,
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

    fn balances(
        &mut self,
        address_index: AddressIndex,
        asset_id: Option<asset::Id>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<(Id, Amount)>>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let req = ViewServiceClient::balances(
                &mut self2,
                tonic::Request::new(pb::BalancesRequest {
                    account_filter: Some(address_index.into()),
                    asset_id_filter: asset_id.map(Into::into),
                }),
            );

            let balances: Vec<BalancesResponse> = req.await?.into_inner().try_collect().await?;

            balances
                .into_iter()
                .map(|rsp| {
                    let pb_value_view = rsp
                        .balance_view
                        .ok_or_else(|| anyhow::anyhow!("empty balance view"))?;

                    let value_view: ValueView = pb_value_view.try_into()?;
                    let id = value_view.asset_id();
                    let amount = value_view.value().amount;
                    Ok((id, amount))
                })
                .collect()
        }
        .boxed()
    }

    fn swap_by_commitment(
        &mut self,
        swap_commitment: penumbra_tct::StateCommitment,
    ) -> Pin<Box<dyn Future<Output = Result<SwapRecord>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let swap_commitment_response = ViewServiceClient::swap_by_commitment(
                &mut self2,
                tonic::Request::new(pb::SwapByCommitmentRequest {
                    swap_commitment: Some(swap_commitment.into()),
                    await_detection: false,
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
        note_commitment: note::StateCommitment,
    ) -> Pin<Box<dyn Future<Output = Result<SpendableNoteRecord>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let spendable_note = ViewServiceClient::note_by_commitment(
                &mut self2,
                tonic::Request::new(pb::NoteByCommitmentRequest {
                    note_commitment: Some(note_commitment.into()),
                    await_detection: true,
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
        nullifier: Nullifier,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let rsp = ViewServiceClient::nullifier_status(
                &mut self2,
                tonic::Request::new(pb::NullifierStatusRequest {
                    nullifier: Some(nullifier.into()),
                    await_detection: false,
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
        nullifier: Nullifier,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let rsp = ViewServiceClient::nullifier_status(
                &mut self2,
                tonic::Request::new(pb::NullifierStatusRequest {
                    nullifier: Some(nullifier.into()),
                    await_detection: true,
                }),
            );
            rsp.await?;
            Ok(())
        }
        .boxed()
    }

    fn witness(
        &mut self,
        plan: &TransactionPlan,
    ) -> Pin<Box<dyn Future<Output = Result<WitnessData>> + Send + 'static>> {
        let request = WitnessRequest {
            transaction_plan: Some(plan.clone().into()),
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
            let rsp = ViewServiceClient::assets(
                &mut self2,
                tonic::Request::new(pb::AssetsRequest {
                    ..Default::default()
                }),
            );

            let pb_assets: Vec<_> = rsp.await?.into_inner().try_collect().await?;

            let assets = pb_assets
                .into_iter()
                .map(Metadata::try_from)
                .collect::<anyhow::Result<Vec<Metadata>>>()?;

            Ok(assets.into_iter().collect())
        }
        .boxed()
    }

    fn owned_position_ids(
        &mut self,
        position_state: Option<position::State>,
        trading_pair: Option<TradingPair>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<position::Id>>> + Send + 'static>> {
        // should the return be streamed here? none of the other viewclient responses are, probably fine for now
        // but might be an issue eventually
        let mut self2 = self.clone();
        async move {
            // We have to manually invoke the method on the type, because it has the
            // same name as the one we're implementing.
            let rsp = ViewServiceClient::owned_position_ids(
                &mut self2,
                tonic::Request::new(pb::OwnedPositionIdsRequest {
                    trading_pair: trading_pair.map(TryInto::try_into).transpose()?,
                    position_state: position_state.map(TryInto::try_into).transpose()?,
                    subaccount: None,
                }),
            );

            let pb_position_ids: Vec<_> = rsp.await?.into_inner().try_collect().await?;

            let position_ids = pb_position_ids
                .into_iter()
                .map(|p| {
                    position::Id::try_from(p.position_id.ok_or_else(|| {
                        anyhow::anyhow!("empty OwnedPositionsIdsResponse message")
                    })?)
                })
                .collect::<anyhow::Result<Vec<position::Id>>>()?;

            Ok(position_ids)
        }
        .boxed()
    }

    fn transaction_info_by_hash(
        &mut self,
        id: TransactionId,
    ) -> Pin<Box<dyn Future<Output = Result<TransactionInfo>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let rsp = ViewServiceClient::transaction_info_by_hash(
                &mut self2,
                tonic::Request::new(pb::TransactionInfoByHashRequest {
                    id: Some(id.into()),
                }),
            )
            .await?
            .into_inner()
            .tx_info
            .ok_or_else(|| anyhow::anyhow!("empty TransactionInfoByHashResponse message"))?;

            // Check some assumptions about response structure
            if rsp.height == 0 {
                anyhow::bail!("missing height");
            }

            let tx_info = TransactionInfo {
                height: rsp.height,
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
            // Unpack optional block heights
            let start_h = if let Some(h) = start_height { h } else { 0 };

            let end_h = if let Some(h) = end_height { h } else { 0 };

            let rsp = self2.transaction_info(tonic::Request::new(pb::TransactionInfoRequest {
                start_height: start_h,
                end_height: end_h,
            }));
            let pb_txs: Vec<_> = rsp.await?.into_inner().try_collect().await?;

            pb_txs
                .into_iter()
                .map(|rsp| {
                    let tx_rsp = rsp
                        .tx_info
                        .ok_or_else(|| anyhow::anyhow!("empty TransactionInfoResponse message"))?;

                    // Confirm height is populated
                    if tx_rsp.height == 0 {
                        anyhow::bail!("missing height");
                    }

                    let tx_info = TransactionInfo {
                        height: tx_rsp.height,
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
    ) -> BroadcastStatusStream {
        let mut self2 = self.clone();
        async move {
            let rsp = ViewServiceClient::broadcast_transaction(
                &mut self2,
                tonic::Request::new(pb::BroadcastTransactionRequest {
                    transaction: Some(transaction.into()),
                    await_detection,
                }),
            )
            .await?
            .into_inner();

            Ok(rsp)
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

    fn index_by_address(
        &mut self,
        address: Address,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AddressIndex>>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let index = self2.index_by_address(tonic::Request::new(pb::IndexByAddressRequest {
                address: Some(address.into()),
            }));
            let index = index
                .await?
                .into_inner()
                .address_index
                .map(|index| index.try_into())
                .transpose()?;
            Ok(index)
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
            let mut rsp = self2
                .witness_and_build(tonic::Request::new(request))
                .await?
                .into_inner();

            while let Some(rsp) = rsp.try_next().await? {
                match rsp.status {
                    Some(status) => match status {
                        pb::witness_and_build_response::Status::BuildProgress(_) => {
                            // TODO: should update progress here
                        }
                        pb::witness_and_build_response::Status::Complete(c) => {
                            return c.transaction
                                .ok_or_else(|| {
                                    anyhow::anyhow!("WitnessAndBuildResponse complete status message missing transaction")
                                })?
                                .try_into();
                        }
                    },
                    None => {
                        // No status is unexpected behavior
                        return Err(anyhow::anyhow!(
                            "empty WitnessAndBuildResponse message"
                        ));
                    }
                }
            }

            Err(anyhow::anyhow!("should have received complete status or error"))
        }
            .boxed()
    }

    fn unclaimed_swaps(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<SwapRecord>>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            let swaps_response = ViewServiceClient::unclaimed_swaps(
                &mut self2,
                tonic::Request::new(pb::UnclaimedSwapsRequest {}),
            );
            let pb_swaps: Vec<_> = swaps_response.await?.into_inner().try_collect().await?;

            pb_swaps
                .into_iter()
                .map(|swap_rsp| {
                    let swap_record = swap_rsp
                        .swap
                        .ok_or_else(|| anyhow::anyhow!("empty UnclaimedSwapsResponse message"));

                    match swap_record {
                        Ok(swap) => swap.try_into(),
                        Err(e) => Err(e),
                    }
                })
                .collect()
        }
        .boxed()
    }

    fn auctions(
        &mut self,
        account_filter: Option<AddressIndex>,
        include_inactive: bool,
        query_latest_state: bool,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<(
                            AuctionId,
                            SpendableNoteRecord,
                            u64,
                            Option<Any>,
                            Vec<Position>,
                        )>,
                    >,
                > + Send
                + 'static,
        >,
    > {
        let mut client = self.clone();
        async move {
            let request = tonic::Request::new(pb::AuctionsRequest {
                account_filter: account_filter.map(Into::into),
                include_inactive,
                query_latest_state,
                auction_ids_filter: Vec::new(), // TODO: Support `auction_ids_filter`
            });

            let auctions: Vec<pb::AuctionsResponse> =
                ViewServiceClient::auctions(&mut client, request)
                    .await?
                    .into_inner()
                    .try_collect()
                    .await?;

            let resp: Vec<(
                AuctionId,
                SpendableNoteRecord,
                u64,
                Option<Any>,
                Vec<Position>,
            )> = auctions
                .into_iter()
                .map(|auction_rsp| {
                    let pb_id = auction_rsp
                        .id
                        .ok_or_else(|| anyhow::anyhow!("missing auction id"))?;
                    let auction_id: AuctionId = pb_id.try_into()?;
                    let snr: SpendableNoteRecord = auction_rsp
                        .note_record
                        .ok_or_else(|| anyhow::anyhow!("missing SNR from auction response"))?
                        .try_into()?;

                    let local_seq = auction_rsp.local_seq;

                    let auction = auction_rsp.auction;
                    let lps: Vec<Position> = auction_rsp
                        .positions
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<Vec<_>>>()?;

                    Ok::<
                        (
                            AuctionId,
                            SpendableNoteRecord,
                            u64, /* the local sequence number */
                            Option<Any>, /* the auction state if it was requested */
                            Vec<Position>, /* associated liquidity positions if we queried the latest state */
                        ),
                        anyhow::Error,
                    >((auction_id, snr, local_seq, auction, lps))
                })
                .filter_map(|res| res.ok()) // TODO: scrap this later.
                .collect();

            Ok(resp)
        }
        .boxed()
    }
}
