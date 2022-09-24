use std::{collections::BTreeMap, pin::Pin};

use anyhow::Result;
use futures::{Stream, StreamExt, TryStreamExt};
use penumbra_chain::params::{ChainParameters, FmdParameters};
use penumbra_crypto::keys::AccountID;
use penumbra_crypto::{asset, keys::AddressIndex, note, Asset, Nullifier};
use penumbra_proto::view::v1alpha1::{self as pb, view_protocol_client::ViewProtocolClient};
use penumbra_transaction::{Transaction, WitnessData};
use tonic::async_trait;
use tonic::codegen::Bytes;
use tracing::instrument;

use crate::{QuarantinedNoteRecord, SpendableNoteRecord, StatusStreamResponse};

/// The view protocol is used by a view client, who wants to do some
/// transaction-related actions, to request data from a view service, which is
/// responsible for synchronizing and scanning the public chain state with one
/// or more full viewing keys.
///
/// This trait is a wrapper around the proto-generated [`ViewProtocolClient`]
/// that serves two goals:
///
/// 1. It can use domain types rather than proto-generated types, avoiding conversions;
/// 2. It's easier to write as a trait bound than the `CustodyProtocolClient`,
///   which requires complex bounds on its inner type to
///   enforce that it is a tower `Service`.
#[async_trait(?Send)]
pub trait ViewClient {
    /// Get the current status of chain sync.
    async fn status(&mut self, account_id: AccountID) -> Result<pb::StatusResponse>;

