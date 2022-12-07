/// An authorization hash for a Penumbra transaction.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[serde(transparent)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthHash {
    #[prost(bytes="bytes", tag="1")]
    #[serde(with = "crate::serializers::hexstr_bytes")]
    pub inner: ::prost::bytes::Bytes,
}
/// A Penumbra transaction.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    #[prost(message, optional, tag="1")]
    pub body: ::core::option::Option<TransactionBody>,
    /// The binding signature is stored separately from the transaction body that it signs.
    #[prost(bytes="bytes", tag="2")]
    #[serde(with = "crate::serializers::hexstr_bytes")]
    pub binding_sig: ::prost::bytes::Bytes,
    /// The root of some previous state of the note commitment tree, used as an anchor for all
    /// ZK state transition proofs.
    #[prost(message, optional, tag="3")]
    pub anchor: ::core::option::Option<super::super::crypto::v1alpha1::MerkleRoot>,
}
/// The body of a transaction.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionBody {
    /// A list of actions (state changes) performed by this transaction.
    #[prost(message, repeated, tag="1")]
    pub actions: ::prost::alloc::vec::Vec<Action>,
    /// The maximum height that this transaction can be included in the chain.
    ///
    /// If zero, there is no maximum.
    #[prost(uint64, tag="2")]
    pub expiry_height: u64,
    /// The chain this transaction is intended for.  Including this prevents
    /// replaying a transaction on one chain onto a different chain.
    #[prost(string, tag="3")]
    pub chain_id: ::prost::alloc::string::String,
    /// The transaction fee.
    #[prost(message, optional, tag="4")]
    pub fee: ::core::option::Option<super::super::crypto::v1alpha1::Fee>,
    /// A list of clues for use with Fuzzy Message Detection.
    #[prost(message, repeated, tag="5")]
    pub fmd_clues: ::prost::alloc::vec::Vec<super::super::crypto::v1alpha1::Clue>,
    /// An optional encrypted memo. It will only be populated if there are
    /// outputs in the actions of this transaction. 528 bytes.
    #[prost(bytes="bytes", optional, tag="6")]
    pub encrypted_memo: ::core::option::Option<::prost::bytes::Bytes>,
}
/// A state change performed by a transaction.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Action {
    #[prost(oneof="action::Action", tags="1, 2, 3, 4, 16, 17, 18, 19, 20, 30, 31, 32, 34, 40, 41, 42, 200")]
    pub action: ::core::option::Option<action::Action>,
}
/// Nested message and enum types in `Action`.
pub mod action {
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Action {
        #[prost(message, tag="1")]
        Spend(super::Spend),
        #[prost(message, tag="2")]
        Output(super::Output),
        #[prost(message, tag="3")]
        Swap(super::super::super::dex::v1alpha1::Swap),
        #[prost(message, tag="4")]
        SwapClaim(super::super::super::dex::v1alpha1::SwapClaim),
        // Uncommon actions have numbers > 15.

        #[prost(message, tag="16")]
        ValidatorDefinition(super::super::super::stake::v1alpha1::ValidatorDefinition),
        #[prost(message, tag="17")]
        IbcAction(super::super::super::ibc::v1alpha1::IbcAction),
        /// Governance:
        #[prost(message, tag="18")]
        ProposalSubmit(super::ProposalSubmit),
        #[prost(message, tag="19")]
        ProposalWithdraw(super::ProposalWithdraw),
        /// DelegatorVote delegator_vote = 21;
        #[prost(message, tag="20")]
        ValidatorVote(super::ValidatorVote),
        #[prost(message, tag="30")]
        PositionOpen(super::super::super::dex::v1alpha1::PositionOpen),
        #[prost(message, tag="31")]
        PositionClose(super::super::super::dex::v1alpha1::PositionClose),
        #[prost(message, tag="32")]
        PositionWithdraw(super::super::super::dex::v1alpha1::PositionWithdraw),
        #[prost(message, tag="34")]
        PositionRewardClaim(super::super::super::dex::v1alpha1::PositionRewardClaim),
        /// (un)delegation
        #[prost(message, tag="40")]
        Delegate(super::super::super::stake::v1alpha1::Delegate),
        #[prost(message, tag="41")]
        Undelegate(super::super::super::stake::v1alpha1::Undelegate),
        #[prost(message, tag="42")]
        UndelegateClaim(super::super::super::stake::v1alpha1::UndelegateClaim),
        #[prost(message, tag="200")]
        Ics20Withdrawal(super::super::super::ibc::v1alpha1::Ics20Withdrawal),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionPerspective {
    #[prost(message, repeated, tag="1")]
    pub payload_keys: ::prost::alloc::vec::Vec<PayloadKeyWithCommitment>,
    #[prost(message, repeated, tag="2")]
    pub spend_nullifiers: ::prost::alloc::vec::Vec<NullifierWithNote>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PayloadKeyWithCommitment {
    #[prost(bytes="bytes", tag="1")]
    pub payload_key: ::prost::bytes::Bytes,
    #[prost(message, optional, tag="2")]
    pub commitment: ::core::option::Option<super::super::crypto::v1alpha1::StateCommitment>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NullifierWithNote {
    #[prost(message, optional, tag="1")]
    pub nullifier: ::core::option::Option<super::super::crypto::v1alpha1::Nullifier>,
    #[prost(message, optional, tag="2")]
    pub note: ::core::option::Option<super::super::crypto::v1alpha1::Note>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionView {
    /// A list views into of actions (state changes) performed by this transaction.
    #[prost(message, repeated, tag="1")]
    pub action_views: ::prost::alloc::vec::Vec<ActionView>,
    /// The maximum height that this transaction can be included in the chain.
    ///
    /// If zero, there is no maximum.
    #[prost(uint64, tag="2")]
    pub expiry_height: u64,
    /// The chain this transaction is intended for.  Including this prevents
    /// replaying a transaction on one chain onto a different chain.
    #[prost(string, tag="3")]
    pub chain_id: ::prost::alloc::string::String,
    /// The transaction fee.
    #[prost(message, optional, tag="4")]
    pub fee: ::core::option::Option<super::super::crypto::v1alpha1::Fee>,
    /// A list of clues for use with Fuzzy Message Detection.
    #[prost(message, repeated, tag="5")]
    pub fmd_clues: ::prost::alloc::vec::Vec<super::super::crypto::v1alpha1::Clue>,
    /// An optional plaintext memo. It will only be populated if there are
    /// outputs in the actions of this transaction.
    #[prost(string, optional, tag="6")]
    pub memo: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendView {
    #[prost(message, optional, tag="1")]
    pub spend: ::core::option::Option<Spend>,
    #[prost(message, optional, tag="2")]
    pub note: ::core::option::Option<super::super::crypto::v1alpha1::Note>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutputView {
    #[prost(message, optional, tag="1")]
    pub output: ::core::option::Option<Output>,
    #[prost(message, optional, tag="2")]
    pub note: ::core::option::Option<super::super::crypto::v1alpha1::Note>,
    #[prost(bytes="bytes", tag="3")]
    pub payload_key: ::prost::bytes::Bytes,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapView {
    #[prost(message, optional, tag="1")]
    pub swap: ::core::option::Option<super::super::dex::v1alpha1::Swap>,
    #[prost(message, optional, tag="2")]
    pub swap_nft: ::core::option::Option<super::super::crypto::v1alpha1::Note>,
    #[prost(message, optional, tag="3")]
    pub swap_plaintext: ::core::option::Option<super::super::dex::v1alpha1::SwapPlaintext>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapClaimView {
    #[prost(message, optional, tag="1")]
    pub swap_claim: ::core::option::Option<super::super::dex::v1alpha1::SwapClaim>,
    #[prost(message, optional, tag="2")]
    pub output_1: ::core::option::Option<super::super::crypto::v1alpha1::Note>,
    #[prost(message, optional, tag="3")]
    pub output_2: ::core::option::Option<super::super::crypto::v1alpha1::Note>,
}
/// A view of a specific state change action performed by a transaction.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActionView {
    #[prost(oneof="action_view::ActionView", tags="1, 2, 3, 4, 16, 17, 18, 19, 20, 30, 31, 32, 34, 41, 42, 43, 200")]
    pub action_view: ::core::option::Option<action_view::ActionView>,
}
/// Nested message and enum types in `ActionView`.
pub mod action_view {
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum ActionView {
        #[prost(message, tag="1")]
        Spend(super::SpendView),
        #[prost(message, tag="2")]
        Output(super::OutputView),
        #[prost(message, tag="3")]
        Swap(super::SwapView),
        #[prost(message, tag="4")]
        SwapClaim(super::SwapClaimView),
        #[prost(message, tag="16")]
        ValidatorDefinition(super::super::super::stake::v1alpha1::ValidatorDefinition),
        #[prost(message, tag="17")]
        IbcAction(super::super::super::ibc::v1alpha1::IbcAction),
        /// Governance:
        #[prost(message, tag="18")]
        ProposalSubmit(super::ProposalSubmit),
        #[prost(message, tag="19")]
        ProposalWithdraw(super::ProposalWithdraw),
        /// DelegatorVote delegator_vote = 21;
        #[prost(message, tag="20")]
        ValidatorVote(super::ValidatorVote),
        #[prost(message, tag="30")]
        PositionOpen(super::super::super::dex::v1alpha1::PositionOpen),
        #[prost(message, tag="31")]
        PositionClose(super::super::super::dex::v1alpha1::PositionClose),
        #[prost(message, tag="32")]
        PositionWithdraw(super::super::super::dex::v1alpha1::PositionWithdraw),
        #[prost(message, tag="34")]
        PositionRewardClaim(super::super::super::dex::v1alpha1::PositionRewardClaim),
        #[prost(message, tag="41")]
        Delegate(super::super::super::stake::v1alpha1::Delegate),
        #[prost(message, tag="42")]
        Undelegate(super::super::super::stake::v1alpha1::Undelegate),
        /// TODO: we have no way to recover the opening of the undelegate_claim's
        /// balance commitment, and can only infer the value from looking at the rest
        /// of the transaction. is that fine?
        #[prost(message, tag="43")]
        UndelegateClaim(super::super::super::stake::v1alpha1::UndelegateClaim),
        #[prost(message, tag="200")]
        Ics20Withdrawal(super::super::super::ibc::v1alpha1::Ics20Withdrawal),
    }
}
/// Spends a shielded note.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Spend {
    /// The authorizing data for the spend, which is included in the authorization hash used for signing.
    #[prost(message, optional, tag="1")]
    pub body: ::core::option::Option<SpendBody>,
    /// The spend authorization signature is effecting data.
    #[prost(message, optional, tag="2")]
    pub auth_sig: ::core::option::Option<super::super::crypto::v1alpha1::SpendAuthSignature>,
    /// The spend proof is effecting data.
    #[prost(bytes="bytes", tag="3")]
    #[serde(with = "crate::serializers::base64str_bytes")]
    pub proof: ::prost::bytes::Bytes,
}
/// The body of a spend description, containing only the "authorizing" data
/// included in the authorization hash used for signing, and not the "effecting"
/// data which is bound to the authorizing data by some other means.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendBody {
    /// A commitment to the value of the input note.
    #[prost(message, optional, tag="1")]
    pub balance_commitment: ::core::option::Option<super::super::crypto::v1alpha1::BalanceCommitment>,
    /// The nullifier of the input note.
    #[prost(bytes="bytes", tag="3")]
    #[serde(with = "crate::serializers::hexstr_bytes")]
    pub nullifier: ::prost::bytes::Bytes,
    /// The randomized validating key for the spend authorization signature.
    #[prost(bytes="bytes", tag="4")]
    #[serde(with = "crate::serializers::hexstr_bytes")]
    pub rk: ::prost::bytes::Bytes,
}
/// Creates a new shielded note.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Output {
    /// The authorizing data for the output.
    #[prost(message, optional, tag="1")]
    pub body: ::core::option::Option<OutputBody>,
    /// The output proof is effecting data.
    #[prost(bytes="bytes", tag="2")]
    #[serde(with = "crate::serializers::base64str_bytes")]
    pub proof: ::prost::bytes::Bytes,
}
/// The body of an output description, containing only the "authorizing" data
/// included in the authorization hash used for signing, and not the "effecting"
/// data which is bound to the authorizing data by some other means.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutputBody {
    /// The minimal data required to scan and process the new output note.
    #[prost(message, optional, tag="1")]
    pub note_payload: ::core::option::Option<super::super::crypto::v1alpha1::EncryptedNote>,
    /// A commitment to the value of the output note. 32 bytes.
    #[prost(message, optional, tag="2")]
    pub balance_commitment: ::core::option::Option<super::super::crypto::v1alpha1::BalanceCommitment>,
    /// An encrypted key for decrypting the memo.
    #[prost(bytes="bytes", tag="3")]
    #[serde(with = "crate::serializers::base64str_bytes")]
    pub wrapped_memo_key: ::prost::bytes::Bytes,
    /// The key material used for note encryption, wrapped in encryption to the
    /// sender's outgoing viewing key. 80 bytes.
    #[prost(bytes="bytes", tag="4")]
    #[serde(with = "crate::serializers::base64str_bytes")]
    pub ovk_wrapped_key: ::prost::bytes::Bytes,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalSubmit {
    /// The proposal to be submitted.
    #[prost(message, optional, tag="1")]
    pub proposal: ::core::option::Option<Proposal>,
    /// The ephemeral transparent refund address for the refund of the proposal deposit.
    #[prost(message, optional, tag="2")]
    pub deposit_refund_address: ::core::option::Option<super::super::crypto::v1alpha1::Address>,
    /// The amount of the proposal deposit.
    #[prost(message, optional, tag="3")]
    pub deposit_amount: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// The randomized proposer key (a randomization of the proposer's spend verification key).
    #[prost(bytes="bytes", tag="4")]
    pub rk: ::prost::bytes::Bytes,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalWithdraw {
    /// The body of the proposal withdraw message.
    #[prost(message, optional, tag="1")]
    pub body: ::core::option::Option<ProposalWithdrawBody>,
    /// The signature with the randomized proposer key of the withdrawal.
    #[prost(message, optional, tag="2")]
    pub auth_sig: ::core::option::Option<super::super::crypto::v1alpha1::SpendAuthSignature>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalWithdrawBody {
    /// The proposal to be withdrawn.
    #[prost(uint64, tag="1")]
    pub proposal: u64,
    /// The reason for the proposal being withdrawn.
    #[prost(string, tag="2")]
    pub reason: ::prost::alloc::string::String,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorVote {
    /// The authorizing data for the vote.
    #[prost(message, optional, tag="1")]
    pub body: ::core::option::Option<ValidatorVoteBody>,
    /// The vote authorization signature is effecting data.
    #[prost(message, optional, tag="2")]
    pub auth_sig: ::core::option::Option<super::super::crypto::v1alpha1::SpendAuthSignature>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorVoteBody {
    /// The proposal being voted on.
    #[prost(uint64, tag="1")]
    pub proposal: u64,
    /// The vote.
    #[prost(message, optional, tag="2")]
    pub vote: ::core::option::Option<super::super::governance::v1alpha1::Vote>,
    /// The validator identity.
    #[prost(message, optional, tag="3")]
    pub identity_key: ::core::option::Option<super::super::crypto::v1alpha1::IdentityKey>,
    /// The validator governance key.
    #[prost(message, optional, tag="4")]
    pub governance_key: ::core::option::Option<super::super::crypto::v1alpha1::GovernanceKey>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegatorVote {
    /// The authorizing data for the vote, which is included in the authorization hash used for signing.
    #[prost(message, optional, tag="1")]
    pub body: ::core::option::Option<DelegatorVoteBody>,
    /// The vote authorization signature is effecting data.
    #[prost(message, optional, tag="2")]
    pub auth_sig: ::core::option::Option<super::super::crypto::v1alpha1::SpendAuthSignature>,
    /// The vote proof is effecting data.
    #[prost(bytes="bytes", tag="3")]
    pub proof: ::prost::bytes::Bytes,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegatorVoteBody {
    /// The proposal being voted on.
    #[prost(uint64, tag="1")]
    pub proposal: u64,
    /// The nullifier of the input note.
    #[prost(bytes="bytes", tag="3")]
    pub nullifier: ::prost::bytes::Bytes,
    /// The randomized validating key for the spend authorization signature.
    #[prost(bytes="bytes", tag="4")]
    pub rk: ::prost::bytes::Bytes,
    /// A commitment to the value voted for "yes".
    ///
    /// A rational voter will place all their voting weight on one vote.
    #[prost(message, optional, tag="5")]
    pub yes_balance_commitment: ::core::option::Option<super::super::crypto::v1alpha1::BalanceCommitment>,
    /// A commitment to the value voted for "no".
    ///
    /// A rational voter will place all their voting weight on one vote.
    #[prost(message, optional, tag="6")]
    pub no_balance_commitment: ::core::option::Option<super::super::crypto::v1alpha1::BalanceCommitment>,
    /// A commitment to the value voted for "abstain".
    ///
    /// A rational voter will place all their voting weight on one vote.
    #[prost(message, optional, tag="7")]
    pub abstain_balance_commitment: ::core::option::Option<super::super::crypto::v1alpha1::BalanceCommitment>,
    /// A commitment to the value voted for "no with veto".
    ///
    /// A rational voter will place all their voting weight on one vote.
    #[prost(message, optional, tag="8")]
    pub no_with_veto_balance_commitment: ::core::option::Option<super::super::crypto::v1alpha1::BalanceCommitment>,
}
/// The data required to authorize a transaction plan.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthorizationData {
    /// The computed auth hash for the approved transaction plan.
    #[prost(message, optional, tag="1")]
    pub auth_hash: ::core::option::Option<AuthHash>,
    /// The required spend authorizations, returned in the same order as the
    /// Spend actions in the original request.
    #[prost(message, repeated, tag="2")]
    pub spend_auths: ::prost::alloc::vec::Vec<super::super::crypto::v1alpha1::SpendAuthSignature>,
    /// The required withdraw proposal authorizations, returned in the same order as the
    /// ProposalWithdraw actions in the original request.
    #[prost(message, repeated, tag="3")]
    pub withdraw_proposal_auths: ::prost::alloc::vec::Vec<super::super::crypto::v1alpha1::SpendAuthSignature>,
}
/// The data required for proving when building a transaction from a plan.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WitnessData {
    /// The anchor for the state transition proofs.
    #[prost(message, optional, tag="1")]
    pub anchor: ::core::option::Option<super::super::crypto::v1alpha1::MerkleRoot>,
    /// The auth paths for the notes the transaction spends, in the
    /// same order as the spends in the transaction plan.
    #[prost(message, repeated, tag="2")]
    pub note_commitment_proofs: ::prost::alloc::vec::Vec<super::super::crypto::v1alpha1::NoteCommitmentProof>,
}
/// Describes a planned transaction.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionPlan {
    #[prost(message, repeated, tag="1")]
    pub actions: ::prost::alloc::vec::Vec<ActionPlan>,
    #[prost(uint64, tag="2")]
    pub expiry_height: u64,
    #[prost(string, tag="3")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="4")]
    pub fee: ::core::option::Option<super::super::crypto::v1alpha1::Fee>,
    #[prost(message, repeated, tag="5")]
    pub clue_plans: ::prost::alloc::vec::Vec<CluePlan>,
    #[prost(message, optional, tag="6")]
    pub memo_plan: ::core::option::Option<MemoPlan>,
}
/// Describes a planned transaction action.
///
/// Some transaction Actions don't have any private data and are treated as being plans
/// themselves.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActionPlan {
    #[prost(oneof="action_plan::Action", tags="1, 2, 3, 4, 16, 17, 18, 19, 20, 21, 30, 31, 32, 34, 40, 41, 42")]
    pub action: ::core::option::Option<action_plan::Action>,
}
/// Nested message and enum types in `ActionPlan`.
pub mod action_plan {
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Action {
        #[prost(message, tag="1")]
        Spend(super::SpendPlan),
        #[prost(message, tag="2")]
        Output(super::OutputPlan),
        #[prost(message, tag="3")]
        Swap(super::SwapPlan),
        #[prost(message, tag="4")]
        SwapClaim(super::SwapClaimPlan),
        /// This is just a message relayed to the chain.
        #[prost(message, tag="16")]
        ValidatorDefinition(super::super::super::stake::v1alpha1::ValidatorDefinition),
        /// This is just a message relayed to the chain.
        #[prost(message, tag="17")]
        IbcAction(super::super::super::ibc::v1alpha1::IbcAction),
        /// Governance:
        #[prost(message, tag="18")]
        ProposalSubmit(super::ProposalSubmit),
        #[prost(message, tag="19")]
        ProposalWithdraw(super::ProposalWithdrawPlan),
        #[prost(message, tag="20")]
        ValidatorVote(super::ValidatorVote),
        #[prost(message, tag="21")]
        DelegatorVote(super::DelegatorVotePlan),
        #[prost(message, tag="30")]
        PositionOpen(super::super::super::dex::v1alpha1::PositionOpen),
        #[prost(message, tag="31")]
        PositionClose(super::super::super::dex::v1alpha1::PositionClose),
        #[prost(message, tag="32")]
        PositionWithdraw(super::super::super::dex::v1alpha1::PositionWithdraw),
        #[prost(message, tag="34")]
        PositionRewardClaim(super::super::super::dex::v1alpha1::PositionRewardClaim),
        /// We don't need any extra information (yet) to understand delegations,
        /// because we don't yet use flow encryption.
        #[prost(message, tag="40")]
        Delegate(super::super::super::stake::v1alpha1::Delegate),
        /// We don't need any extra information (yet) to understand undelegations,
        /// because we don't yet use flow encryption.
        #[prost(message, tag="41")]
        Undelegate(super::super::super::stake::v1alpha1::Undelegate),
        #[prost(message, tag="42")]
        UndelegateClaim(super::super::super::stake::v1alpha1::UndelegateClaimPlan),
    }
}
/// Describes a plan for forming a `Clue`.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CluePlan {
    /// The address.
    #[prost(message, optional, tag="1")]
    pub address: ::core::option::Option<super::super::crypto::v1alpha1::Address>,
    /// The random seed to use for the clue plan.
    #[prost(bytes="bytes", tag="2")]
    pub rseed: ::prost::bytes::Bytes,
    /// The bits of precision.
    #[prost(uint64, tag="3")]
    pub precision_bits: u64,
}
/// Describes a plan for forming a `Memo`.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MemoPlan {
    /// The plaintext.
    #[prost(bytes="bytes", tag="1")]
    pub plaintext: ::prost::bytes::Bytes,
    /// The key to use to encrypt the memo.
    #[prost(bytes="bytes", tag="2")]
    pub key: ::prost::bytes::Bytes,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendPlan {
    /// The plaintext note we plan to spend.
    #[prost(message, optional, tag="1")]
    pub note: ::core::option::Option<super::super::crypto::v1alpha1::Note>,
    /// The position of the note we plan to spend.
    #[prost(uint64, tag="2")]
    pub position: u64,
    /// The randomizer to use for the spend.
    #[prost(bytes="bytes", tag="3")]
    #[serde(with = "crate::serializers::hexstr_bytes")]
    pub randomizer: ::prost::bytes::Bytes,
    /// The blinding factor to use for the value commitment.
    #[prost(bytes="bytes", tag="4")]
    #[serde(with = "crate::serializers::hexstr_bytes")]
    pub value_blinding: ::prost::bytes::Bytes,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutputPlan {
    /// The value to send to this output.
    #[prost(message, optional, tag="1")]
    pub value: ::core::option::Option<super::super::crypto::v1alpha1::Value>,
    /// The destination address to send it to.
    #[prost(message, optional, tag="2")]
    pub dest_address: ::core::option::Option<super::super::crypto::v1alpha1::Address>,
    /// The blinding factor to use for the new note.
    #[prost(bytes="bytes", tag="3")]
    #[serde(with = "crate::serializers::hexstr_bytes")]
    pub note_blinding: ::prost::bytes::Bytes,
    /// The blinding factor to use for the value commitment.
    #[prost(bytes="bytes", tag="4")]
    #[serde(with = "crate::serializers::hexstr_bytes")]
    pub value_blinding: ::prost::bytes::Bytes,
    /// The ephemeral secret key to use for the note encryption.
    #[prost(bytes="bytes", tag="5")]
    #[serde(with = "crate::serializers::hexstr_bytes")]
    pub esk: ::prost::bytes::Bytes,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapPlan {
    /// The plaintext version of the swap to be performed.
    #[prost(message, optional, tag="1")]
    pub swap_plaintext: ::core::option::Option<super::super::dex::v1alpha1::SwapPlaintext>,
    /// The blinding factor for the fee commitment. The fee in the SwapPlan is private to prevent linkability with the SwapClaim.
    #[prost(bytes="bytes", tag="5")]
    #[serde(with = "crate::serializers::hexstr_bytes")]
    pub fee_blinding: ::prost::bytes::Bytes,
    /// The blinding factor to use for the new swap NFT note.
    #[prost(bytes="bytes", tag="7")]
    #[serde(with = "crate::serializers::hexstr_bytes")]
    pub note_blinding: ::prost::bytes::Bytes,
    /// The ephemeral secret key to use for the swap NFT note encryption.
    #[prost(bytes="bytes", tag="8")]
    #[serde(with = "crate::serializers::hexstr_bytes")]
    pub esk: ::prost::bytes::Bytes,
}
///
/// @exclude
/// Fields describing the swap NFT note to be redeemed.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapClaimPlan {
    /// The input swap NFT note to be spent.
    #[prost(message, optional, tag="1")]
    pub swap_nft_note: ::core::option::Option<super::super::crypto::v1alpha1::Note>,
    /// The position of the input swap NFT note.
    #[prost(uint64, tag="2")]
    pub swap_nft_position: u64,
    /// The plaintext version of the swap to be performed.
    #[prost(message, optional, tag="3")]
    pub swap_plaintext: ::core::option::Option<super::super::dex::v1alpha1::SwapPlaintext>,
    /// Input and output amounts for the Swap.
    #[prost(message, optional, tag="11")]
    pub output_data: ::core::option::Option<super::super::dex::v1alpha1::BatchSwapOutputData>,
    /// The ephemeral secret key used for the first output note encryption.
    #[prost(bytes="bytes", tag="17")]
    #[serde(with = "crate::serializers::hexstr_bytes")]
    pub esk_1: ::prost::bytes::Bytes,
    /// The ephemeral secret key used for the second output note encryption.
    #[prost(bytes="bytes", tag="18")]
    #[serde(with = "crate::serializers::hexstr_bytes")]
    pub esk_2: ::prost::bytes::Bytes,
    /// The epoch duration when the swap claim took place.
    #[prost(uint64, tag="20")]
    pub epoch_duration: u64,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalWithdrawPlan {
    /// The body of the proposal withdrawal.
    #[prost(message, optional, tag="1")]
    pub body: ::core::option::Option<ProposalWithdrawBody>,
    /// The randomizer to use for signing the proposal withdrawal.
    #[prost(bytes="bytes", tag="2")]
    pub randomizer: ::prost::bytes::Bytes,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegatorVotePlan {
    /// The proposal to vote on.
    #[prost(uint64, tag="1")]
    pub proposal: u64,
    /// The vote to cast.
    #[prost(message, optional, tag="2")]
    pub vote: ::core::option::Option<super::super::governance::v1alpha1::Vote>,
    /// The delegation note to prove that we can vote.
    #[prost(message, optional, tag="3")]
    pub staked_note: ::core::option::Option<super::super::crypto::v1alpha1::Note>,
    /// The position of that delegation note.
    #[prost(uint64, tag="4")]
    pub position: u64,
    /// The randomizer to use for the proof of spend capability.
    #[prost(bytes="bytes", tag="5")]
    pub randomizer: ::prost::bytes::Bytes,
}
// The reader may ask: why is this here, instead of in a separate `governance.v1alpha1.proto` file? It should
// be, but protos can't have cyclic file dependencies, and most of the below induces a cycle
// because `DaoSpend` contains a `TransactionPlan`.

/// A proposal to be voted upon.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proposal {
    #[prost(string, tag="1")]
    pub title: ::prost::alloc::string::String,
    /// A natural-language description of the effect of the proposal and its justification.
    #[prost(string, tag="2")]
    pub description: ::prost::alloc::string::String,
    /// The payload of the proposal.
    #[prost(message, optional, tag="3")]
    #[serde(flatten)]
    pub payload: ::core::option::Option<proposal::Payload>,
}
/// Nested message and enum types in `Proposal`.
pub mod proposal {
    /// The kind of the proposal and details relevant only to that kind of proposal.
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[serde(rename_all = "snake_case")]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Payload {
        #[prost(oneof="payload::Payload", tags="2, 3, 4, 5")]
        #[serde(flatten)]
        pub payload: ::core::option::Option<payload::Payload>,
    }
    /// Nested message and enum types in `Payload`.
    pub mod payload {
        #[derive(::serde::Deserialize, ::serde::Serialize)]
        #[serde(rename_all = "snake_case")]
        #[serde(tag = "kind")]
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Payload {
            /// A signaling proposal.
            #[prost(message, tag="2")]
            Signaling(super::Signaling),
            /// An emergency proposal.
            #[prost(message, tag="3")]
            Emergency(super::Emergency),
            /// A parameter change proposal.
            #[prost(message, tag="4")]
            ParameterChange(super::ParameterChange),
            /// A DAO spend proposal.
            #[prost(message, tag="5")]
            DaoSpend(super::DaoSpend),
        }
    }
    // A signaling proposal is meant to register a vote on-chain, but does not have an automatic
    // effect when passed.

    /// It optionally contains a reference to a commit which contains code to upgrade the chain.
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Signaling {
        /// The commit to be voted upon, if any is relevant.
        #[prost(string, optional, tag="1")]
        pub commit: ::core::option::Option<::prost::alloc::string::String>,
    }
    /// An emergency proposal can be passed instantaneously by a 2/3 majority of validators, without
    /// waiting for the voting period to expire.
    ///
    /// If the boolean `halt_chain` is set to `true`, then the chain will halt immediately when the
    /// proposal is passed.
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Emergency {
        /// If `true`, the chain will halt immediately when the proposal is passed.
        #[prost(bool, tag="1")]
        pub halt_chain: bool,
    }
    /// A parameter change proposal describes an alteration to one or more chain parameters, which
    /// should take effect at a particular block height `effective_height` (which should be at least
    /// the height when the proposal would be passed).
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ParameterChange {
        /// The height at which the change should take effect.
        #[prost(uint64, tag="1")]
        pub effective_height: u64,
        /// The set of changes to chain parameters.
        #[prost(message, repeated, tag="2")]
        pub new_parameters: ::prost::alloc::vec::Vec<parameter_change::SetParameter>,
    }
    /// Nested message and enum types in `ParameterChange`.
    pub mod parameter_change {
        /// A single change to an individual chain parameter.
        #[derive(::serde::Deserialize, ::serde::Serialize)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct SetParameter {
            /// The name of the parameter.
            #[prost(string, tag="1")]
            pub parameter: ::prost::alloc::string::String,
            /// Its new value, as a string (this will be parsed as appropriate for the parameter's type).
            #[prost(string, tag="2")]
            pub value: ::prost::alloc::string::String,
        }
    }
    /// A DAO spend proposal describes zero or more transactions to execute on behalf of the DAO, with
    /// access to its funds, and zero or more scheduled transactions from previous passed proposals to
    /// cancel.
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct DaoSpend {
        /// The sequence of transactions to schedule for execution.
        #[prost(message, repeated, tag="1")]
        pub schedule_transactions: ::prost::alloc::vec::Vec<dao_spend::ScheduleTransaction>,
        /// A sequence of previously-scheduled transactions to cancel before they are executed.
        #[prost(message, repeated, tag="2")]
        pub cancel_transactions: ::prost::alloc::vec::Vec<dao_spend::CancelTransaction>,
    }
    /// Nested message and enum types in `DaoSpend`.
    pub mod dao_spend {
        /// A transaction to be executed as a consequence of this proposal.
        ///
        /// It is permissible for there to be duplicate transactions scheduled for a given height; they
        /// will both be executed.
        #[derive(::serde::Deserialize, ::serde::Serialize)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct ScheduleTransaction {
            /// The height at which the transaction should be executed.
            #[prost(uint64, tag="1")]
            pub execute_at_height: u64,
            /// The transaction to be executed.
            #[prost(message, optional, tag="2")]
            pub transaction: ::core::option::Option<super::super::TransactionPlan>,
        }
        /// A transaction to be canceled as a consequence of this proposal.
        ///
        /// If there are multiple duplicate transactions at the height, this cancels only the first.
        /// To cancel more of them, specify duplicate cancellations.
        #[derive(::serde::Deserialize, ::serde::Serialize)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct CancelTransaction {
            /// The height for which the transaction was scheduled.
            #[prost(uint64, tag="1")]
            pub scheduled_at_height: u64,
            /// The auth hash of the transaction to cancel.
            #[prost(message, optional, tag="2")]
            pub auth_hash: ::core::option::Option<super::super::AuthHash>,
        }
    }
}
