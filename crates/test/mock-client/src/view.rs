use {
    crate::MockClient,
    anyhow::Result,
    futures::Stream,
    penumbra_app::params::AppParameters,
    penumbra_asset::asset::{self, Id},
    penumbra_dex::{
        lp::position::{self},
        TradingPair,
    },
    penumbra_fee::GasPrices,
    penumbra_keys::{keys::AddressIndex, Address},
    penumbra_num::Amount,
    penumbra_proto::view::v1::{self as pb},
    penumbra_sct::Nullifier,
    penumbra_shielded_pool::{fmd, note},
    penumbra_stake::IdentityKey,
    penumbra_transaction::{
        txhash::TransactionId, AuthorizationData, Transaction, TransactionPlan, WitnessData,
    },
    penumbra_view::{
        BroadcastStatusStream, SpendableNoteRecord, StatusStreamResponse, SwapRecord,
        TransactionInfo, ViewClient,
    },
    std::{future::Future, pin::Pin},
};

impl ViewClient for MockClient {
    /// Get the current status of chain sync.
    fn status(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<pb::StatusResponse>> + Send + 'static>> {
        todo!()
    }

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
    > {
        todo!()
    }

    /// Get a copy of the app parameters.
    fn app_params(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<AppParameters>> + Send + 'static>> {
        todo!()
    }

    /// Get a copy of the gas prices.
    fn gas_prices(&mut self) -> Pin<Box<dyn Future<Output = Result<GasPrices>> + Send + 'static>> {
        todo!()
    }

    /// Get a copy of the FMD parameters.
    fn fmd_parameters(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<fmd::Parameters>> + Send + 'static>> {
        todo!()
    }

    /// Queries for notes.
    fn notes(
        &mut self,
        request: pb::NotesRequest,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<SpendableNoteRecord>>> + Send + 'static>> {
        todo!()
    }

    /// Queries for notes for voting.
    fn notes_for_voting(
        &mut self,
        request: pb::NotesForVotingRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<Vec<(SpendableNoteRecord, IdentityKey)>>> + Send + 'static>,
    > {
        todo!()
    }

    /// Queries for account balance by address
    fn balances(
        &mut self,
        address_index: AddressIndex,
        asset_id: Option<asset::Id>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<(Id, Amount)>>> + Send + 'static>> {
        todo!()
    }

    /// Queries for a specific note by commitment, returning immediately if it is not found.
    fn note_by_commitment(
        &mut self,
        note_commitment: note::StateCommitment,
    ) -> Pin<Box<dyn Future<Output = Result<SpendableNoteRecord>> + Send + 'static>> {
        todo!()
    }

    /// Queries for a specific swap by commitment, returning immediately if it is not found.
    fn swap_by_commitment(
        &mut self,
        swap_commitment: penumbra_tct::StateCommitment,
    ) -> Pin<Box<dyn Future<Output = Result<SwapRecord>> + Send + 'static>> {
        todo!()
    }

    /// Queries for a specific nullifier's status, returning immediately if it is not found.
    fn nullifier_status(
        &mut self,
        nullifier: Nullifier,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'static>> {
        todo!()
    }

    /// Waits for a specific nullifier to be detected, returning immediately if it is already
    /// present, but waiting otherwise.
    fn await_nullifier(
        &mut self,
        nullifier: Nullifier,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>> {
        todo!()
    }

    /// Queries for a specific note by commitment, waiting until the note is detected if it is not found.
    ///
    /// This is useful for waiting for a note to be detected by the view service.
    fn await_note_by_commitment(
        &mut self,
        note_commitment: note::StateCommitment,
    ) -> Pin<Box<dyn Future<Output = Result<SpendableNoteRecord>> + Send + 'static>> {
        todo!()
    }

    /// Returns authentication paths for the given note commitments.
    ///
    /// This method takes a batch of input commitments, rather than just one, so
    /// that the client can get a consistent set of authentication paths to a
    /// common root.  (Otherwise, if a client made multiple requests, the wallet
    /// service could have advanced the state commitment tree state between queries).
    fn witness(
        &mut self,
        plan: &TransactionPlan,
    ) -> Pin<Box<dyn Future<Output = Result<WitnessData>> + Send + 'static>> {
        todo!()
    }

    /// Returns a transaction built from the provided TransactionPlan and AuthorizationData
    fn witness_and_build(
        &mut self,
        plan: TransactionPlan,
        auth_data: AuthorizationData,
    ) -> Pin<Box<dyn Future<Output = Result<Transaction>> + Send + 'static>> {
        todo!()
    }

    /// Queries for all known assets.
    fn assets(&mut self) -> Pin<Box<dyn Future<Output = Result<asset::Cache>> + Send + 'static>> {
        todo!()
    }

    /// Queries for liquidity positions owned by the full viewing key.
    fn owned_position_ids(
        &mut self,
        position_state: Option<position::State>,
        trading_pair: Option<TradingPair>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<position::Id>>> + Send + 'static>> {
        todo!()
    }

    /// Generates a full perspective for a selected transaction using a full viewing key
    fn transaction_info_by_hash(
        &mut self,
        id: TransactionId,
    ) -> Pin<Box<dyn Future<Output = Result<TransactionInfo>> + Send + 'static>> {
        todo!()
    }

    /// Queries for transactions in a range of block heights
    fn transaction_info(
        &mut self,
        start_height: Option<u64>,
        end_height: Option<u64>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<TransactionInfo>>> + Send + 'static>> {
        todo!()
    }

    fn broadcast_transaction(
        &mut self,
        transaction: Transaction,
        await_detection: bool,
    ) -> BroadcastStatusStream {
        todo!()
    }

    fn address_by_index(
        &mut self,
        address_index: AddressIndex,
    ) -> Pin<Box<dyn Future<Output = Result<Address>> + Send + 'static>> {
        todo!()
    }

    /// Queries for unclaimed Swaps.
    fn unclaimed_swaps(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<SwapRecord>>> + Send + 'static>> {
        todo!()
    }
}
