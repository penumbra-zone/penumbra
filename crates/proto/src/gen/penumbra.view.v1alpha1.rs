#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BroadcastTransactionRequest {
    /// The transaction to broadcast.
    #[prost(message, optional, tag = "1")]
    pub transaction: ::core::option::Option<
        super::super::core::transaction::v1alpha1::Transaction,
    >,
    /// If true, wait for the view service to detect the transaction during sync.
    #[prost(bool, tag = "2")]
    pub await_detection: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BroadcastTransactionResponse {
    /// The hash of the transaction that was broadcast.
    #[prost(message, optional, tag = "1")]
    pub id: ::core::option::Option<super::super::core::transaction::v1alpha1::Id>,
    /// The height in which the transaction was detected as included in the chain, if any.
    /// Will not be included unless await_detection was true.
    #[prost(uint64, tag = "2")]
    pub detection_height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionPlannerRequest {
    /// The expiry height for the requested TransactionPlan
    #[prost(uint64, tag = "1")]
    pub expiry_height: u64,
    /// The fee for the requested TransactionPlan, if any.
    #[prost(message, optional, tag = "2")]
    pub fee: ::core::option::Option<super::super::core::crypto::v1alpha1::Fee>,
    /// The memo for the requested TransactionPlan
    #[prost(string, tag = "3")]
    pub memo: ::prost::alloc::string::String,
    /// Identifies the account group to query.
    #[prost(message, optional, tag = "14")]
    pub account_group_id: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AccountGroupId,
    >,
    /// Request contents
    #[prost(message, repeated, tag = "20")]
    pub outputs: ::prost::alloc::vec::Vec<transaction_planner_request::Output>,
    #[prost(message, repeated, tag = "30")]
    pub swaps: ::prost::alloc::vec::Vec<transaction_planner_request::Swap>,
    #[prost(message, repeated, tag = "40")]
    pub delegations: ::prost::alloc::vec::Vec<transaction_planner_request::Delegate>,
    #[prost(message, repeated, tag = "50")]
    pub undelegations: ::prost::alloc::vec::Vec<transaction_planner_request::Undelegate>,
    #[prost(message, repeated, tag = "60")]
    pub ibc_actions: ::prost::alloc::vec::Vec<
        super::super::core::ibc::v1alpha1::IbcAction,
    >,
}
/// Nested message and enum types in `TransactionPlannerRequest`.
pub mod transaction_planner_request {
    /// Request message subtypes
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Output {
        #[prost(message, optional, tag = "1")]
        pub value: ::core::option::Option<
            super::super::super::core::crypto::v1alpha1::Value,
        >,
        #[prost(message, optional, tag = "2")]
        pub address: ::core::option::Option<
            super::super::super::core::crypto::v1alpha1::Address,
        >,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Swap {
        #[prost(message, optional, tag = "1")]
        pub value: ::core::option::Option<
            super::super::super::core::crypto::v1alpha1::Value,
        >,
        #[prost(message, optional, tag = "2")]
        pub target_asset: ::core::option::Option<
            super::super::super::core::crypto::v1alpha1::AssetId,
        >,
        #[prost(message, optional, tag = "3")]
        pub fee: ::core::option::Option<
            super::super::super::core::crypto::v1alpha1::Fee,
        >,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Delegate {
        #[prost(message, optional, tag = "1")]
        pub amount: ::core::option::Option<
            super::super::super::core::crypto::v1alpha1::Amount,
        >,
        #[prost(message, optional, tag = "3")]
        pub rate_data: ::core::option::Option<
            super::super::super::core::stake::v1alpha1::RateData,
        >,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Undelegate {
        #[prost(message, optional, tag = "1")]
        pub value: ::core::option::Option<
            super::super::super::core::crypto::v1alpha1::Value,
        >,
        #[prost(message, optional, tag = "2")]
        pub rate_data: ::core::option::Option<
            super::super::super::core::stake::v1alpha1::RateData,
        >,
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionPlannerResponse {
    #[prost(message, optional, tag = "1")]
    pub plan: ::core::option::Option<
        super::super::core::transaction::v1alpha1::TransactionPlan,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddressByIndexRequest {
    #[prost(message, optional, tag = "1")]
    pub address_index: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AddressIndex,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddressByIndexResponse {
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<super::super::core::crypto::v1alpha1::Address>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IndexByAddressRequest {
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<super::super::core::crypto::v1alpha1::Address>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IndexByAddressResponse {
    /// Will be absent if given an address not viewable by this viewing service
    #[prost(message, optional, tag = "1")]
    pub address_index: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AddressIndex,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EphemeralAddressRequest {
    #[prost(message, optional, tag = "1")]
    pub address_index: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AddressIndex,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EphemeralAddressResponse {
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<super::super::core::crypto::v1alpha1::Address>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BalanceByAddressRequest {
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<super::super::core::crypto::v1alpha1::Address>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BalanceByAddressResponse {
    #[prost(message, optional, tag = "1")]
    pub asset: ::core::option::Option<super::super::core::crypto::v1alpha1::AssetId>,
    #[prost(message, optional, tag = "2")]
    pub amount: ::core::option::Option<super::super::core::crypto::v1alpha1::Amount>,
}
/// Scaffolding for bearer-token authentication for the ViewService.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ViewAuthToken {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ViewAuthRequest {
    #[prost(message, optional, tag = "1")]
    pub fvk: ::core::option::Option<
        super::super::core::crypto::v1alpha1::FullViewingKey,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ViewAuthResponse {
    #[prost(message, optional, tag = "1")]
    pub token: ::core::option::Option<ViewAuthToken>,
}
/// Requests sync status of the view service.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusRequest {
    /// Identifies the account group to query.
    #[prost(message, optional, tag = "14")]
    pub account_group_id: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AccountGroupId,
    >,
}
/// Returns the status of the view service and whether it is synchronized with the chain state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusResponse {
    /// The height the view service has synchronized to so far
    #[prost(uint64, tag = "1")]
    pub sync_height: u64,
    /// Whether the view service is catching up with the chain state
    #[prost(bool, tag = "2")]
    pub catching_up: bool,
}
/// Requests streaming updates on the sync height until the view service is synchronized.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusStreamRequest {
    /// Identifies the account group to query.
    #[prost(message, optional, tag = "14")]
    pub account_group_id: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AccountGroupId,
    >,
}
/// A streaming sync status update
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusStreamResponse {
    #[prost(uint64, tag = "1")]
    pub latest_known_block_height: u64,
    #[prost(uint64, tag = "2")]
    pub sync_height: u64,
}
/// A query for notes known by the view service.
///
/// This message uses the fact that all proto fields are optional
/// to allow various filtering on the returned notes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NotesRequest {
    /// If set, return spent notes as well as unspent notes.
    #[prost(bool, tag = "2")]
    pub include_spent: bool,
    /// If set, only return notes with the specified asset id.
    #[prost(message, optional, tag = "3")]
    pub asset_id: ::core::option::Option<super::super::core::crypto::v1alpha1::AssetId>,
    /// If set, only return notes with the specified address incore.dex.v1alpha1.
    #[prost(message, optional, tag = "4")]
    pub address_index: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AddressIndex,
    >,
    /// If set, stop returning notes once the total exceeds this amount.
    ///
    /// Ignored if `asset_id` is unset or if `include_spent` is set.
    #[prost(message, optional, tag = "6")]
    pub amount_to_spend: ::core::option::Option<
        super::super::core::crypto::v1alpha1::Amount,
    >,
    /// Identifies the account group to query.
    #[prost(message, optional, tag = "14")]
    pub account_group_id: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AccountGroupId,
    >,
}
/// A query for notes to be used for voting on a proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NotesForVotingRequest {
    /// The starting height of the proposal.
    #[prost(uint64, tag = "1")]
    pub votable_at_height: u64,
    /// If set, only return notes with the specified asset id.
    #[prost(message, optional, tag = "3")]
    pub address_index: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AddressIndex,
    >,
    /// Identifies the account group to query.
    #[prost(message, optional, tag = "14")]
    pub account_group_id: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AccountGroupId,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WitnessRequest {
    /// The note commitments to obtain auth paths for.
    #[prost(message, repeated, tag = "2")]
    pub note_commitments: ::prost::alloc::vec::Vec<
        super::super::core::crypto::v1alpha1::StateCommitment,
    >,
    /// The transaction plan to witness
    #[prost(message, optional, tag = "3")]
    pub transaction_plan: ::core::option::Option<
        super::super::core::transaction::v1alpha1::TransactionPlan,
    >,
    /// Identifies the account group to query.
    #[prost(message, optional, tag = "14")]
    pub account_group_id: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AccountGroupId,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WitnessResponse {
    #[prost(message, optional, tag = "1")]
    pub witness_data: ::core::option::Option<
        super::super::core::transaction::v1alpha1::WitnessData,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WitnessAndBuildRequest {
    #[prost(message, optional, tag = "1")]
    pub transaction_plan: ::core::option::Option<
        super::super::core::transaction::v1alpha1::TransactionPlan,
    >,
    #[prost(message, optional, tag = "2")]
    pub authorization_data: ::core::option::Option<
        super::super::core::transaction::v1alpha1::AuthorizationData,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WitnessAndBuildResponse {
    #[prost(message, optional, tag = "1")]
    pub transaction: ::core::option::Option<
        super::super::core::transaction::v1alpha1::Transaction,
    >,
}
/// Requests all assets known to the view service.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetsRequest {
    /// If set to false (default), returns all assets, regardless of whether the rest of the fields of
    /// the request indicate a filter.
    #[prost(bool, tag = "1")]
    pub filtered: bool,
    /// Include these specific denominations in the response.
    #[prost(message, repeated, tag = "2")]
    pub include_specific_denominations: ::prost::alloc::vec::Vec<
        super::super::core::crypto::v1alpha1::Denom,
    >,
    /// Include all delegation tokens, to any validator, in the response.
    #[prost(bool, tag = "3")]
    pub include_delegation_tokens: bool,
    /// Include all unbonding tokens, from any validator, in the response.
    #[prost(bool, tag = "4")]
    pub include_unbonding_tokens: bool,
    /// Include all LP NFTs in the response.
    #[prost(bool, tag = "5")]
    pub include_lp_nfts: bool,
    /// Include all proposal NFTs in the response.
    #[prost(bool, tag = "6")]
    pub include_proposal_nfts: bool,
    /// Include all voting receipt tokens in the response.
    #[prost(bool, tag = "7")]
    pub include_voting_receipt_tokens: bool,
}
/// Requests all assets known to the view service.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetsResponse {
    #[prost(message, optional, tag = "2")]
    pub denom_metadata: ::core::option::Option<
        super::super::core::crypto::v1alpha1::DenomMetadata,
    >,
}
/// Requests the current chain parameters from the view service.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChainParametersRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChainParametersResponse {
    #[prost(message, optional, tag = "1")]
    pub parameters: ::core::option::Option<
        super::super::core::chain::v1alpha1::ChainParameters,
    >,
}
/// Requests the current FMD parameters from the view service.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FmdParametersRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FmdParametersResponse {
    #[prost(message, optional, tag = "1")]
    pub parameters: ::core::option::Option<
        super::super::core::chain::v1alpha1::FmdParameters,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NoteByCommitmentRequest {
    #[prost(message, optional, tag = "2")]
    pub note_commitment: ::core::option::Option<
        super::super::core::crypto::v1alpha1::StateCommitment,
    >,
    /// If set to true, waits to return until the requested note is detected.
    #[prost(bool, tag = "3")]
    pub await_detection: bool,
    /// Identifies the account group to query.
    #[prost(message, optional, tag = "14")]
    pub account_group_id: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AccountGroupId,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NoteByCommitmentResponse {
    #[prost(message, optional, tag = "1")]
    pub spendable_note: ::core::option::Option<SpendableNoteRecord>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapByCommitmentRequest {
    #[prost(message, optional, tag = "2")]
    pub swap_commitment: ::core::option::Option<
        super::super::core::crypto::v1alpha1::StateCommitment,
    >,
    /// If set to true, waits to return until the requested swap is detected.
    #[prost(bool, tag = "3")]
    pub await_detection: bool,
    /// Identifies the account group to query.
    #[prost(message, optional, tag = "14")]
    pub account_group_id: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AccountGroupId,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapByCommitmentResponse {
    #[prost(message, optional, tag = "1")]
    pub swap: ::core::option::Option<SwapRecord>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NullifierStatusRequest {
    #[prost(message, optional, tag = "2")]
    pub nullifier: ::core::option::Option<
        super::super::core::crypto::v1alpha1::Nullifier,
    >,
    #[prost(bool, tag = "3")]
    pub await_detection: bool,
    /// Identifies the account group to query.
    #[prost(message, optional, tag = "14")]
    pub account_group_id: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AccountGroupId,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NullifierStatusResponse {
    #[prost(bool, tag = "1")]
    pub spent: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfoByHashRequest {
    /// The transaction hash to query for.
    #[prost(message, optional, tag = "2")]
    pub id: ::core::option::Option<super::super::core::transaction::v1alpha1::Id>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfoRequest {
    /// If present, return only transactions after this height.
    #[prost(uint64, optional, tag = "1")]
    pub start_height: ::core::option::Option<u64>,
    /// If present, return only transactions before this height.
    #[prost(uint64, optional, tag = "2")]
    pub end_height: ::core::option::Option<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfo {
    /// The height the transaction was included in a block, if known.
    #[prost(uint64, optional, tag = "1")]
    pub height: ::core::option::Option<u64>,
    /// The hash of the transaction.
    #[prost(message, optional, tag = "2")]
    pub id: ::core::option::Option<super::super::core::transaction::v1alpha1::Id>,
    /// The transaction data itself.
    #[prost(message, optional, tag = "3")]
    pub transaction: ::core::option::Option<
        super::super::core::transaction::v1alpha1::Transaction,
    >,
    /// The transaction perspective, as seen by this view server.
    #[prost(message, optional, tag = "4")]
    pub perspective: ::core::option::Option<
        super::super::core::transaction::v1alpha1::TransactionPerspective,
    >,
    /// A precomputed transaction view of `transaction` from `perspective`, included for convenience of clients that don't have support for viewing transactions on their own.
    #[prost(message, optional, tag = "5")]
    pub view: ::core::option::Option<
        super::super::core::transaction::v1alpha1::TransactionView,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub tx_info: ::core::option::Option<TransactionInfo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfoByHashResponse {
    #[prost(message, optional, tag = "1")]
    pub tx_info: ::core::option::Option<TransactionInfo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NotesResponse {
    #[prost(message, optional, tag = "1")]
    pub note_record: ::core::option::Option<SpendableNoteRecord>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NotesForVotingResponse {
    #[prost(message, optional, tag = "1")]
    pub note_record: ::core::option::Option<SpendableNoteRecord>,
    #[prost(message, optional, tag = "2")]
    pub identity_key: ::core::option::Option<
        super::super::core::crypto::v1alpha1::IdentityKey,
    >,
}
/// A note plaintext with associated metadata about its status.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendableNoteRecord {
    /// The note commitment, identifying the note.
    #[prost(message, optional, tag = "1")]
    pub note_commitment: ::core::option::Option<
        super::super::core::crypto::v1alpha1::StateCommitment,
    >,
    /// The note plaintext itself.
    #[prost(message, optional, tag = "2")]
    pub note: ::core::option::Option<super::super::core::crypto::v1alpha1::Note>,
    /// A precomputed decryption of the note's address incore.dex.v1alpha1.
    #[prost(message, optional, tag = "3")]
    pub address_index: ::core::option::Option<
        super::super::core::crypto::v1alpha1::AddressIndex,
    >,
    /// The note's nullifier.
    #[prost(message, optional, tag = "4")]
    pub nullifier: ::core::option::Option<
        super::super::core::crypto::v1alpha1::Nullifier,
    >,
    /// The height at which the note was created.
    #[prost(uint64, tag = "5")]
    pub height_created: u64,
    /// Records whether the note was spent (and if so, at what height).
    #[prost(uint64, optional, tag = "6")]
    pub height_spent: ::core::option::Option<u64>,
    /// The note position.
    #[prost(uint64, tag = "7")]
    pub position: u64,
    /// The source of the note (a tx hash or otherwise)
    #[prost(message, optional, tag = "8")]
    pub source: ::core::option::Option<super::super::core::chain::v1alpha1::NoteSource>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapRecord {
    #[prost(message, optional, tag = "1")]
    pub swap_commitment: ::core::option::Option<
        super::super::core::crypto::v1alpha1::StateCommitment,
    >,
    #[prost(message, optional, tag = "2")]
    pub swap: ::core::option::Option<super::super::core::dex::v1alpha1::SwapPlaintext>,
    #[prost(uint64, tag = "3")]
    pub position: u64,
    #[prost(message, optional, tag = "4")]
    pub nullifier: ::core::option::Option<
        super::super::core::crypto::v1alpha1::Nullifier,
    >,
    #[prost(message, optional, tag = "5")]
    pub output_data: ::core::option::Option<
        super::super::core::dex::v1alpha1::BatchSwapOutputData,
    >,
    #[prost(uint64, optional, tag = "6")]
    pub height_claimed: ::core::option::Option<u64>,
    #[prost(message, optional, tag = "7")]
    pub source: ::core::option::Option<super::super::core::chain::v1alpha1::NoteSource>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OwnedPositionIdsRequest {
    /// If present, return only positions with this position state.
    #[prost(message, optional, tag = "1")]
    pub position_state: ::core::option::Option<
        super::super::core::dex::v1alpha1::PositionState,
    >,
    /// If present, return only positions for this trading pair.
    #[prost(message, optional, tag = "2")]
    pub trading_pair: ::core::option::Option<
        super::super::core::dex::v1alpha1::TradingPair,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OwnedPositionIdsResponse {
    #[prost(message, repeated, tag = "1")]
    pub position_ids: ::prost::alloc::vec::Vec<
        super::super::core::dex::v1alpha1::PositionId,
    >,
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod view_protocol_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// The view protocol is used by a view client, who wants to do some
    /// transaction-related actions, to request data from a view service, which is
    /// responsible for synchronizing and scanning the public chain state with one or
    /// more full viewing keys.
    ///
    /// View protocol requests optionally include the account group ID, used to
    /// identify which set of data to query.
    #[derive(Debug, Clone)]
    pub struct ViewProtocolServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ViewProtocolServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ViewProtocolServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ViewProtocolServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            ViewProtocolServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Get current status of chain sync
        pub async fn status(
            &mut self,
            request: impl tonic::IntoRequest<super::StatusRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/Status",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Stream sync status updates until the view service has caught up with the core.chain.v1alpha1.
        pub async fn status_stream(
            &mut self,
            request: impl tonic::IntoRequest<super::StatusStreamRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::StatusStreamResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/StatusStream",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        /// Queries for notes that have been accepted by the core.chain.v1alpha1.
        pub async fn notes(
            &mut self,
            request: impl tonic::IntoRequest<super::NotesRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::NotesResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/Notes",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn notes_for_voting(
            &mut self,
            request: impl tonic::IntoRequest<super::NotesForVotingRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::NotesForVotingResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/NotesForVoting",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        /// Returns authentication paths for the given note commitments.
        ///
        /// This method takes a batch of input commitments, rather than just one, so
        /// that the client can get a consistent set of authentication paths to a
        /// common root.  (Otherwise, if a client made multiple requests, the wallet
        /// service could have advanced the state commitment tree state between queries).
        pub async fn witness(
            &mut self,
            request: impl tonic::IntoRequest<super::WitnessRequest>,
        ) -> Result<tonic::Response<super::WitnessResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/Witness",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn witness_and_build(
            &mut self,
            request: impl tonic::IntoRequest<super::WitnessAndBuildRequest>,
        ) -> Result<tonic::Response<super::WitnessAndBuildResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/WitnessAndBuild",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Queries for assets.
        pub async fn assets(
            &mut self,
            request: impl tonic::IntoRequest<super::AssetsRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::AssetsResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/Assets",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        /// Query for the current chain parameters.
        pub async fn chain_parameters(
            &mut self,
            request: impl tonic::IntoRequest<super::ChainParametersRequest>,
        ) -> Result<tonic::Response<super::ChainParametersResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/ChainParameters",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Query for the current FMD parameters.
        pub async fn fmd_parameters(
            &mut self,
            request: impl tonic::IntoRequest<super::FmdParametersRequest>,
        ) -> Result<tonic::Response<super::FmdParametersResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/FMDParameters",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Query for an address given an address index
        pub async fn address_by_index(
            &mut self,
            request: impl tonic::IntoRequest<super::AddressByIndexRequest>,
        ) -> Result<tonic::Response<super::AddressByIndexResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/AddressByIndex",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Query for an address given an address index
        pub async fn index_by_address(
            &mut self,
            request: impl tonic::IntoRequest<super::IndexByAddressRequest>,
        ) -> Result<tonic::Response<super::IndexByAddressResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/IndexByAddress",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Query for an ephemeral address
        pub async fn ephemeral_address(
            &mut self,
            request: impl tonic::IntoRequest<super::EphemeralAddressRequest>,
        ) -> Result<tonic::Response<super::EphemeralAddressResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/EphemeralAddress",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Query for balance of a given address
        pub async fn balance_by_address(
            &mut self,
            request: impl tonic::IntoRequest<super::BalanceByAddressRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::BalanceByAddressResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/BalanceByAddress",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        /// Query for a note by its note commitment, optionally waiting until the note is detected.
        pub async fn note_by_commitment(
            &mut self,
            request: impl tonic::IntoRequest<super::NoteByCommitmentRequest>,
        ) -> Result<tonic::Response<super::NoteByCommitmentResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/NoteByCommitment",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Query for a swap by its swap commitment, optionally waiting until the swap is detected.
        pub async fn swap_by_commitment(
            &mut self,
            request: impl tonic::IntoRequest<super::SwapByCommitmentRequest>,
        ) -> Result<tonic::Response<super::SwapByCommitmentResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/SwapByCommitment",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Query for whether a nullifier has been spent, optionally waiting until it is spent.
        pub async fn nullifier_status(
            &mut self,
            request: impl tonic::IntoRequest<super::NullifierStatusRequest>,
        ) -> Result<tonic::Response<super::NullifierStatusResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/NullifierStatus",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Query for a given transaction by its hash.
        pub async fn transaction_info_by_hash(
            &mut self,
            request: impl tonic::IntoRequest<super::TransactionInfoByHashRequest>,
        ) -> Result<
            tonic::Response<super::TransactionInfoByHashResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/TransactionInfoByHash",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Query for the full transactions in the given range of blocks.
        pub async fn transaction_info(
            &mut self,
            request: impl tonic::IntoRequest<super::TransactionInfoRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::TransactionInfoResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/TransactionInfo",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        /// Query for a transaction plan
        pub async fn transaction_planner(
            &mut self,
            request: impl tonic::IntoRequest<super::TransactionPlannerRequest>,
        ) -> Result<tonic::Response<super::TransactionPlannerResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/TransactionPlanner",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Broadcast a transaction to the network, optionally waiting for full confirmation.
        pub async fn broadcast_transaction(
            &mut self,
            request: impl tonic::IntoRequest<super::BroadcastTransactionRequest>,
        ) -> Result<
            tonic::Response<super::BroadcastTransactionResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/BroadcastTransaction",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Query for owned position IDs for the given trading pair and in the given position state.
        pub async fn owned_position_ids(
            &mut self,
            request: impl tonic::IntoRequest<super::OwnedPositionIdsRequest>,
        ) -> Result<tonic::Response<super::OwnedPositionIdsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewProtocolService/OwnedPositionIds",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod view_auth_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct ViewAuthServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ViewAuthServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ViewAuthServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ViewAuthServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            ViewAuthServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn view_auth(
            &mut self,
            request: impl tonic::IntoRequest<super::ViewAuthRequest>,
        ) -> Result<tonic::Response<super::ViewAuthResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.view.v1alpha1.ViewAuthService/ViewAuth",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod view_protocol_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ViewProtocolServiceServer.
    #[async_trait]
    pub trait ViewProtocolService: Send + Sync + 'static {
        /// Get current status of chain sync
        async fn status(
            &self,
            request: tonic::Request<super::StatusRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        /// Server streaming response type for the StatusStream method.
        type StatusStreamStream: futures_core::Stream<
                Item = Result<super::StatusStreamResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Stream sync status updates until the view service has caught up with the core.chain.v1alpha1.
        async fn status_stream(
            &self,
            request: tonic::Request<super::StatusStreamRequest>,
        ) -> Result<tonic::Response<Self::StatusStreamStream>, tonic::Status>;
        /// Server streaming response type for the Notes method.
        type NotesStream: futures_core::Stream<
                Item = Result<super::NotesResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Queries for notes that have been accepted by the core.chain.v1alpha1.
        async fn notes(
            &self,
            request: tonic::Request<super::NotesRequest>,
        ) -> Result<tonic::Response<Self::NotesStream>, tonic::Status>;
        /// Server streaming response type for the NotesForVoting method.
        type NotesForVotingStream: futures_core::Stream<
                Item = Result<super::NotesForVotingResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn notes_for_voting(
            &self,
            request: tonic::Request<super::NotesForVotingRequest>,
        ) -> Result<tonic::Response<Self::NotesForVotingStream>, tonic::Status>;
        /// Returns authentication paths for the given note commitments.
        ///
        /// This method takes a batch of input commitments, rather than just one, so
        /// that the client can get a consistent set of authentication paths to a
        /// common root.  (Otherwise, if a client made multiple requests, the wallet
        /// service could have advanced the state commitment tree state between queries).
        async fn witness(
            &self,
            request: tonic::Request<super::WitnessRequest>,
        ) -> Result<tonic::Response<super::WitnessResponse>, tonic::Status>;
        async fn witness_and_build(
            &self,
            request: tonic::Request<super::WitnessAndBuildRequest>,
        ) -> Result<tonic::Response<super::WitnessAndBuildResponse>, tonic::Status>;
        /// Server streaming response type for the Assets method.
        type AssetsStream: futures_core::Stream<
                Item = Result<super::AssetsResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Queries for assets.
        async fn assets(
            &self,
            request: tonic::Request<super::AssetsRequest>,
        ) -> Result<tonic::Response<Self::AssetsStream>, tonic::Status>;
        /// Query for the current chain parameters.
        async fn chain_parameters(
            &self,
            request: tonic::Request<super::ChainParametersRequest>,
        ) -> Result<tonic::Response<super::ChainParametersResponse>, tonic::Status>;
        /// Query for the current FMD parameters.
        async fn fmd_parameters(
            &self,
            request: tonic::Request<super::FmdParametersRequest>,
        ) -> Result<tonic::Response<super::FmdParametersResponse>, tonic::Status>;
        /// Query for an address given an address index
        async fn address_by_index(
            &self,
            request: tonic::Request<super::AddressByIndexRequest>,
        ) -> Result<tonic::Response<super::AddressByIndexResponse>, tonic::Status>;
        /// Query for an address given an address index
        async fn index_by_address(
            &self,
            request: tonic::Request<super::IndexByAddressRequest>,
        ) -> Result<tonic::Response<super::IndexByAddressResponse>, tonic::Status>;
        /// Query for an ephemeral address
        async fn ephemeral_address(
            &self,
            request: tonic::Request<super::EphemeralAddressRequest>,
        ) -> Result<tonic::Response<super::EphemeralAddressResponse>, tonic::Status>;
        /// Server streaming response type for the BalanceByAddress method.
        type BalanceByAddressStream: futures_core::Stream<
                Item = Result<super::BalanceByAddressResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Query for balance of a given address
        async fn balance_by_address(
            &self,
            request: tonic::Request<super::BalanceByAddressRequest>,
        ) -> Result<tonic::Response<Self::BalanceByAddressStream>, tonic::Status>;
        /// Query for a note by its note commitment, optionally waiting until the note is detected.
        async fn note_by_commitment(
            &self,
            request: tonic::Request<super::NoteByCommitmentRequest>,
        ) -> Result<tonic::Response<super::NoteByCommitmentResponse>, tonic::Status>;
        /// Query for a swap by its swap commitment, optionally waiting until the swap is detected.
        async fn swap_by_commitment(
            &self,
            request: tonic::Request<super::SwapByCommitmentRequest>,
        ) -> Result<tonic::Response<super::SwapByCommitmentResponse>, tonic::Status>;
        /// Query for whether a nullifier has been spent, optionally waiting until it is spent.
        async fn nullifier_status(
            &self,
            request: tonic::Request<super::NullifierStatusRequest>,
        ) -> Result<tonic::Response<super::NullifierStatusResponse>, tonic::Status>;
        /// Query for a given transaction by its hash.
        async fn transaction_info_by_hash(
            &self,
            request: tonic::Request<super::TransactionInfoByHashRequest>,
        ) -> Result<
            tonic::Response<super::TransactionInfoByHashResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the TransactionInfo method.
        type TransactionInfoStream: futures_core::Stream<
                Item = Result<super::TransactionInfoResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Query for the full transactions in the given range of blocks.
        async fn transaction_info(
            &self,
            request: tonic::Request<super::TransactionInfoRequest>,
        ) -> Result<tonic::Response<Self::TransactionInfoStream>, tonic::Status>;
        /// Query for a transaction plan
        async fn transaction_planner(
            &self,
            request: tonic::Request<super::TransactionPlannerRequest>,
        ) -> Result<tonic::Response<super::TransactionPlannerResponse>, tonic::Status>;
        /// Broadcast a transaction to the network, optionally waiting for full confirmation.
        async fn broadcast_transaction(
            &self,
            request: tonic::Request<super::BroadcastTransactionRequest>,
        ) -> Result<tonic::Response<super::BroadcastTransactionResponse>, tonic::Status>;
        /// Query for owned position IDs for the given trading pair and in the given position state.
        async fn owned_position_ids(
            &self,
            request: tonic::Request<super::OwnedPositionIdsRequest>,
        ) -> Result<tonic::Response<super::OwnedPositionIdsResponse>, tonic::Status>;
    }
    /// The view protocol is used by a view client, who wants to do some
    /// transaction-related actions, to request data from a view service, which is
    /// responsible for synchronizing and scanning the public chain state with one or
    /// more full viewing keys.
    ///
    /// View protocol requests optionally include the account group ID, used to
    /// identify which set of data to query.
    #[derive(Debug)]
    pub struct ViewProtocolServiceServer<T: ViewProtocolService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ViewProtocolService> ViewProtocolServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ViewProtocolServiceServer<T>
    where
        T: ViewProtocolService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/penumbra.view.v1alpha1.ViewProtocolService/Status" => {
                    #[allow(non_camel_case_types)]
                    struct StatusSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::StatusRequest>
                    for StatusSvc<T> {
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::StatusRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).status(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StatusSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/StatusStream" => {
                    #[allow(non_camel_case_types)]
                    struct StatusStreamSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::ServerStreamingService<super::StatusStreamRequest>
                    for StatusStreamSvc<T> {
                        type Response = super::StatusStreamResponse;
                        type ResponseStream = T::StatusStreamStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::StatusStreamRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).status_stream(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StatusStreamSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/Notes" => {
                    #[allow(non_camel_case_types)]
                    struct NotesSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::ServerStreamingService<super::NotesRequest>
                    for NotesSvc<T> {
                        type Response = super::NotesResponse;
                        type ResponseStream = T::NotesStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NotesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).notes(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NotesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/NotesForVoting" => {
                    #[allow(non_camel_case_types)]
                    struct NotesForVotingSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::ServerStreamingService<super::NotesForVotingRequest>
                    for NotesForVotingSvc<T> {
                        type Response = super::NotesForVotingResponse;
                        type ResponseStream = T::NotesForVotingStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NotesForVotingRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).notes_for_voting(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NotesForVotingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/Witness" => {
                    #[allow(non_camel_case_types)]
                    struct WitnessSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::WitnessRequest>
                    for WitnessSvc<T> {
                        type Response = super::WitnessResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::WitnessRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).witness(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = WitnessSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/WitnessAndBuild" => {
                    #[allow(non_camel_case_types)]
                    struct WitnessAndBuildSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::WitnessAndBuildRequest>
                    for WitnessAndBuildSvc<T> {
                        type Response = super::WitnessAndBuildResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::WitnessAndBuildRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).witness_and_build(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = WitnessAndBuildSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/Assets" => {
                    #[allow(non_camel_case_types)]
                    struct AssetsSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::ServerStreamingService<super::AssetsRequest>
                    for AssetsSvc<T> {
                        type Response = super::AssetsResponse;
                        type ResponseStream = T::AssetsStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AssetsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).assets(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AssetsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/ChainParameters" => {
                    #[allow(non_camel_case_types)]
                    struct ChainParametersSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::ChainParametersRequest>
                    for ChainParametersSvc<T> {
                        type Response = super::ChainParametersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ChainParametersRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).chain_parameters(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChainParametersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/FMDParameters" => {
                    #[allow(non_camel_case_types)]
                    struct FMDParametersSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::FmdParametersRequest>
                    for FMDParametersSvc<T> {
                        type Response = super::FmdParametersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FmdParametersRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).fmd_parameters(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FMDParametersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/AddressByIndex" => {
                    #[allow(non_camel_case_types)]
                    struct AddressByIndexSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::AddressByIndexRequest>
                    for AddressByIndexSvc<T> {
                        type Response = super::AddressByIndexResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AddressByIndexRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).address_by_index(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AddressByIndexSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/IndexByAddress" => {
                    #[allow(non_camel_case_types)]
                    struct IndexByAddressSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::IndexByAddressRequest>
                    for IndexByAddressSvc<T> {
                        type Response = super::IndexByAddressResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::IndexByAddressRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).index_by_address(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = IndexByAddressSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/EphemeralAddress" => {
                    #[allow(non_camel_case_types)]
                    struct EphemeralAddressSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::EphemeralAddressRequest>
                    for EphemeralAddressSvc<T> {
                        type Response = super::EphemeralAddressResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::EphemeralAddressRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).ephemeral_address(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EphemeralAddressSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/BalanceByAddress" => {
                    #[allow(non_camel_case_types)]
                    struct BalanceByAddressSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::ServerStreamingService<
                        super::BalanceByAddressRequest,
                    > for BalanceByAddressSvc<T> {
                        type Response = super::BalanceByAddressResponse;
                        type ResponseStream = T::BalanceByAddressStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BalanceByAddressRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).balance_by_address(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BalanceByAddressSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/NoteByCommitment" => {
                    #[allow(non_camel_case_types)]
                    struct NoteByCommitmentSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::NoteByCommitmentRequest>
                    for NoteByCommitmentSvc<T> {
                        type Response = super::NoteByCommitmentResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NoteByCommitmentRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).note_by_commitment(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NoteByCommitmentSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/SwapByCommitment" => {
                    #[allow(non_camel_case_types)]
                    struct SwapByCommitmentSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::SwapByCommitmentRequest>
                    for SwapByCommitmentSvc<T> {
                        type Response = super::SwapByCommitmentResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SwapByCommitmentRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).swap_by_commitment(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SwapByCommitmentSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/NullifierStatus" => {
                    #[allow(non_camel_case_types)]
                    struct NullifierStatusSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::NullifierStatusRequest>
                    for NullifierStatusSvc<T> {
                        type Response = super::NullifierStatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NullifierStatusRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).nullifier_status(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NullifierStatusSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/TransactionInfoByHash" => {
                    #[allow(non_camel_case_types)]
                    struct TransactionInfoByHashSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::TransactionInfoByHashRequest>
                    for TransactionInfoByHashSvc<T> {
                        type Response = super::TransactionInfoByHashResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TransactionInfoByHashRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).transaction_info_by_hash(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TransactionInfoByHashSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/TransactionInfo" => {
                    #[allow(non_camel_case_types)]
                    struct TransactionInfoSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::ServerStreamingService<
                        super::TransactionInfoRequest,
                    > for TransactionInfoSvc<T> {
                        type Response = super::TransactionInfoResponse;
                        type ResponseStream = T::TransactionInfoStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TransactionInfoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).transaction_info(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TransactionInfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/TransactionPlanner" => {
                    #[allow(non_camel_case_types)]
                    struct TransactionPlannerSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::TransactionPlannerRequest>
                    for TransactionPlannerSvc<T> {
                        type Response = super::TransactionPlannerResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TransactionPlannerRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).transaction_planner(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TransactionPlannerSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/BroadcastTransaction" => {
                    #[allow(non_camel_case_types)]
                    struct BroadcastTransactionSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::BroadcastTransactionRequest>
                    for BroadcastTransactionSvc<T> {
                        type Response = super::BroadcastTransactionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BroadcastTransactionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).broadcast_transaction(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BroadcastTransactionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/OwnedPositionIds" => {
                    #[allow(non_camel_case_types)]
                    struct OwnedPositionIdsSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::OwnedPositionIdsRequest>
                    for OwnedPositionIdsSvc<T> {
                        type Response = super::OwnedPositionIdsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::OwnedPositionIdsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).owned_position_ids(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = OwnedPositionIdsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: ViewProtocolService> Clone for ViewProtocolServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: ViewProtocolService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ViewProtocolService> tonic::server::NamedService
    for ViewProtocolServiceServer<T> {
        const NAME: &'static str = "penumbra.view.v1alpha1.ViewProtocolService";
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod view_auth_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ViewAuthServiceServer.
    #[async_trait]
    pub trait ViewAuthService: Send + Sync + 'static {
        async fn view_auth(
            &self,
            request: tonic::Request<super::ViewAuthRequest>,
        ) -> Result<tonic::Response<super::ViewAuthResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ViewAuthServiceServer<T: ViewAuthService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ViewAuthService> ViewAuthServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ViewAuthServiceServer<T>
    where
        T: ViewAuthService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/penumbra.view.v1alpha1.ViewAuthService/ViewAuth" => {
                    #[allow(non_camel_case_types)]
                    struct ViewAuthSvc<T: ViewAuthService>(pub Arc<T>);
                    impl<
                        T: ViewAuthService,
                    > tonic::server::UnaryService<super::ViewAuthRequest>
                    for ViewAuthSvc<T> {
                        type Response = super::ViewAuthResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ViewAuthRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).view_auth(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ViewAuthSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: ViewAuthService> Clone for ViewAuthServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: ViewAuthService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ViewAuthService> tonic::server::NamedService for ViewAuthServiceServer<T> {
        const NAME: &'static str = "penumbra.view.v1alpha1.ViewAuthService";
    }
}
