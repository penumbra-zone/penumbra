#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthorizeAndBuildRequest {
    /// The transaction plan to authorize and build.
    #[prost(message, optional, tag = "1")]
    pub transaction_plan: ::core::option::Option<
        super::super::core::transaction::v1alpha1::TransactionPlan,
    >,
    /// The authorization data to use to authorize the transaction plan.
    #[prost(message, optional, tag = "2")]
    pub authorization_data: ::core::option::Option<
        super::super::core::transaction::v1alpha1::AuthorizationData,
    >,
}
impl ::prost::Name for AuthorizeAndBuildRequest {
    const NAME: &'static str = "AuthorizeAndBuildRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthorizeAndBuildResponse {
    /// The transaction that was built.
    #[prost(message, optional, tag = "1")]
    pub transaction: ::core::option::Option<
        super::super::core::transaction::v1alpha1::Transaction,
    >,
}
impl ::prost::Name for AuthorizeAndBuildResponse {
    const NAME: &'static str = "AuthorizeAndBuildResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
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
impl ::prost::Name for BroadcastTransactionRequest {
    const NAME: &'static str = "BroadcastTransactionRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BroadcastTransactionResponse {
    /// The hash of the transaction that was broadcast.
    #[prost(message, optional, tag = "1")]
    pub id: ::core::option::Option<super::super::core::txhash::v1alpha1::TransactionId>,
    /// The height in which the transaction was detected as included in the chain, if any.
    /// Will not be included unless await_detection was true.
    #[prost(uint64, tag = "2")]
    pub detection_height: u64,
}
impl ::prost::Name for BroadcastTransactionResponse {
    const NAME: &'static str = "BroadcastTransactionResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionPlannerRequest {
    /// The expiry height for the requested TransactionPlan
    #[prost(uint64, tag = "1")]
    pub expiry_height: u64,
    /// The fee for the requested TransactionPlan, if any.
    #[prost(message, optional, tag = "2")]
    pub fee: ::core::option::Option<super::super::core::component::fee::v1alpha1::Fee>,
    /// The memo for the requested TransactionPlan.
    /// The memo must be unspecified unless `outputs` is nonempty.
    #[prost(message, optional, tag = "3")]
    pub memo: ::core::option::Option<
        super::super::core::transaction::v1alpha1::MemoPlaintext,
    >,
    /// If present, only spends funds from the given account.
    #[prost(message, optional, tag = "4")]
    pub source: ::core::option::Option<super::super::core::keys::v1alpha1::AddressIndex>,
    /// Request contents
    #[prost(message, repeated, tag = "20")]
    pub outputs: ::prost::alloc::vec::Vec<transaction_planner_request::Output>,
    #[prost(message, repeated, tag = "30")]
    pub swaps: ::prost::alloc::vec::Vec<transaction_planner_request::Swap>,
    #[prost(message, repeated, tag = "31")]
    pub swap_claims: ::prost::alloc::vec::Vec<transaction_planner_request::SwapClaim>,
    #[prost(message, repeated, tag = "40")]
    pub delegations: ::prost::alloc::vec::Vec<transaction_planner_request::Delegate>,
    #[prost(message, repeated, tag = "50")]
    pub undelegations: ::prost::alloc::vec::Vec<transaction_planner_request::Undelegate>,
    #[prost(message, repeated, tag = "60")]
    pub ibc_relay_actions: ::prost::alloc::vec::Vec<
        super::super::core::component::ibc::v1alpha1::IbcRelay,
    >,
    #[prost(message, repeated, tag = "61")]
    pub ics20_withdrawals: ::prost::alloc::vec::Vec<
        super::super::core::component::ibc::v1alpha1::Ics20Withdrawal,
    >,
    #[prost(message, repeated, tag = "70")]
    pub position_opens: ::prost::alloc::vec::Vec<
        transaction_planner_request::PositionOpen,
    >,
    #[prost(message, repeated, tag = "71")]
    pub position_closes: ::prost::alloc::vec::Vec<
        transaction_planner_request::PositionClose,
    >,
    #[prost(message, repeated, tag = "72")]
    pub position_withdraws: ::prost::alloc::vec::Vec<
        transaction_planner_request::PositionWithdraw,
    >,
}
/// Nested message and enum types in `TransactionPlannerRequest`.
pub mod transaction_planner_request {
    /// Request message subtypes
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Output {
        /// The amount and denomination in which the Output is issued.
        #[prost(message, optional, tag = "1")]
        pub value: ::core::option::Option<
            super::super::super::core::asset::v1alpha1::Value,
        >,
        /// The address to which Output will be sent.
        #[prost(message, optional, tag = "2")]
        pub address: ::core::option::Option<
            super::super::super::core::keys::v1alpha1::Address,
        >,
    }
    impl ::prost::Name for Output {
        const NAME: &'static str = "Output";
        const PACKAGE: &'static str = "penumbra.view.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.view.v1alpha1.TransactionPlannerRequest.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Swap {
        /// The input amount and denomination to be traded in the Swap.
        #[prost(message, optional, tag = "1")]
        pub value: ::core::option::Option<
            super::super::super::core::asset::v1alpha1::Value,
        >,
        /// The denomination to be received as a Output of the Swap.
        #[prost(message, optional, tag = "2")]
        pub target_asset: ::core::option::Option<
            super::super::super::core::asset::v1alpha1::AssetId,
        >,
        /// The pre-paid fee to be paid for claiming the Swap outputs.
        #[prost(message, optional, tag = "3")]
        pub fee: ::core::option::Option<
            super::super::super::core::component::fee::v1alpha1::Fee,
        >,
        /// The address to which swap claim output will be sent.
        #[prost(message, optional, tag = "4")]
        pub claim_address: ::core::option::Option<
            super::super::super::core::keys::v1alpha1::Address,
        >,
    }
    impl ::prost::Name for Swap {
        const NAME: &'static str = "Swap";
        const PACKAGE: &'static str = "penumbra.view.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.view.v1alpha1.TransactionPlannerRequest.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SwapClaim {
        /// SwapCommitment to identify the Swap to be claimed.
        /// Use the commitment from the Swap message:
        /// penumbra.core.component.dex.v1alpha1.Swap.body.payload.commitment.
        #[prost(message, optional, tag = "1")]
        pub swap_commitment: ::core::option::Option<
            super::super::super::crypto::tct::v1alpha1::StateCommitment,
        >,
    }
    impl ::prost::Name for SwapClaim {
        const NAME: &'static str = "SwapClaim";
        const PACKAGE: &'static str = "penumbra.view.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.view.v1alpha1.TransactionPlannerRequest.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Delegate {
        #[prost(message, optional, tag = "1")]
        pub amount: ::core::option::Option<
            super::super::super::core::num::v1alpha1::Amount,
        >,
        #[prost(message, optional, tag = "3")]
        pub rate_data: ::core::option::Option<
            super::super::super::core::component::stake::v1alpha1::RateData,
        >,
    }
    impl ::prost::Name for Delegate {
        const NAME: &'static str = "Delegate";
        const PACKAGE: &'static str = "penumbra.view.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.view.v1alpha1.TransactionPlannerRequest.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Undelegate {
        #[prost(message, optional, tag = "1")]
        pub value: ::core::option::Option<
            super::super::super::core::asset::v1alpha1::Value,
        >,
        #[prost(message, optional, tag = "2")]
        pub rate_data: ::core::option::Option<
            super::super::super::core::component::stake::v1alpha1::RateData,
        >,
    }
    impl ::prost::Name for Undelegate {
        const NAME: &'static str = "Undelegate";
        const PACKAGE: &'static str = "penumbra.view.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.view.v1alpha1.TransactionPlannerRequest.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PositionOpen {
        /// Contains the data defining the position, sufficient to compute its `PositionId`.
        ///
        /// Positions are immutable, so the `PositionData` (and hence the `PositionId`)
        /// are unchanged over the entire lifetime of the position.
        #[prost(message, optional, tag = "1")]
        pub position: ::core::option::Option<
            super::super::super::core::component::dex::v1alpha1::Position,
        >,
    }
    impl ::prost::Name for PositionOpen {
        const NAME: &'static str = "PositionOpen";
        const PACKAGE: &'static str = "penumbra.view.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.view.v1alpha1.TransactionPlannerRequest.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PositionClose {
        /// The position to close.
        #[prost(message, optional, tag = "1")]
        pub position_id: ::core::option::Option<
            super::super::super::core::component::dex::v1alpha1::PositionId,
        >,
    }
    impl ::prost::Name for PositionClose {
        const NAME: &'static str = "PositionClose";
        const PACKAGE: &'static str = "penumbra.view.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.view.v1alpha1.TransactionPlannerRequest.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PositionWithdraw {
        /// The position to withdraw.
        #[prost(message, optional, tag = "1")]
        pub position_id: ::core::option::Option<
            super::super::super::core::component::dex::v1alpha1::PositionId,
        >,
        /// The position's final reserves.
        #[prost(message, optional, tag = "2")]
        pub reserves: ::core::option::Option<
            super::super::super::core::component::dex::v1alpha1::Reserves,
        >,
        /// The trading pair of the position.
        #[prost(message, optional, tag = "3")]
        pub trading_pair: ::core::option::Option<
            super::super::super::core::component::dex::v1alpha1::TradingPair,
        >,
    }
    impl ::prost::Name for PositionWithdraw {
        const NAME: &'static str = "PositionWithdraw";
        const PACKAGE: &'static str = "penumbra.view.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.view.v1alpha1.TransactionPlannerRequest.{}", Self::NAME
            )
        }
    }
}
impl ::prost::Name for TransactionPlannerRequest {
    const NAME: &'static str = "TransactionPlannerRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
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
impl ::prost::Name for TransactionPlannerResponse {
    const NAME: &'static str = "TransactionPlannerResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddressByIndexRequest {
    #[prost(message, optional, tag = "1")]
    pub address_index: ::core::option::Option<
        super::super::core::keys::v1alpha1::AddressIndex,
    >,
}
impl ::prost::Name for AddressByIndexRequest {
    const NAME: &'static str = "AddressByIndexRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddressByIndexResponse {
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<super::super::core::keys::v1alpha1::Address>,
}
impl ::prost::Name for AddressByIndexResponse {
    const NAME: &'static str = "AddressByIndexResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WalletIdRequest {}
impl ::prost::Name for WalletIdRequest {
    const NAME: &'static str = "WalletIdRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WalletIdResponse {
    #[prost(message, optional, tag = "1")]
    pub wallet_id: ::core::option::Option<super::super::core::keys::v1alpha1::WalletId>,
}
impl ::prost::Name for WalletIdResponse {
    const NAME: &'static str = "WalletIdResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IndexByAddressRequest {
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<super::super::core::keys::v1alpha1::Address>,
}
impl ::prost::Name for IndexByAddressRequest {
    const NAME: &'static str = "IndexByAddressRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IndexByAddressResponse {
    /// Will be absent if given an address not viewable by this viewing service
    #[prost(message, optional, tag = "1")]
    pub address_index: ::core::option::Option<
        super::super::core::keys::v1alpha1::AddressIndex,
    >,
}
impl ::prost::Name for IndexByAddressResponse {
    const NAME: &'static str = "IndexByAddressResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EphemeralAddressRequest {
    #[prost(message, optional, tag = "1")]
    pub address_index: ::core::option::Option<
        super::super::core::keys::v1alpha1::AddressIndex,
    >,
    #[prost(bool, tag = "2")]
    pub display_confirm: bool,
}
impl ::prost::Name for EphemeralAddressRequest {
    const NAME: &'static str = "EphemeralAddressRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EphemeralAddressResponse {
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<super::super::core::keys::v1alpha1::Address>,
}
impl ::prost::Name for EphemeralAddressResponse {
    const NAME: &'static str = "EphemeralAddressResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BalancesRequest {
    /// If present, filter balances to only include the account specified by the `AddressIndex`.
    #[prost(message, optional, tag = "1")]
    pub account_filter: ::core::option::Option<
        super::super::core::keys::v1alpha1::AddressIndex,
    >,
    /// If present, filter balances to only include the specified asset ID.
    #[prost(message, optional, tag = "2")]
    pub asset_id_filter: ::core::option::Option<
        super::super::core::asset::v1alpha1::AssetId,
    >,
}
impl ::prost::Name for BalancesRequest {
    const NAME: &'static str = "BalancesRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BalancesResponse {
    #[prost(message, optional, tag = "1")]
    pub account: ::core::option::Option<
        super::super::core::keys::v1alpha1::AddressIndex,
    >,
    #[prost(message, optional, tag = "2")]
    pub balance: ::core::option::Option<super::super::core::asset::v1alpha1::Value>,
}
impl ::prost::Name for BalancesResponse {
    const NAME: &'static str = "BalancesResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
/// Requests sync status of the view service.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusRequest {}
impl ::prost::Name for StatusRequest {
    const NAME: &'static str = "StatusRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
/// Returns the status of the view service and whether it is synchronized with the chain state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusResponse {
    /// The height the view service has synchronized to so far when doing a full linear sync
    #[prost(uint64, tag = "1")]
    pub full_sync_height: u64,
    /// The height the view service has synchronized to so far when doing a partial sync
    #[prost(uint64, tag = "2")]
    pub partial_sync_height: u64,
    /// Whether the view service is catching up with the chain state
    #[prost(bool, tag = "3")]
    pub catching_up: bool,
}
impl ::prost::Name for StatusResponse {
    const NAME: &'static str = "StatusResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
/// Requests streaming updates on the sync height until the view service is synchronized.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusStreamRequest {}
impl ::prost::Name for StatusStreamRequest {
    const NAME: &'static str = "StatusStreamRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
/// A streaming sync status update
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusStreamResponse {
    /// The latest known block height
    #[prost(uint64, tag = "1")]
    pub latest_known_block_height: u64,
    /// The height the view service has synchronized to so far when doing a full linear sync
    #[prost(uint64, tag = "2")]
    pub full_sync_height: u64,
    /// The height the view service has synchronized to so far when doing a partial sync
    #[prost(uint64, tag = "3")]
    pub partial_sync_height: u64,
}
impl ::prost::Name for StatusStreamResponse {
    const NAME: &'static str = "StatusStreamResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
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
    pub asset_id: ::core::option::Option<super::super::core::asset::v1alpha1::AssetId>,
    /// If set, only return notes with the specified address incore.component.dex.v1alpha1.
    #[prost(message, optional, tag = "4")]
    pub address_index: ::core::option::Option<
        super::super::core::keys::v1alpha1::AddressIndex,
    >,
    /// If set, stop returning notes once the total exceeds this amount.
    ///
    /// Ignored if `asset_id` is unset or if `include_spent` is set.
    #[prost(message, optional, tag = "6")]
    pub amount_to_spend: ::core::option::Option<
        super::super::core::num::v1alpha1::Amount,
    >,
}
impl ::prost::Name for NotesRequest {
    const NAME: &'static str = "NotesRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
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
        super::super::core::keys::v1alpha1::AddressIndex,
    >,
}
impl ::prost::Name for NotesForVotingRequest {
    const NAME: &'static str = "NotesForVotingRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WitnessRequest {
    /// The note commitments to obtain auth paths for.
    #[prost(message, repeated, tag = "2")]
    pub note_commitments: ::prost::alloc::vec::Vec<
        super::super::crypto::tct::v1alpha1::StateCommitment,
    >,
    /// The transaction plan to witness
    #[prost(message, optional, tag = "3")]
    pub transaction_plan: ::core::option::Option<
        super::super::core::transaction::v1alpha1::TransactionPlan,
    >,
}
impl ::prost::Name for WitnessRequest {
    const NAME: &'static str = "WitnessRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WitnessResponse {
    #[prost(message, optional, tag = "1")]
    pub witness_data: ::core::option::Option<
        super::super::core::transaction::v1alpha1::WitnessData,
    >,
}
impl ::prost::Name for WitnessResponse {
    const NAME: &'static str = "WitnessResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
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
impl ::prost::Name for WitnessAndBuildRequest {
    const NAME: &'static str = "WitnessAndBuildRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WitnessAndBuildResponse {
    #[prost(message, optional, tag = "1")]
    pub transaction: ::core::option::Option<
        super::super::core::transaction::v1alpha1::Transaction,
    >,
}
impl ::prost::Name for WitnessAndBuildResponse {
    const NAME: &'static str = "WitnessAndBuildResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
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
        super::super::core::asset::v1alpha1::Denom,
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
impl ::prost::Name for AssetsRequest {
    const NAME: &'static str = "AssetsRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
/// Requests all assets known to the view service.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetsResponse {
    #[prost(message, optional, tag = "2")]
    pub denom_metadata: ::core::option::Option<
        super::super::core::asset::v1alpha1::DenomMetadata,
    >,
}
impl ::prost::Name for AssetsResponse {
    const NAME: &'static str = "AssetsResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
/// Requests the current app parameters from the view service.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppParametersRequest {}
impl ::prost::Name for AppParametersRequest {
    const NAME: &'static str = "AppParametersRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppParametersResponse {
    #[prost(message, optional, tag = "1")]
    pub parameters: ::core::option::Option<
        super::super::core::app::v1alpha1::AppParameters,
    >,
}
impl ::prost::Name for AppParametersResponse {
    const NAME: &'static str = "AppParametersResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
/// Requests the current gas prices from the view service.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GasPricesRequest {}
impl ::prost::Name for GasPricesRequest {
    const NAME: &'static str = "GasPricesRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GasPricesResponse {
    #[prost(message, optional, tag = "1")]
    pub gas_prices: ::core::option::Option<
        super::super::core::component::fee::v1alpha1::GasPrices,
    >,
}
impl ::prost::Name for GasPricesResponse {
    const NAME: &'static str = "GasPricesResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
/// Requests the current FMD parameters from the view service.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FmdParametersRequest {}
impl ::prost::Name for FmdParametersRequest {
    const NAME: &'static str = "FMDParametersRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FmdParametersResponse {
    #[prost(message, optional, tag = "1")]
    pub parameters: ::core::option::Option<
        super::super::core::component::chain::v1alpha1::FmdParameters,
    >,
}
impl ::prost::Name for FmdParametersResponse {
    const NAME: &'static str = "FMDParametersResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NoteByCommitmentRequest {
    #[prost(message, optional, tag = "2")]
    pub note_commitment: ::core::option::Option<
        super::super::crypto::tct::v1alpha1::StateCommitment,
    >,
    /// If set to true, waits to return until the requested note is detected.
    #[prost(bool, tag = "3")]
    pub await_detection: bool,
}
impl ::prost::Name for NoteByCommitmentRequest {
    const NAME: &'static str = "NoteByCommitmentRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NoteByCommitmentResponse {
    #[prost(message, optional, tag = "1")]
    pub spendable_note: ::core::option::Option<SpendableNoteRecord>,
}
impl ::prost::Name for NoteByCommitmentResponse {
    const NAME: &'static str = "NoteByCommitmentResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapByCommitmentRequest {
    #[prost(message, optional, tag = "2")]
    pub swap_commitment: ::core::option::Option<
        super::super::crypto::tct::v1alpha1::StateCommitment,
    >,
    /// If set to true, waits to return until the requested swap is detected.
    #[prost(bool, tag = "3")]
    pub await_detection: bool,
}
impl ::prost::Name for SwapByCommitmentRequest {
    const NAME: &'static str = "SwapByCommitmentRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapByCommitmentResponse {
    #[prost(message, optional, tag = "1")]
    pub swap: ::core::option::Option<SwapRecord>,
}
impl ::prost::Name for SwapByCommitmentResponse {
    const NAME: &'static str = "SwapByCommitmentResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnclaimedSwapsRequest {}
impl ::prost::Name for UnclaimedSwapsRequest {
    const NAME: &'static str = "UnclaimedSwapsRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnclaimedSwapsResponse {
    #[prost(message, optional, tag = "1")]
    pub swap: ::core::option::Option<SwapRecord>,
}
impl ::prost::Name for UnclaimedSwapsResponse {
    const NAME: &'static str = "UnclaimedSwapsResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NullifierStatusRequest {
    #[prost(message, optional, tag = "2")]
    pub nullifier: ::core::option::Option<
        super::super::core::component::sct::v1alpha1::Nullifier,
    >,
    #[prost(bool, tag = "3")]
    pub await_detection: bool,
}
impl ::prost::Name for NullifierStatusRequest {
    const NAME: &'static str = "NullifierStatusRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NullifierStatusResponse {
    #[prost(bool, tag = "1")]
    pub spent: bool,
}
impl ::prost::Name for NullifierStatusResponse {
    const NAME: &'static str = "NullifierStatusResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfoByHashRequest {
    /// The transaction hash to query for.
    #[prost(message, optional, tag = "2")]
    pub id: ::core::option::Option<super::super::core::txhash::v1alpha1::TransactionId>,
}
impl ::prost::Name for TransactionInfoByHashRequest {
    const NAME: &'static str = "TransactionInfoByHashRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfoRequest {
    /// If present, return only transactions after this height.
    #[prost(uint64, tag = "1")]
    pub start_height: u64,
    /// If present, return only transactions before this height.
    #[prost(uint64, tag = "2")]
    pub end_height: u64,
}
impl ::prost::Name for TransactionInfoRequest {
    const NAME: &'static str = "TransactionInfoRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfo {
    /// The height the transaction was included in a block, if known.
    #[prost(uint64, tag = "1")]
    pub height: u64,
    /// The hash of the transaction.
    #[prost(message, optional, tag = "2")]
    pub id: ::core::option::Option<super::super::core::txhash::v1alpha1::TransactionId>,
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
impl ::prost::Name for TransactionInfo {
    const NAME: &'static str = "TransactionInfo";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub tx_info: ::core::option::Option<TransactionInfo>,
}
impl ::prost::Name for TransactionInfoResponse {
    const NAME: &'static str = "TransactionInfoResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfoByHashResponse {
    #[prost(message, optional, tag = "1")]
    pub tx_info: ::core::option::Option<TransactionInfo>,
}
impl ::prost::Name for TransactionInfoByHashResponse {
    const NAME: &'static str = "TransactionInfoByHashResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NotesResponse {
    #[prost(message, optional, tag = "1")]
    pub note_record: ::core::option::Option<SpendableNoteRecord>,
}
impl ::prost::Name for NotesResponse {
    const NAME: &'static str = "NotesResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NotesForVotingResponse {
    #[prost(message, optional, tag = "1")]
    pub note_record: ::core::option::Option<SpendableNoteRecord>,
    #[prost(message, optional, tag = "2")]
    pub identity_key: ::core::option::Option<
        super::super::core::keys::v1alpha1::IdentityKey,
    >,
}
impl ::prost::Name for NotesForVotingResponse {
    const NAME: &'static str = "NotesForVotingResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
/// A note plaintext with associated metadata about its status.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendableNoteRecord {
    /// The note commitment, identifying the note.
    #[prost(message, optional, tag = "1")]
    pub note_commitment: ::core::option::Option<
        super::super::crypto::tct::v1alpha1::StateCommitment,
    >,
    /// The note plaintext itself.
    #[prost(message, optional, tag = "2")]
    pub note: ::core::option::Option<
        super::super::core::component::shielded_pool::v1alpha1::Note,
    >,
    /// A precomputed decryption of the note's address incore.component.dex.v1alpha1.
    #[prost(message, optional, tag = "3")]
    pub address_index: ::core::option::Option<
        super::super::core::keys::v1alpha1::AddressIndex,
    >,
    /// The note's nullifier.
    #[prost(message, optional, tag = "4")]
    pub nullifier: ::core::option::Option<
        super::super::core::component::sct::v1alpha1::Nullifier,
    >,
    /// The height at which the note was created.
    #[prost(uint64, tag = "5")]
    pub height_created: u64,
    /// Records whether the note was spent (and if so, at what height).
    #[prost(uint64, tag = "6")]
    pub height_spent: u64,
    /// The note position.
    #[prost(uint64, tag = "7")]
    pub position: u64,
    /// The source of the note
    #[prost(message, optional, tag = "8")]
    pub source: ::core::option::Option<
        super::super::core::component::sct::v1alpha1::CommitmentSource,
    >,
}
impl ::prost::Name for SpendableNoteRecord {
    const NAME: &'static str = "SpendableNoteRecord";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapRecord {
    #[prost(message, optional, tag = "1")]
    pub swap_commitment: ::core::option::Option<
        super::super::crypto::tct::v1alpha1::StateCommitment,
    >,
    #[prost(message, optional, tag = "2")]
    pub swap: ::core::option::Option<
        super::super::core::component::dex::v1alpha1::SwapPlaintext,
    >,
    #[prost(uint64, tag = "3")]
    pub position: u64,
    #[prost(message, optional, tag = "4")]
    pub nullifier: ::core::option::Option<
        super::super::core::component::sct::v1alpha1::Nullifier,
    >,
    #[prost(message, optional, tag = "5")]
    pub output_data: ::core::option::Option<
        super::super::core::component::dex::v1alpha1::BatchSwapOutputData,
    >,
    #[prost(uint64, tag = "6")]
    pub height_claimed: u64,
    #[prost(message, optional, tag = "7")]
    pub source: ::core::option::Option<
        super::super::core::component::sct::v1alpha1::CommitmentSource,
    >,
}
impl ::prost::Name for SwapRecord {
    const NAME: &'static str = "SwapRecord";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OwnedPositionIdsRequest {
    /// If present, return only positions with this position state.
    #[prost(message, optional, tag = "1")]
    pub position_state: ::core::option::Option<
        super::super::core::component::dex::v1alpha1::PositionState,
    >,
    /// If present, return only positions for this trading pair.
    #[prost(message, optional, tag = "2")]
    pub trading_pair: ::core::option::Option<
        super::super::core::component::dex::v1alpha1::TradingPair,
    >,
}
impl ::prost::Name for OwnedPositionIdsRequest {
    const NAME: &'static str = "OwnedPositionIdsRequest";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OwnedPositionIdsResponse {
    #[prost(message, optional, tag = "1")]
    pub position_id: ::core::option::Option<
        super::super::core::component::dex::v1alpha1::PositionId,
    >,
}
impl ::prost::Name for OwnedPositionIdsResponse {
    const NAME: &'static str = "OwnedPositionIdsResponse";
    const PACKAGE: &'static str = "penumbra.view.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.view.v1alpha1.{}", Self::NAME)
    }
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
    #[derive(Debug, Clone)]
    pub struct ViewProtocolServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ViewProtocolServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
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
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Get current status of chain sync
        pub async fn status(
            &mut self,
            request: impl tonic::IntoRequest<super::StatusRequest>,
        ) -> std::result::Result<tonic::Response<super::StatusResponse>, tonic::Status> {
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
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "Status",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Stream sync status updates until the view service has caught up with the chain.
        /// Returns a stream of `StatusStreamResponse`s.
        pub async fn status_stream(
            &mut self,
            request: impl tonic::IntoRequest<super::StatusStreamRequest>,
        ) -> std::result::Result<
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
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "StatusStream",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        /// Queries for notes that have been accepted by the chain.
        /// Returns a stream of `NotesResponse`s.
        pub async fn notes(
            &mut self,
            request: impl tonic::IntoRequest<super::NotesRequest>,
        ) -> std::result::Result<
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
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "Notes",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        /// Returns a stream of `NotesForVotingResponse`s.
        pub async fn notes_for_voting(
            &mut self,
            request: impl tonic::IntoRequest<super::NotesForVotingRequest>,
        ) -> std::result::Result<
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
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "NotesForVoting",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
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
        ) -> std::result::Result<
            tonic::Response<super::WitnessResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/Witness",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "Witness",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn witness_and_build(
            &mut self,
            request: impl tonic::IntoRequest<super::WitnessAndBuildRequest>,
        ) -> std::result::Result<
            tonic::Response<super::WitnessAndBuildResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/WitnessAndBuild",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "WitnessAndBuild",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Queries for assets.
        /// Returns a stream of `AssetsResponse`s.
        pub async fn assets(
            &mut self,
            request: impl tonic::IntoRequest<super::AssetsRequest>,
        ) -> std::result::Result<
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
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "Assets",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        /// Query for the current app parameters.
        pub async fn app_parameters(
            &mut self,
            request: impl tonic::IntoRequest<super::AppParametersRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AppParametersResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/AppParameters",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "AppParameters",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Query for the current gas prices.
        pub async fn gas_prices(
            &mut self,
            request: impl tonic::IntoRequest<super::GasPricesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GasPricesResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/GasPrices",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "GasPrices",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Query for the current FMD parameters.
        pub async fn fmd_parameters(
            &mut self,
            request: impl tonic::IntoRequest<super::FmdParametersRequest>,
        ) -> std::result::Result<
            tonic::Response<super::FmdParametersResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/FMDParameters",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "FMDParameters",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Query for an address given an address index
        pub async fn address_by_index(
            &mut self,
            request: impl tonic::IntoRequest<super::AddressByIndexRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AddressByIndexResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/AddressByIndex",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "AddressByIndex",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Query for wallet id
        pub async fn wallet_id(
            &mut self,
            request: impl tonic::IntoRequest<super::WalletIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::WalletIdResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/WalletId",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "WalletId",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Query for an address given an address index
        pub async fn index_by_address(
            &mut self,
            request: impl tonic::IntoRequest<super::IndexByAddressRequest>,
        ) -> std::result::Result<
            tonic::Response<super::IndexByAddressResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/IndexByAddress",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "IndexByAddress",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Query for an ephemeral address
        pub async fn ephemeral_address(
            &mut self,
            request: impl tonic::IntoRequest<super::EphemeralAddressRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EphemeralAddressResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/EphemeralAddress",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "EphemeralAddress",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Query for balance of a given address.
        /// Returns a stream of `BalancesResponses`.
        pub async fn balances(
            &mut self,
            request: impl tonic::IntoRequest<super::BalancesRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::BalancesResponse>>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/Balances",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "Balances",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        /// Query for a note by its note commitment, optionally waiting until the note is detected.
        pub async fn note_by_commitment(
            &mut self,
            request: impl tonic::IntoRequest<super::NoteByCommitmentRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NoteByCommitmentResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/NoteByCommitment",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "NoteByCommitment",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Query for a swap by its swap commitment, optionally waiting until the swap is detected.
        pub async fn swap_by_commitment(
            &mut self,
            request: impl tonic::IntoRequest<super::SwapByCommitmentRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SwapByCommitmentResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/SwapByCommitment",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "SwapByCommitment",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Query for all unclaimed swaps.
        pub async fn unclaimed_swaps(
            &mut self,
            request: impl tonic::IntoRequest<super::UnclaimedSwapsRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::UnclaimedSwapsResponse>>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/UnclaimedSwaps",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "UnclaimedSwaps",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        /// Query for whether a nullifier has been spent, optionally waiting until it is spent.
        pub async fn nullifier_status(
            &mut self,
            request: impl tonic::IntoRequest<super::NullifierStatusRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NullifierStatusResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/NullifierStatus",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "NullifierStatus",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Query for a given transaction by its hash.
        pub async fn transaction_info_by_hash(
            &mut self,
            request: impl tonic::IntoRequest<super::TransactionInfoByHashRequest>,
        ) -> std::result::Result<
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
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "TransactionInfoByHash",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Query for the full transactions in the given range of blocks.
        /// Returns a stream of `TransactionInfoResponse`s.
        pub async fn transaction_info(
            &mut self,
            request: impl tonic::IntoRequest<super::TransactionInfoRequest>,
        ) -> std::result::Result<
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
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "TransactionInfo",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        /// Query for a transaction plan
        pub async fn transaction_planner(
            &mut self,
            request: impl tonic::IntoRequest<super::TransactionPlannerRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TransactionPlannerResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/TransactionPlanner",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "TransactionPlanner",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Broadcast a transaction to the network, optionally waiting for full confirmation.
        pub async fn broadcast_transaction(
            &mut self,
            request: impl tonic::IntoRequest<super::BroadcastTransactionRequest>,
        ) -> std::result::Result<
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
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "BroadcastTransaction",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Query for owned position IDs for the given trading pair and in the given position state.
        pub async fn owned_position_ids(
            &mut self,
            request: impl tonic::IntoRequest<super::OwnedPositionIdsRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::OwnedPositionIdsResponse>>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/OwnedPositionIds",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "OwnedPositionIds",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        /// Authorize a transaction plan and build the transaction.
        pub async fn authorize_and_build(
            &mut self,
            request: impl tonic::IntoRequest<super::AuthorizeAndBuildRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AuthorizeAndBuildResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/AuthorizeAndBuild",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.view.v1alpha1.ViewProtocolService",
                        "AuthorizeAndBuild",
                    ),
                );
            self.inner.unary(req, path, codec).await
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
        ) -> std::result::Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        /// Server streaming response type for the StatusStream method.
        type StatusStreamStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::StatusStreamResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Stream sync status updates until the view service has caught up with the chain.
        /// Returns a stream of `StatusStreamResponse`s.
        async fn status_stream(
            &self,
            request: tonic::Request<super::StatusStreamRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::StatusStreamStream>,
            tonic::Status,
        >;
        /// Server streaming response type for the Notes method.
        type NotesStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::NotesResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Queries for notes that have been accepted by the chain.
        /// Returns a stream of `NotesResponse`s.
        async fn notes(
            &self,
            request: tonic::Request<super::NotesRequest>,
        ) -> std::result::Result<tonic::Response<Self::NotesStream>, tonic::Status>;
        /// Server streaming response type for the NotesForVoting method.
        type NotesForVotingStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::NotesForVotingResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Returns a stream of `NotesForVotingResponse`s.
        async fn notes_for_voting(
            &self,
            request: tonic::Request<super::NotesForVotingRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::NotesForVotingStream>,
            tonic::Status,
        >;
        /// Returns authentication paths for the given note commitments.
        ///
        /// This method takes a batch of input commitments, rather than just one, so
        /// that the client can get a consistent set of authentication paths to a
        /// common root.  (Otherwise, if a client made multiple requests, the wallet
        /// service could have advanced the state commitment tree state between queries).
        async fn witness(
            &self,
            request: tonic::Request<super::WitnessRequest>,
        ) -> std::result::Result<tonic::Response<super::WitnessResponse>, tonic::Status>;
        async fn witness_and_build(
            &self,
            request: tonic::Request<super::WitnessAndBuildRequest>,
        ) -> std::result::Result<
            tonic::Response<super::WitnessAndBuildResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the Assets method.
        type AssetsStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::AssetsResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Queries for assets.
        /// Returns a stream of `AssetsResponse`s.
        async fn assets(
            &self,
            request: tonic::Request<super::AssetsRequest>,
        ) -> std::result::Result<tonic::Response<Self::AssetsStream>, tonic::Status>;
        /// Query for the current app parameters.
        async fn app_parameters(
            &self,
            request: tonic::Request<super::AppParametersRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AppParametersResponse>,
            tonic::Status,
        >;
        /// Query for the current gas prices.
        async fn gas_prices(
            &self,
            request: tonic::Request<super::GasPricesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GasPricesResponse>,
            tonic::Status,
        >;
        /// Query for the current FMD parameters.
        async fn fmd_parameters(
            &self,
            request: tonic::Request<super::FmdParametersRequest>,
        ) -> std::result::Result<
            tonic::Response<super::FmdParametersResponse>,
            tonic::Status,
        >;
        /// Query for an address given an address index
        async fn address_by_index(
            &self,
            request: tonic::Request<super::AddressByIndexRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AddressByIndexResponse>,
            tonic::Status,
        >;
        /// Query for wallet id
        async fn wallet_id(
            &self,
            request: tonic::Request<super::WalletIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::WalletIdResponse>,
            tonic::Status,
        >;
        /// Query for an address given an address index
        async fn index_by_address(
            &self,
            request: tonic::Request<super::IndexByAddressRequest>,
        ) -> std::result::Result<
            tonic::Response<super::IndexByAddressResponse>,
            tonic::Status,
        >;
        /// Query for an ephemeral address
        async fn ephemeral_address(
            &self,
            request: tonic::Request<super::EphemeralAddressRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EphemeralAddressResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the Balances method.
        type BalancesStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::BalancesResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Query for balance of a given address.
        /// Returns a stream of `BalancesResponses`.
        async fn balances(
            &self,
            request: tonic::Request<super::BalancesRequest>,
        ) -> std::result::Result<tonic::Response<Self::BalancesStream>, tonic::Status>;
        /// Query for a note by its note commitment, optionally waiting until the note is detected.
        async fn note_by_commitment(
            &self,
            request: tonic::Request<super::NoteByCommitmentRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NoteByCommitmentResponse>,
            tonic::Status,
        >;
        /// Query for a swap by its swap commitment, optionally waiting until the swap is detected.
        async fn swap_by_commitment(
            &self,
            request: tonic::Request<super::SwapByCommitmentRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SwapByCommitmentResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the UnclaimedSwaps method.
        type UnclaimedSwapsStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::UnclaimedSwapsResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Query for all unclaimed swaps.
        async fn unclaimed_swaps(
            &self,
            request: tonic::Request<super::UnclaimedSwapsRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::UnclaimedSwapsStream>,
            tonic::Status,
        >;
        /// Query for whether a nullifier has been spent, optionally waiting until it is spent.
        async fn nullifier_status(
            &self,
            request: tonic::Request<super::NullifierStatusRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NullifierStatusResponse>,
            tonic::Status,
        >;
        /// Query for a given transaction by its hash.
        async fn transaction_info_by_hash(
            &self,
            request: tonic::Request<super::TransactionInfoByHashRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TransactionInfoByHashResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the TransactionInfo method.
        type TransactionInfoStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::TransactionInfoResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Query for the full transactions in the given range of blocks.
        /// Returns a stream of `TransactionInfoResponse`s.
        async fn transaction_info(
            &self,
            request: tonic::Request<super::TransactionInfoRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::TransactionInfoStream>,
            tonic::Status,
        >;
        /// Query for a transaction plan
        async fn transaction_planner(
            &self,
            request: tonic::Request<super::TransactionPlannerRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TransactionPlannerResponse>,
            tonic::Status,
        >;
        /// Broadcast a transaction to the network, optionally waiting for full confirmation.
        async fn broadcast_transaction(
            &self,
            request: tonic::Request<super::BroadcastTransactionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BroadcastTransactionResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the OwnedPositionIds method.
        type OwnedPositionIdsStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<
                    super::OwnedPositionIdsResponse,
                    tonic::Status,
                >,
            >
            + Send
            + 'static;
        /// Query for owned position IDs for the given trading pair and in the given position state.
        async fn owned_position_ids(
            &self,
            request: tonic::Request<super::OwnedPositionIdsRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::OwnedPositionIdsStream>,
            tonic::Status,
        >;
        /// Authorize a transaction plan and build the transaction.
        async fn authorize_and_build(
            &self,
            request: tonic::Request<super::AuthorizeAndBuildRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AuthorizeAndBuildResponse>,
            tonic::Status,
        >;
    }
    /// The view protocol is used by a view client, who wants to do some
    /// transaction-related actions, to request data from a view service, which is
    /// responsible for synchronizing and scanning the public chain state with one or
    /// more full viewing keys.
    #[derive(Debug)]
    pub struct ViewProtocolServiceServer<T: ViewProtocolService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
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
                max_decoding_message_size: None,
                max_encoding_message_size: None,
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
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
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
        ) -> Poll<std::result::Result<(), Self::Error>> {
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::status(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StatusSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::status_stream(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StatusStreamSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::notes(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NotesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::notes_for_voting(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NotesForVotingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::witness(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = WitnessSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::witness_and_build(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = WitnessAndBuildSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::assets(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AssetsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/AppParameters" => {
                    #[allow(non_camel_case_types)]
                    struct AppParametersSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::AppParametersRequest>
                    for AppParametersSvc<T> {
                        type Response = super::AppParametersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AppParametersRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::app_parameters(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AppParametersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/GasPrices" => {
                    #[allow(non_camel_case_types)]
                    struct GasPricesSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::GasPricesRequest>
                    for GasPricesSvc<T> {
                        type Response = super::GasPricesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GasPricesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::gas_prices(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GasPricesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::fmd_parameters(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FMDParametersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::address_by_index(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AddressByIndexSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/WalletId" => {
                    #[allow(non_camel_case_types)]
                    struct WalletIdSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::WalletIdRequest>
                    for WalletIdSvc<T> {
                        type Response = super::WalletIdResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::WalletIdRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::wallet_id(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = WalletIdSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::index_by_address(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = IndexByAddressSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::ephemeral_address(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EphemeralAddressSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/Balances" => {
                    #[allow(non_camel_case_types)]
                    struct BalancesSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::ServerStreamingService<super::BalancesRequest>
                    for BalancesSvc<T> {
                        type Response = super::BalancesResponse;
                        type ResponseStream = T::BalancesStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BalancesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::balances(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BalancesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::note_by_commitment(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NoteByCommitmentSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::swap_by_commitment(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SwapByCommitmentSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/UnclaimedSwaps" => {
                    #[allow(non_camel_case_types)]
                    struct UnclaimedSwapsSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::ServerStreamingService<super::UnclaimedSwapsRequest>
                    for UnclaimedSwapsSvc<T> {
                        type Response = super::UnclaimedSwapsResponse;
                        type ResponseStream = T::UnclaimedSwapsStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UnclaimedSwapsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::unclaimed_swaps(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UnclaimedSwapsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::nullifier_status(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NullifierStatusSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::transaction_info_by_hash(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TransactionInfoByHashSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::transaction_info(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TransactionInfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::transaction_planner(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TransactionPlannerSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::broadcast_transaction(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BroadcastTransactionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                    > tonic::server::ServerStreamingService<
                        super::OwnedPositionIdsRequest,
                    > for OwnedPositionIdsSvc<T> {
                        type Response = super::OwnedPositionIdsResponse;
                        type ResponseStream = T::OwnedPositionIdsStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::OwnedPositionIdsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::owned_position_ids(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = OwnedPositionIdsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.view.v1alpha1.ViewProtocolService/AuthorizeAndBuild" => {
                    #[allow(non_camel_case_types)]
                    struct AuthorizeAndBuildSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::AuthorizeAndBuildRequest>
                    for AuthorizeAndBuildSvc<T> {
                        type Response = super::AuthorizeAndBuildResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AuthorizeAndBuildRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ViewProtocolService>::authorize_and_build(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AuthorizeAndBuildSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
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
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: ViewProtocolService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
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