    /// Stream status updates on chain sync until it completes.
    async fn status_stream(
        &mut self,
        account_id: AccountID,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StatusStreamResponse>> + Send + 'static>>>;

    /// Get a copy of the chain parameters.
    async fn chain_params(&mut self) -> Result<ChainParameters>;

    /// Get a copy of the FMD parameters.
    async fn fmd_parameters(&mut self) -> Result<FmdParameters>;

    /// Queries for notes.
    async fn notes(&mut self, request: pb::NotesRequest) -> Result<Vec<SpendableNoteRecord>>;

    /// Queries for quarantined notes.
    async fn quarantined_notes(
        &mut self,
        request: pb::QuarantinedNotesRequest,
    ) -> Result<Vec<QuarantinedNoteRecord>>;

    /// Queries for a specific note by commitment, returning immediately if it is not found.
    async fn note_by_commitment(
        &mut self,
        account_id: AccountID,
        note_commitment: note::Commitment,
    ) -> Result<SpendableNoteRecord>;

    /// Queries for a specific nullifier's status, returning immediately if it is not found.
    async fn nullifier_status(
        &mut self,
        account_id: AccountID,
        nullifier: Nullifier,
    ) -> Result<bool>;

    /// Waits for a specific nullifier to be detected, returning immediately if it is already
    /// present, but waiting otherwise.
    async fn await_nullifier(&mut self, account_id: AccountID, nullifier: Nullifier) -> Result<()>;

    /// Queries for a specific note by commitment, waiting until the note is detected if it is not found.
    ///
    /// This is useful for waiting for a note to be detected by the view service.
    async fn await_note_by_commitment(
        &mut self,
        account_id: AccountID,
        note_commitment: note::Commitment,
    ) -> Result<SpendableNoteRecord>;

    /// Returns authentication paths for the given note commitments.
    ///
    /// This method takes a batch of input commitments, rather than just one, so
    /// that the client can get a consistent set of authentication paths to a
    /// common root.  (Otherwise, if a client made multiple requests, the wallet
    /// service could have advanced the note commitment tree state between queries).
    async fn witness(&mut self, request: pb::WitnessRequest) -> Result<WitnessData>;

    /// Queries for all known assets.
    async fn assets(&mut self) -> Result<asset::Cache>;

    /// Queries for transaction hashes in a range of block heights
    async fn transaction_hashes(
        &mut self,
        start_height: Option<u64>,
        end_height: Option<u64>,
    ) -> Result<Vec<(u64, Vec<u8>)>>;

    /// Queries for transactions in a range of block heights
    async fn transactions(
        &mut self,
        start_height: Option<u64>,
        end_height: Option<u64>,
    ) -> Result<Vec<(u64, Transaction)>>;

    /// Return unspent notes, grouped by address index and then by asset id.
    #[instrument(skip(self, account_id))]
    async fn unspent_notes_by_address_and_asset(
        &mut self,
        account_id: AccountID,
    ) -> Result<BTreeMap<AddressIndex, BTreeMap<asset::Id, Vec<SpendableNoteRecord>>>> {
        let notes = self
            .notes(pb::NotesRequest {
                account_id: Some(account_id.into()),
                include_spent: false,
                ..Default::default()
            })
            .await?;
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

    /// Return unspent notes, grouped by denom and then by address index.
    #[instrument(skip(self, account_id))]
    async fn unspent_notes_by_asset_and_address(
        &mut self,
        account_id: AccountID,
    ) -> Result<BTreeMap<asset::Id, BTreeMap<AddressIndex, Vec<SpendableNoteRecord>>>> {
        let notes = self
            .notes(pb::NotesRequest {
                account_id: Some(account_id.into()),
                include_spent: false,
                ..Default::default()
            })
            .await?;
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

    /// Return quarantined notes, grouped by address index and then by asset id.
    #[instrument(skip(self, account_id))]
    async fn quarantined_notes_by_address_and_asset(
        &mut self,
        account_id: AccountID,
    ) -> Result<BTreeMap<AddressIndex, BTreeMap<asset::Id, Vec<QuarantinedNoteRecord>>>> {
        let notes = self
            .quarantined_notes(pb::QuarantinedNotesRequest {
                account_id: Some(account_id.into()),
            })
            .await?;
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

    /// Return quarantined notes, grouped by denom and then by address index.
    #[instrument(skip(self, account_id))]
    async fn quarantined_notes_by_asset_and_address(
        &mut self,
        account_id: AccountID,
    ) -> Result<BTreeMap<asset::Id, BTreeMap<AddressIndex, Vec<QuarantinedNoteRecord>>>> {
        let notes = self
            .quarantined_notes(pb::QuarantinedNotesRequest {
                account_id: Some(account_id.into()),
            })
            .await?;
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
}

// We need to tell `async_trait` not to add a `Send` bound to the boxed
// futures it generates, because the underlying `CustodyProtocolClient` isn't `Sync`,
// but its `authorize` method takes `&mut self`. This would normally cause a huge
// amount of problems, because non-`Send` futures don't compose well, but as long
// as we're calling the method within an async block on a local mutable variable,
// it should be fine.
#[async_trait(?Send)]
impl<T> ViewClient for ViewProtocolClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::ResponseBody: tonic::codegen::Body<Data = Bytes> + Send + 'static,
    T::Error: Into<tonic::codegen::StdError>,
    <T::ResponseBody as tonic::codegen::Body>::Error: Into<tonic::codegen::StdError> + Send,
{
    async fn status(&mut self, account_id: AccountID) -> Result<pb::StatusResponse> {
        let status = self
            .status(tonic::Request::new(pb::StatusRequest {
                account_id: Some(account_id.into()),
            }))
            .await?
            .into_inner();

        Ok(status)
    }

    async fn status_stream(
        &mut self,
        account_id: AccountID,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StatusStreamResponse>> + Send + 'static>>> {
        let stream = self
            .status_stream(tonic::Request::new(pb::StatusStreamRequest {
                account_id: Some(account_id.into()),
            }))
            .await?
            .into_inner();

        Ok(stream
            .map_err(|e| anyhow::anyhow!("view service error: {}", e))
            .and_then(|msg| async move { StatusStreamResponse::try_from(msg) })
            .boxed())
    }

    async fn chain_params(&mut self) -> Result<ChainParameters> {
        // We have to manually invoke the method on the type, because it has the
        // same name as the one we're implementing.
        let params = ViewProtocolClient::chain_parameters(
            self,
            tonic::Request::new(pb::ChainParamsRequest {}),
        )
        .await?
        .into_inner()
        .try_into()?;

        Ok(params)
    }

    async fn fmd_parameters(&mut self) -> Result<FmdParameters> {
        let params = ViewProtocolClient::fmd_parameters(
            self,
            tonic::Request::new(pb::FmdParametersRequest {}),
        )
        .await?
        .into_inner()
        .try_into()?;

        Ok(params)
    }

    async fn notes(&mut self, request: pb::NotesRequest) -> Result<Vec<SpendableNoteRecord>> {
        let pb_notes: Vec<_> = self
            .notes(tonic::Request::new(request))
            .await?
            .into_inner()
            .try_collect()
            .await?;

        pb_notes.into_iter().map(TryInto::try_into).collect()
    }

    async fn quarantined_notes(
        &mut self,
        request: pb::QuarantinedNotesRequest,
    ) -> Result<Vec<QuarantinedNoteRecord>> {
        let pb_notes: Vec<_> = self
            .quarantined_notes(tonic::Request::new(request))
            .await?
            .into_inner()
            .try_collect()
            .await?;

        pb_notes.into_iter().map(TryInto::try_into).collect()
    }

    async fn note_by_commitment(
        &mut self,
        account_id: AccountID,
        note_commitment: note::Commitment,
    ) -> Result<SpendableNoteRecord> {
        ViewProtocolClient::note_by_commitment(
            self,
            tonic::Request::new(pb::NoteByCommitmentRequest {
                account_id: Some(account_id.into()),
                note_commitment: Some(note_commitment.into()),
                await_detection: false,
            }),
        )
        .await?
        .into_inner()
        .try_into()
    }

    /// Queries for a specific note by commitment, waiting until the note is detected if it is not found.
    ///
    /// This is useful for waiting for a note to be detected by the view service.
    async fn await_note_by_commitment(
        &mut self,
        account_id: AccountID,
        note_commitment: note::Commitment,
    ) -> Result<SpendableNoteRecord> {
        ViewProtocolClient::note_by_commitment(
            self,
            tonic::Request::new(pb::NoteByCommitmentRequest {
                account_id: Some(account_id.into()),
                note_commitment: Some(note_commitment.into()),
                await_detection: true,
            }),
        )
        .await?
        .into_inner()
        .try_into()
    }

    /// Queries for a specific nullifier's status, returning immediately if it is not found.
    async fn nullifier_status(
        &mut self,
        account_id: AccountID,
        nullifier: Nullifier,
    ) -> Result<bool> {
        Ok(ViewProtocolClient::nullifier_status(
            self,
            tonic::Request::new(pb::NullifierStatusRequest {
                account_id: Some(account_id.into()),
                nullifier: Some(nullifier.into()),
                await_detection: false,
            }),
        )
        .await?
        .into_inner()
        .spent)
    }

    /// Waits for a specific nullifier to be detected, returning immediately if it is already
    /// present, but waiting otherwise.
    async fn await_nullifier(&mut self, account_id: AccountID, nullifier: Nullifier) -> Result<()> {
        ViewProtocolClient::nullifier_status(
            self,
            tonic::Request::new(pb::NullifierStatusRequest {
                account_id: Some(account_id.into()),
                nullifier: Some(nullifier.into()),
                await_detection: true,
            }),
        )
        .await?;

        Ok(())
    }

    async fn witness(&mut self, request: pb::WitnessRequest) -> Result<WitnessData> {
        let witness_data: WitnessData = self
            .witness(tonic::Request::new(request))
            .await?
            .into_inner()
            .try_into()?;

        Ok(witness_data)
    }

    async fn assets(&mut self) -> Result<asset::Cache> {
        // We have to manually invoke the method on the type, because it has the
        // same name as the one we're implementing.
        let pb_assets: Vec<_> =
            ViewProtocolClient::assets(self, tonic::Request::new(pb::AssetRequest {}))
                .await?
                .into_inner()
                .try_collect()
                .await?;

        let assets = pb_assets
            .into_iter()
            .map(Asset::try_from)
            .collect::<Result<Vec<Asset>, anyhow::Error>>()?;

        Ok(assets.into_iter().map(|asset| asset.denom).collect())
    }

    async fn transaction_hashes(
        &mut self,
        start_height: Option<u64>,
        end_height: Option<u64>,
    ) -> Result<Vec<(u64, Vec<u8>)>> {
        let pb_txs: Vec<_> = self
            .transaction_hashes(tonic::Request::new(pb::TransactionsRequest {
                start_height,
                end_height,
            }))
            .await?
            .into_inner()
            .try_collect()
            .await?;

        let txs = pb_txs
            .into_iter()
            .map(|x| (x.block_height, x.tx_hash))
            .collect();

        Ok(txs)
    }

    async fn transactions(
        &mut self,
        start_height: Option<u64>,
        end_height: Option<u64>,
    ) -> Result<Vec<(u64, Transaction)>> {
        let pb_txs: Vec<_> = self
            .transactions(tonic::Request::new(pb::TransactionsRequest {
                start_height,
                end_height,
            }))
            .await?
            .into_inner()
            .try_collect()
            .await?;

        let txs = pb_txs
            .into_iter()
            .map(|x| (x.block_height, x.tx.unwrap().try_into().unwrap()))
            .collect();

        Ok(txs)
    }
}
