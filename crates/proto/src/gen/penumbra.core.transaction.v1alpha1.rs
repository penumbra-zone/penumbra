/// A Penumbra transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<TransactionBody>,
    /// The binding signature is stored separately from the transaction body that it signs.
    #[prost(message, optional, tag = "2")]
    pub binding_sig: ::core::option::Option<
        super::super::super::crypto::decaf377_rdsa::v1alpha1::BindingSignature,
    >,
    /// The root of some previous state of the state commitment tree, used as an anchor for all
    /// ZK state transition proofs.
    #[prost(message, optional, tag = "3")]
    pub anchor: ::core::option::Option<
        super::super::super::crypto::tct::v1alpha1::MerkleRoot,
    >,
}
impl ::prost::Name for Transaction {
    const NAME: &'static str = "Transaction";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// The body of a transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionBody {
    /// A list of actions (state changes) performed by this transaction.
    #[prost(message, repeated, tag = "1")]
    pub actions: ::prost::alloc::vec::Vec<Action>,
    /// Parameters determining if a transaction should be accepted by this chain.
    #[prost(message, optional, tag = "2")]
    pub transaction_parameters: ::core::option::Option<TransactionParameters>,
    /// Detection data for use with Fuzzy Message Detection
    #[prost(message, optional, tag = "4")]
    pub detection_data: ::core::option::Option<DetectionData>,
    /// The encrypted memo for this transaction.
    ///
    /// This field will be present if and only if the transaction has outputs.
    #[prost(message, optional, tag = "5")]
    pub memo: ::core::option::Option<MemoCiphertext>,
}
impl ::prost::Name for TransactionBody {
    const NAME: &'static str = "TransactionBody";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// The parameters determining if a transaction should be accepted by the chain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionParameters {
    /// The maximum height that this transaction can be included in the chain.
    ///
    /// If zero, there is no maximum.
    #[prost(uint64, tag = "1")]
    pub expiry_height: u64,
    /// The chain this transaction is intended for.  Including this prevents
    /// replaying a transaction on one chain onto a different chain.
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    /// The transaction fee.
    #[prost(message, optional, tag = "3")]
    pub fee: ::core::option::Option<super::super::component::fee::v1alpha1::Fee>,
}
impl ::prost::Name for TransactionParameters {
    const NAME: &'static str = "TransactionParameters";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// Detection data used by a detection server performing Fuzzy Message Detection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DetectionData {
    /// A list of clues for use with Fuzzy Message Detection.
    #[prost(message, repeated, tag = "4")]
    pub fmd_clues: ::prost::alloc::vec::Vec<
        super::super::super::crypto::decaf377_fmd::v1alpha1::Clue,
    >,
}
impl ::prost::Name for DetectionData {
    const NAME: &'static str = "DetectionData";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// A state change performed by a transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Action {
    #[prost(
        oneof = "action::Action",
        tags = "1, 2, 3, 4, 16, 17, 18, 19, 20, 21, 22, 30, 31, 32, 34, 40, 41, 42, 50, 51, 52, 200"
    )]
    pub action: ::core::option::Option<action::Action>,
}
/// Nested message and enum types in `Action`.
pub mod action {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Action {
        /// Common actions have numbers \< 15, to save space.
        #[prost(message, tag = "1")]
        Spend(super::super::super::component::shielded_pool::v1alpha1::Spend),
        #[prost(message, tag = "2")]
        Output(super::super::super::component::shielded_pool::v1alpha1::Output),
        #[prost(message, tag = "3")]
        Swap(super::super::super::component::dex::v1alpha1::Swap),
        #[prost(message, tag = "4")]
        SwapClaim(super::super::super::component::dex::v1alpha1::SwapClaim),
        #[prost(message, tag = "16")]
        ValidatorDefinition(
            super::super::super::component::stake::v1alpha1::ValidatorDefinition,
        ),
        #[prost(message, tag = "17")]
        IbcRelayAction(super::super::super::component::ibc::v1alpha1::IbcRelay),
        /// Governance:
        #[prost(message, tag = "18")]
        ProposalSubmit(
            super::super::super::component::governance::v1alpha1::ProposalSubmit,
        ),
        #[prost(message, tag = "19")]
        ProposalWithdraw(
            super::super::super::component::governance::v1alpha1::ProposalWithdraw,
        ),
        #[prost(message, tag = "20")]
        ValidatorVote(
            super::super::super::component::governance::v1alpha1::ValidatorVote,
        ),
        #[prost(message, tag = "21")]
        DelegatorVote(
            super::super::super::component::governance::v1alpha1::DelegatorVote,
        ),
        #[prost(message, tag = "22")]
        ProposalDepositClaim(
            super::super::super::component::governance::v1alpha1::ProposalDepositClaim,
        ),
        /// Positions
        #[prost(message, tag = "30")]
        PositionOpen(super::super::super::component::dex::v1alpha1::PositionOpen),
        #[prost(message, tag = "31")]
        PositionClose(super::super::super::component::dex::v1alpha1::PositionClose),
        #[prost(message, tag = "32")]
        PositionWithdraw(
            super::super::super::component::dex::v1alpha1::PositionWithdraw,
        ),
        #[prost(message, tag = "34")]
        PositionRewardClaim(
            super::super::super::component::dex::v1alpha1::PositionRewardClaim,
        ),
        /// (un)delegation
        #[prost(message, tag = "40")]
        Delegate(super::super::super::component::stake::v1alpha1::Delegate),
        #[prost(message, tag = "41")]
        Undelegate(super::super::super::component::stake::v1alpha1::Undelegate),
        #[prost(message, tag = "42")]
        UndelegateClaim(
            super::super::super::component::stake::v1alpha1::UndelegateClaim,
        ),
        /// Community Pool
        #[prost(message, tag = "50")]
        CommunityPoolSpend(
            super::super::super::component::governance::v1alpha1::CommunityPoolSpend,
        ),
        #[prost(message, tag = "51")]
        CommunityPoolOutput(
            super::super::super::component::governance::v1alpha1::CommunityPoolOutput,
        ),
        #[prost(message, tag = "52")]
        CommunityPoolDeposit(
            super::super::super::component::governance::v1alpha1::CommunityPoolDeposit,
        ),
        #[prost(message, tag = "200")]
        Ics20Withdrawal(super::super::super::component::ibc::v1alpha1::Ics20Withdrawal),
    }
}
impl ::prost::Name for Action {
    const NAME: &'static str = "Action";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// A transaction perspective is a bundle of key material and commitment openings
/// that allow generating a view of a transaction from that perspective.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionPerspective {
    #[prost(message, repeated, tag = "1")]
    pub payload_keys: ::prost::alloc::vec::Vec<PayloadKeyWithCommitment>,
    #[prost(message, repeated, tag = "2")]
    pub spend_nullifiers: ::prost::alloc::vec::Vec<NullifierWithNote>,
    /// The openings of note commitments referred to in the transaction
    /// but not included in the transaction.
    #[prost(message, repeated, tag = "3")]
    pub advice_notes: ::prost::alloc::vec::Vec<
        super::super::component::shielded_pool::v1alpha1::Note,
    >,
    /// Any relevant address views.
    #[prost(message, repeated, tag = "4")]
    pub address_views: ::prost::alloc::vec::Vec<
        super::super::keys::v1alpha1::AddressView,
    >,
    /// Any relevant denoms for viewed assets.
    #[prost(message, repeated, tag = "5")]
    pub denoms: ::prost::alloc::vec::Vec<super::super::asset::v1alpha1::DenomMetadata>,
    /// The transaction ID associated with this TransactionPerspective
    #[prost(message, optional, tag = "6")]
    pub transaction_id: ::core::option::Option<
        super::super::txhash::v1alpha1::TransactionId,
    >,
}
impl ::prost::Name for TransactionPerspective {
    const NAME: &'static str = "TransactionPerspective";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PayloadKeyWithCommitment {
    #[prost(message, optional, tag = "1")]
    pub payload_key: ::core::option::Option<super::super::keys::v1alpha1::PayloadKey>,
    #[prost(message, optional, tag = "2")]
    pub commitment: ::core::option::Option<
        super::super::super::crypto::tct::v1alpha1::StateCommitment,
    >,
}
impl ::prost::Name for PayloadKeyWithCommitment {
    const NAME: &'static str = "PayloadKeyWithCommitment";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NullifierWithNote {
    #[prost(message, optional, tag = "1")]
    pub nullifier: ::core::option::Option<
        super::super::component::sct::v1alpha1::Nullifier,
    >,
    #[prost(message, optional, tag = "2")]
    pub note: ::core::option::Option<
        super::super::component::shielded_pool::v1alpha1::Note,
    >,
}
impl ::prost::Name for NullifierWithNote {
    const NAME: &'static str = "NullifierWithNote";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// View of a Penumbra transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionView {
    /// View of the transaction body
    #[prost(message, optional, tag = "1")]
    pub body_view: ::core::option::Option<TransactionBodyView>,
    /// The binding signature is stored separately from the transaction body that it signs.
    #[prost(message, optional, tag = "2")]
    pub binding_sig: ::core::option::Option<
        super::super::super::crypto::decaf377_rdsa::v1alpha1::BindingSignature,
    >,
    /// The root of some previous state of the state commitment tree, used as an anchor for all
    /// ZK state transition proofs.
    #[prost(message, optional, tag = "3")]
    pub anchor: ::core::option::Option<
        super::super::super::crypto::tct::v1alpha1::MerkleRoot,
    >,
}
impl ::prost::Name for TransactionView {
    const NAME: &'static str = "TransactionView";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionBodyView {
    /// A list views into of actions (state changes) performed by this transaction.
    #[prost(message, repeated, tag = "1")]
    pub action_views: ::prost::alloc::vec::Vec<ActionView>,
    /// Transaction parameters.
    #[prost(message, optional, tag = "2")]
    pub transaction_parameters: ::core::option::Option<TransactionParameters>,
    /// The detection data in this transaction, only populated if
    /// there are outputs in the actions of this transaction.
    #[prost(message, optional, tag = "4")]
    pub detection_data: ::core::option::Option<DetectionData>,
    /// An optional view of a transaction memo. It will only be populated if there are
    /// outputs in the actions of this transaction.
    #[prost(message, optional, tag = "5")]
    pub memo_view: ::core::option::Option<MemoView>,
}
impl ::prost::Name for TransactionBodyView {
    const NAME: &'static str = "TransactionBodyView";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// A view of a specific state change action performed by a transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActionView {
    #[prost(
        oneof = "action_view::ActionView",
        tags = "1, 2, 3, 4, 16, 17, 18, 19, 20, 21, 22, 30, 31, 32, 34, 41, 42, 50, 51, 52, 43, 200"
    )]
    pub action_view: ::core::option::Option<action_view::ActionView>,
}
/// Nested message and enum types in `ActionView`.
pub mod action_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum ActionView {
        /// Action types with visible/opaque variants
        #[prost(message, tag = "1")]
        Spend(super::super::super::component::shielded_pool::v1alpha1::SpendView),
        #[prost(message, tag = "2")]
        Output(super::super::super::component::shielded_pool::v1alpha1::OutputView),
        #[prost(message, tag = "3")]
        Swap(super::super::super::component::dex::v1alpha1::SwapView),
        #[prost(message, tag = "4")]
        SwapClaim(super::super::super::component::dex::v1alpha1::SwapClaimView),
        /// Action types without visible/opaque variants
        #[prost(message, tag = "16")]
        ValidatorDefinition(
            super::super::super::component::stake::v1alpha1::ValidatorDefinition,
        ),
        #[prost(message, tag = "17")]
        IbcRelayAction(super::super::super::component::ibc::v1alpha1::IbcRelay),
        /// Governance:
        #[prost(message, tag = "18")]
        ProposalSubmit(
            super::super::super::component::governance::v1alpha1::ProposalSubmit,
        ),
        #[prost(message, tag = "19")]
        ProposalWithdraw(
            super::super::super::component::governance::v1alpha1::ProposalWithdraw,
        ),
        #[prost(message, tag = "20")]
        ValidatorVote(
            super::super::super::component::governance::v1alpha1::ValidatorVote,
        ),
        #[prost(message, tag = "21")]
        DelegatorVote(
            super::super::super::component::governance::v1alpha1::DelegatorVoteView,
        ),
        #[prost(message, tag = "22")]
        ProposalDepositClaim(
            super::super::super::component::governance::v1alpha1::ProposalDepositClaim,
        ),
        #[prost(message, tag = "30")]
        PositionOpen(super::super::super::component::dex::v1alpha1::PositionOpen),
        #[prost(message, tag = "31")]
        PositionClose(super::super::super::component::dex::v1alpha1::PositionClose),
        #[prost(message, tag = "32")]
        PositionWithdraw(
            super::super::super::component::dex::v1alpha1::PositionWithdraw,
        ),
        #[prost(message, tag = "34")]
        PositionRewardClaim(
            super::super::super::component::dex::v1alpha1::PositionRewardClaim,
        ),
        #[prost(message, tag = "41")]
        Delegate(super::super::super::component::stake::v1alpha1::Delegate),
        #[prost(message, tag = "42")]
        Undelegate(super::super::super::component::stake::v1alpha1::Undelegate),
        /// Community Pool
        #[prost(message, tag = "50")]
        CommunityPoolSpend(
            super::super::super::component::governance::v1alpha1::CommunityPoolSpend,
        ),
        #[prost(message, tag = "51")]
        CommunityPoolOutput(
            super::super::super::component::governance::v1alpha1::CommunityPoolOutput,
        ),
        #[prost(message, tag = "52")]
        CommunityPoolDeposit(
            super::super::super::component::governance::v1alpha1::CommunityPoolDeposit,
        ),
        /// TODO: we have no way to recover the opening of the undelegate_claim's
        /// balance commitment, and can only infer the value from looking at the rest
        /// of the transaction. is that fine?
        #[prost(message, tag = "43")]
        UndelegateClaim(
            super::super::super::component::stake::v1alpha1::UndelegateClaim,
        ),
        #[prost(message, tag = "200")]
        Ics20Withdrawal(super::super::super::component::ibc::v1alpha1::Ics20Withdrawal),
    }
}
impl ::prost::Name for ActionView {
    const NAME: &'static str = "ActionView";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// The data required to authorize a transaction plan.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthorizationData {
    /// The computed auth hash for the approved transaction plan.
    #[prost(message, optional, tag = "1")]
    pub effect_hash: ::core::option::Option<super::super::txhash::v1alpha1::EffectHash>,
    /// The required spend authorizations, returned in the same order as the
    /// Spend actions in the original request.
    #[prost(message, repeated, tag = "2")]
    pub spend_auths: ::prost::alloc::vec::Vec<
        super::super::super::crypto::decaf377_rdsa::v1alpha1::SpendAuthSignature,
    >,
    /// The required delegator vote authorizations, returned in the same order as the
    /// DelegatorVote actions in the original request.
    #[prost(message, repeated, tag = "3")]
    pub delegator_vote_auths: ::prost::alloc::vec::Vec<
        super::super::super::crypto::decaf377_rdsa::v1alpha1::SpendAuthSignature,
    >,
}
impl ::prost::Name for AuthorizationData {
    const NAME: &'static str = "AuthorizationData";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// The data required for proving when building a transaction from a plan.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WitnessData {
    /// The anchor for the state transition proofs.
    #[prost(message, optional, tag = "1")]
    pub anchor: ::core::option::Option<
        super::super::super::crypto::tct::v1alpha1::MerkleRoot,
    >,
    /// The auth paths for the notes the transaction spends, in the
    /// same order as the spends in the transaction plan.
    #[prost(message, repeated, tag = "2")]
    pub state_commitment_proofs: ::prost::alloc::vec::Vec<
        super::super::super::crypto::tct::v1alpha1::StateCommitmentProof,
    >,
}
impl ::prost::Name for WitnessData {
    const NAME: &'static str = "WitnessData";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// Describes a planned transaction. Permits clients to prepare a transaction
/// prior submission, so that a user can review it prior to authorizing its execution.
///
/// The `TransactionPlan` is a fully determined bundle binding all of a transaction's effects.
/// The only thing it does not include is the witness data used for proving.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionPlan {
    /// The sequence of actions planned for this transaction.
    #[prost(message, repeated, tag = "1")]
    pub actions: ::prost::alloc::vec::Vec<ActionPlan>,
    /// Parameters determining if a transaction should be accepted by this chain.
    #[prost(message, optional, tag = "2")]
    pub transaction_parameters: ::core::option::Option<TransactionParameters>,
    /// Detection data for use with Fuzzy Message Detection
    #[prost(message, optional, tag = "4")]
    pub detection_data: ::core::option::Option<DetectionDataPlan>,
    /// The memo plan for this transaction.
    #[prost(message, optional, tag = "5")]
    pub memo: ::core::option::Option<MemoPlan>,
}
impl ::prost::Name for TransactionPlan {
    const NAME: &'static str = "TransactionPlan";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DetectionDataPlan {
    #[prost(message, repeated, tag = "5")]
    pub clue_plans: ::prost::alloc::vec::Vec<CluePlan>,
}
impl ::prost::Name for DetectionDataPlan {
    const NAME: &'static str = "DetectionDataPlan";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// Describes a planned transaction action.
///
/// Some transaction Actions don't have any private data and are treated as being plans
/// themselves.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActionPlan {
    #[prost(
        oneof = "action_plan::Action",
        tags = "1, 2, 3, 4, 16, 17, 18, 19, 20, 21, 22, 23, 30, 31, 32, 34, 40, 41, 42, 50, 51, 52"
    )]
    pub action: ::core::option::Option<action_plan::Action>,
}
/// Nested message and enum types in `ActionPlan`.
pub mod action_plan {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Action {
        #[prost(message, tag = "1")]
        Spend(super::super::super::component::shielded_pool::v1alpha1::SpendPlan),
        #[prost(message, tag = "2")]
        Output(super::super::super::component::shielded_pool::v1alpha1::OutputPlan),
        #[prost(message, tag = "3")]
        Swap(super::super::super::component::dex::v1alpha1::SwapPlan),
        #[prost(message, tag = "4")]
        SwapClaim(super::super::super::component::dex::v1alpha1::SwapClaimPlan),
        /// This is just a message relayed to the chain.
        #[prost(message, tag = "16")]
        ValidatorDefinition(
            super::super::super::component::stake::v1alpha1::ValidatorDefinition,
        ),
        /// This is just a message relayed to the chain.
        #[prost(message, tag = "17")]
        IbcRelayAction(super::super::super::component::ibc::v1alpha1::IbcRelay),
        /// Governance:
        #[prost(message, tag = "18")]
        ProposalSubmit(
            super::super::super::component::governance::v1alpha1::ProposalSubmit,
        ),
        #[prost(message, tag = "19")]
        ProposalWithdraw(
            super::super::super::component::governance::v1alpha1::ProposalWithdraw,
        ),
        #[prost(message, tag = "20")]
        ValidatorVote(
            super::super::super::component::governance::v1alpha1::ValidatorVote,
        ),
        #[prost(message, tag = "21")]
        DelegatorVote(
            super::super::super::component::governance::v1alpha1::DelegatorVotePlan,
        ),
        #[prost(message, tag = "22")]
        ProposalDepositClaim(
            super::super::super::component::governance::v1alpha1::ProposalDepositClaim,
        ),
        #[prost(message, tag = "23")]
        Withdrawal(super::super::super::component::ibc::v1alpha1::Ics20Withdrawal),
        #[prost(message, tag = "30")]
        PositionOpen(super::super::super::component::dex::v1alpha1::PositionOpen),
        #[prost(message, tag = "31")]
        PositionClose(super::super::super::component::dex::v1alpha1::PositionClose),
        /// The position withdraw/reward claim actions require balance information so they have Plan types.
        #[prost(message, tag = "32")]
        PositionWithdraw(
            super::super::super::component::dex::v1alpha1::PositionWithdrawPlan,
        ),
        #[prost(message, tag = "34")]
        PositionRewardClaim(
            super::super::super::component::dex::v1alpha1::PositionRewardClaimPlan,
        ),
        /// We don't need any extra information (yet) to understand delegations,
        /// because we don't yet use flow encryption.
        #[prost(message, tag = "40")]
        Delegate(super::super::super::component::stake::v1alpha1::Delegate),
        /// We don't need any extra information (yet) to understand undelegations,
        /// because we don't yet use flow encryption.
        #[prost(message, tag = "41")]
        Undelegate(super::super::super::component::stake::v1alpha1::Undelegate),
        #[prost(message, tag = "42")]
        UndelegateClaim(
            super::super::super::component::stake::v1alpha1::UndelegateClaimPlan,
        ),
        /// Community Pool
        #[prost(message, tag = "50")]
        CommunityPoolSpend(
            super::super::super::component::governance::v1alpha1::CommunityPoolSpend,
        ),
        #[prost(message, tag = "51")]
        CommunityPoolOutput(
            super::super::super::component::governance::v1alpha1::CommunityPoolOutput,
        ),
        #[prost(message, tag = "52")]
        CommunityPoolDeposit(
            super::super::super::component::governance::v1alpha1::CommunityPoolDeposit,
        ),
    }
}
impl ::prost::Name for ActionPlan {
    const NAME: &'static str = "ActionPlan";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// Describes a plan for forming a `Clue`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CluePlan {
    /// The address.
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<super::super::keys::v1alpha1::Address>,
    /// The random seed to use for the clue plan.
    #[prost(bytes = "vec", tag = "2")]
    pub rseed: ::prost::alloc::vec::Vec<u8>,
    /// The bits of precision.
    #[prost(uint64, tag = "3")]
    pub precision_bits: u64,
}
impl ::prost::Name for CluePlan {
    const NAME: &'static str = "CluePlan";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// Describes a plan for forming the transaction memo.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MemoPlan {
    /// The plaintext.
    #[prost(message, optional, tag = "1")]
    pub plaintext: ::core::option::Option<MemoPlaintext>,
    /// The key to use to encrypt the memo.
    #[prost(bytes = "vec", tag = "2")]
    pub key: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MemoPlan {
    const NAME: &'static str = "MemoPlan";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// The encrypted memo data describing information about the purpose of a transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MemoCiphertext {
    /// The encrypted data. 528 bytes.
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MemoCiphertext {
    const NAME: &'static str = "MemoCiphertext";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
/// The plaintext describing information about the purpose of a transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MemoPlaintext {
    /// The sender's return address.
    ///
    /// This should always be a valid address; the sender is responsible for ensuring
    /// that if the receiver returns funds to this address, they will not be lost.
    #[prost(message, optional, tag = "1")]
    pub return_address: ::core::option::Option<super::super::keys::v1alpha1::Address>,
    /// Free-form text, up to 432 bytes long.
    #[prost(string, tag = "2")]
    pub text: ::prost::alloc::string::String,
}
impl ::prost::Name for MemoPlaintext {
    const NAME: &'static str = "MemoPlaintext";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MemoPlaintextView {
    #[prost(message, optional, tag = "1")]
    pub return_address: ::core::option::Option<
        super::super::keys::v1alpha1::AddressView,
    >,
    #[prost(string, tag = "2")]
    pub text: ::prost::alloc::string::String,
}
impl ::prost::Name for MemoPlaintextView {
    const NAME: &'static str = "MemoPlaintextView";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MemoView {
    #[prost(oneof = "memo_view::MemoView", tags = "1, 2")]
    pub memo_view: ::core::option::Option<memo_view::MemoView>,
}
/// Nested message and enum types in `MemoView`.
pub mod memo_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Visible {
        #[prost(message, optional, tag = "1")]
        pub ciphertext: ::core::option::Option<super::MemoCiphertext>,
        #[prost(message, optional, tag = "2")]
        pub plaintext: ::core::option::Option<super::MemoPlaintextView>,
    }
    impl ::prost::Name for Visible {
        const NAME: &'static str = "Visible";
        const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.transaction.v1alpha1.MemoView.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub ciphertext: ::core::option::Option<super::MemoCiphertext>,
    }
    impl ::prost::Name for Opaque {
        const NAME: &'static str = "Opaque";
        const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.transaction.v1alpha1.MemoView.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum MemoView {
        #[prost(message, tag = "1")]
        Visible(Visible),
        #[prost(message, tag = "2")]
        Opaque(Opaque),
    }
}
impl ::prost::Name for MemoView {
    const NAME: &'static str = "MemoView";
    const PACKAGE: &'static str = "penumbra.core.transaction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.transaction.v1alpha1.{}", Self::NAME)
    }
}
