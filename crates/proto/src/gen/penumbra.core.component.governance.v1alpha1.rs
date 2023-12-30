/// A Penumbra ZK delegator vote proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkDelegatorVoteProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ZkDelegatorVoteProof {
    const NAME: &'static str = "ZKDelegatorVoteProof";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalSubmit {
    /// The proposal to be submitted.
    #[prost(message, optional, tag = "1")]
    pub proposal: ::core::option::Option<Proposal>,
    /// The amount of the proposal deposit.
    #[prost(message, optional, tag = "3")]
    pub deposit_amount: ::core::option::Option<
        super::super::super::num::v1alpha1::Amount,
    >,
}
impl ::prost::Name for ProposalSubmit {
    const NAME: &'static str = "ProposalSubmit";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalWithdraw {
    /// The proposal to be withdrawn.
    #[prost(uint64, tag = "1")]
    pub proposal: u64,
    /// The reason for the proposal being withdrawn.
    #[prost(string, tag = "2")]
    pub reason: ::prost::alloc::string::String,
}
impl ::prost::Name for ProposalWithdraw {
    const NAME: &'static str = "ProposalWithdraw";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalDepositClaim {
    /// The proposal to claim the deposit for.
    #[prost(uint64, tag = "1")]
    pub proposal: u64,
    /// The expected deposit amount.
    #[prost(message, optional, tag = "2")]
    pub deposit_amount: ::core::option::Option<
        super::super::super::num::v1alpha1::Amount,
    >,
    /// The outcome of the proposal.
    #[prost(message, optional, tag = "3")]
    pub outcome: ::core::option::Option<ProposalOutcome>,
}
impl ::prost::Name for ProposalDepositClaim {
    const NAME: &'static str = "ProposalDepositClaim";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorVote {
    /// The effecting data for the vote.
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<ValidatorVoteBody>,
    /// The vote authorization signature is authorizing data.
    #[prost(message, optional, tag = "2")]
    pub auth_sig: ::core::option::Option<
        super::super::super::super::crypto::decaf377_rdsa::v1alpha1::SpendAuthSignature,
    >,
}
impl ::prost::Name for ValidatorVote {
    const NAME: &'static str = "ValidatorVote";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorVoteReason {
    #[prost(string, tag = "1")]
    pub reason: ::prost::alloc::string::String,
}
impl ::prost::Name for ValidatorVoteReason {
    const NAME: &'static str = "ValidatorVoteReason";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorVoteBody {
    /// The proposal being voted on.
    #[prost(uint64, tag = "1")]
    pub proposal: u64,
    /// The vote.
    #[prost(message, optional, tag = "2")]
    pub vote: ::core::option::Option<Vote>,
    /// The validator identity.
    #[prost(message, optional, tag = "3")]
    pub identity_key: ::core::option::Option<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
    /// The validator governance key.
    #[prost(message, optional, tag = "4")]
    pub governance_key: ::core::option::Option<
        super::super::super::keys::v1alpha1::GovernanceKey,
    >,
    /// A justification of the vote.
    #[prost(message, optional, tag = "5")]
    pub reason: ::core::option::Option<ValidatorVoteReason>,
}
impl ::prost::Name for ValidatorVoteBody {
    const NAME: &'static str = "ValidatorVoteBody";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegatorVote {
    /// The effecting data for the vote.
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<DelegatorVoteBody>,
    /// The vote authorization signature is authorizing data.
    #[prost(message, optional, tag = "2")]
    pub auth_sig: ::core::option::Option<
        super::super::super::super::crypto::decaf377_rdsa::v1alpha1::SpendAuthSignature,
    >,
    /// The vote proof is authorizing data.
    #[prost(message, optional, tag = "3")]
    pub proof: ::core::option::Option<ZkDelegatorVoteProof>,
}
impl ::prost::Name for DelegatorVote {
    const NAME: &'static str = "DelegatorVote";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegatorVoteBody {
    /// The proposal being voted on.
    #[prost(uint64, tag = "1")]
    pub proposal: u64,
    /// The start position of the proposal in the TCT.
    #[prost(uint64, tag = "2")]
    pub start_position: u64,
    /// The vote.
    #[prost(message, optional, tag = "3")]
    pub vote: ::core::option::Option<Vote>,
    /// The value of the delegation note.
    #[prost(message, optional, tag = "4")]
    pub value: ::core::option::Option<super::super::super::asset::v1alpha1::Value>,
    /// The amount of the delegation note, in unbonded penumbra.
    #[prost(message, optional, tag = "5")]
    pub unbonded_amount: ::core::option::Option<
        super::super::super::num::v1alpha1::Amount,
    >,
    /// The nullifier of the input note.
    #[prost(message, optional, tag = "6")]
    pub nullifier: ::core::option::Option<super::super::sct::v1alpha1::Nullifier>,
    /// The randomized validating key for the spend authorization signature.
    #[prost(message, optional, tag = "7")]
    pub rk: ::core::option::Option<
        super::super::super::super::crypto::decaf377_rdsa::v1alpha1::SpendVerificationKey,
    >,
}
impl ::prost::Name for DelegatorVoteBody {
    const NAME: &'static str = "DelegatorVoteBody";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegatorVoteView {
    #[prost(oneof = "delegator_vote_view::DelegatorVote", tags = "1, 2")]
    pub delegator_vote: ::core::option::Option<delegator_vote_view::DelegatorVote>,
}
/// Nested message and enum types in `DelegatorVoteView`.
pub mod delegator_vote_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Visible {
        #[prost(message, optional, tag = "1")]
        pub delegator_vote: ::core::option::Option<super::DelegatorVote>,
        #[prost(message, optional, tag = "2")]
        pub note: ::core::option::Option<
            super::super::super::shielded_pool::v1alpha1::NoteView,
        >,
    }
    impl ::prost::Name for Visible {
        const NAME: &'static str = "Visible";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.DelegatorVoteView.{}",
                Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub delegator_vote: ::core::option::Option<super::DelegatorVote>,
    }
    impl ::prost::Name for Opaque {
        const NAME: &'static str = "Opaque";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.DelegatorVoteView.{}",
                Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum DelegatorVote {
        #[prost(message, tag = "1")]
        Visible(Visible),
        #[prost(message, tag = "2")]
        Opaque(Opaque),
    }
}
impl ::prost::Name for DelegatorVoteView {
    const NAME: &'static str = "DelegatorVoteView";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegatorVotePlan {
    /// The proposal to vote on.
    #[prost(uint64, tag = "1")]
    pub proposal: u64,
    /// The start position of the proposal in the TCT.
    #[prost(uint64, tag = "2")]
    pub start_position: u64,
    /// The vote to cast.
    #[prost(message, optional, tag = "3")]
    pub vote: ::core::option::Option<Vote>,
    /// The delegation note to prove that we can vote.
    #[prost(message, optional, tag = "4")]
    pub staked_note: ::core::option::Option<super::super::shielded_pool::v1alpha1::Note>,
    /// The position of that delegation note.
    #[prost(uint64, tag = "5")]
    pub staked_note_position: u64,
    /// The unbonded amount equivalent to the delegation note.
    #[prost(message, optional, tag = "6")]
    pub unbonded_amount: ::core::option::Option<
        super::super::super::num::v1alpha1::Amount,
    >,
    /// The randomizer to use for the proof of spend capability.
    #[prost(bytes = "vec", tag = "7")]
    pub randomizer: ::prost::alloc::vec::Vec<u8>,
    /// The first blinding factor to use for the ZK delegator vote proof.
    #[prost(bytes = "vec", tag = "8")]
    pub proof_blinding_r: ::prost::alloc::vec::Vec<u8>,
    /// The second blinding factor to use for the ZK delegator vote proof.
    #[prost(bytes = "vec", tag = "9")]
    pub proof_blinding_s: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for DelegatorVotePlan {
    const NAME: &'static str = "DelegatorVotePlan";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommunityPoolDeposit {
    /// The value to deposit into the Community Pool.
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::super::asset::v1alpha1::Value>,
}
impl ::prost::Name for CommunityPoolDeposit {
    const NAME: &'static str = "CommunityPoolDeposit";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommunityPoolSpend {
    /// The value to spend from the Community Pool.
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::super::asset::v1alpha1::Value>,
}
impl ::prost::Name for CommunityPoolSpend {
    const NAME: &'static str = "CommunityPoolSpend";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommunityPoolOutput {
    /// The value to output from the Community Pool.
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::super::asset::v1alpha1::Value>,
    /// The address to send the output to.
    #[prost(message, optional, tag = "2")]
    pub address: ::core::option::Option<super::super::super::keys::v1alpha1::Address>,
}
impl ::prost::Name for CommunityPoolOutput {
    const NAME: &'static str = "CommunityPoolOutput";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// A vote on a proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vote {
    /// The vote.
    #[prost(enumeration = "vote::Vote", tag = "1")]
    pub vote: i32,
}
/// Nested message and enum types in `Vote`.
pub mod vote {
    /// A vote.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Vote {
        Unspecified = 0,
        Abstain = 1,
        Yes = 2,
        No = 3,
    }
    impl Vote {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Vote::Unspecified => "VOTE_UNSPECIFIED",
                Vote::Abstain => "VOTE_ABSTAIN",
                Vote::Yes => "VOTE_YES",
                Vote::No => "VOTE_NO",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "VOTE_UNSPECIFIED" => Some(Self::Unspecified),
                "VOTE_ABSTAIN" => Some(Self::Abstain),
                "VOTE_YES" => Some(Self::Yes),
                "VOTE_NO" => Some(Self::No),
                _ => None,
            }
        }
    }
}
impl ::prost::Name for Vote {
    const NAME: &'static str = "Vote";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// The current state of a proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalState {
    /// The state of the proposal.
    #[prost(oneof = "proposal_state::State", tags = "2, 3, 4, 5")]
    pub state: ::core::option::Option<proposal_state::State>,
}
/// Nested message and enum types in `ProposalState`.
pub mod proposal_state {
    /// Voting is in progress and the proposal has not yet concluded voting or been withdrawn.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Voting {}
    impl ::prost::Name for Voting {
        const NAME: &'static str = "Voting";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.ProposalState.{}",
                Self::NAME
            )
        }
    }
    /// The proposal has been withdrawn but the voting period is not yet concluded.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Withdrawn {
        /// The reason for the withdrawal.
        #[prost(string, tag = "1")]
        pub reason: ::prost::alloc::string::String,
    }
    impl ::prost::Name for Withdrawn {
        const NAME: &'static str = "Withdrawn";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.ProposalState.{}",
                Self::NAME
            )
        }
    }
    /// The voting period has ended, and the proposal has been assigned an outcome.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Finished {
        #[prost(message, optional, tag = "1")]
        pub outcome: ::core::option::Option<super::ProposalOutcome>,
    }
    impl ::prost::Name for Finished {
        const NAME: &'static str = "Finished";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.ProposalState.{}",
                Self::NAME
            )
        }
    }
    /// The voting period has ended, and the original proposer has claimed their deposit.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Claimed {
        #[prost(message, optional, tag = "1")]
        pub outcome: ::core::option::Option<super::ProposalOutcome>,
    }
    impl ::prost::Name for Claimed {
        const NAME: &'static str = "Claimed";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.ProposalState.{}",
                Self::NAME
            )
        }
    }
    /// The state of the proposal.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum State {
        #[prost(message, tag = "2")]
        Voting(Voting),
        #[prost(message, tag = "3")]
        Withdrawn(Withdrawn),
        #[prost(message, tag = "4")]
        Finished(Finished),
        #[prost(message, tag = "5")]
        Claimed(Claimed),
    }
}
impl ::prost::Name for ProposalState {
    const NAME: &'static str = "ProposalState";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// The outcome of a concluded proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalOutcome {
    #[prost(oneof = "proposal_outcome::Outcome", tags = "1, 2, 3")]
    pub outcome: ::core::option::Option<proposal_outcome::Outcome>,
}
/// Nested message and enum types in `ProposalOutcome`.
pub mod proposal_outcome {
    /// Whether or not the proposal was withdrawn.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Withdrawn {
        /// The reason for withdrawing the proposal during the voting period.
        #[prost(string, tag = "1")]
        pub reason: ::prost::alloc::string::String,
    }
    impl ::prost::Name for Withdrawn {
        const NAME: &'static str = "Withdrawn";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.ProposalOutcome.{}",
                Self::NAME
            )
        }
    }
    /// The proposal was passed.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Passed {}
    impl ::prost::Name for Passed {
        const NAME: &'static str = "Passed";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.ProposalOutcome.{}",
                Self::NAME
            )
        }
    }
    /// The proposal did not pass.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Failed {
        /// Present if the proposal was withdrawn during the voting period.
        #[prost(message, optional, tag = "1")]
        pub withdrawn: ::core::option::Option<Withdrawn>,
    }
    impl ::prost::Name for Failed {
        const NAME: &'static str = "Failed";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.ProposalOutcome.{}",
                Self::NAME
            )
        }
    }
    /// The proposal did not pass, and was slashed.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Slashed {
        /// Present if the proposal was withdrawn during the voting period.
        #[prost(message, optional, tag = "1")]
        pub withdrawn: ::core::option::Option<Withdrawn>,
    }
    impl ::prost::Name for Slashed {
        const NAME: &'static str = "Slashed";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.ProposalOutcome.{}",
                Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Outcome {
        #[prost(message, tag = "1")]
        Passed(Passed),
        #[prost(message, tag = "2")]
        Failed(Failed),
        #[prost(message, tag = "3")]
        Slashed(Slashed),
    }
}
impl ::prost::Name for ProposalOutcome {
    const NAME: &'static str = "ProposalOutcome";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// A tally of votes on a proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tally {
    /// The number of votes in favor of the proposal.
    #[prost(uint64, tag = "1")]
    pub yes: u64,
    /// The number of votes against the proposal.
    #[prost(uint64, tag = "2")]
    pub no: u64,
    /// The number of abstentions.
    #[prost(uint64, tag = "3")]
    pub abstain: u64,
}
impl ::prost::Name for Tally {
    const NAME: &'static str = "Tally";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// A proposal to be voted upon.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proposal {
    /// The unique identifier of the proposal.
    #[prost(uint64, tag = "4")]
    pub id: u64,
    /// A short title for the proposal.
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// A natural-language description of the effect of the proposal and its justification.
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// The different kinds of proposal. Only one of these should be set.
    #[prost(message, optional, tag = "5")]
    pub signaling: ::core::option::Option<proposal::Signaling>,
    #[prost(message, optional, tag = "6")]
    pub emergency: ::core::option::Option<proposal::Emergency>,
    #[prost(message, optional, tag = "7")]
    pub parameter_change: ::core::option::Option<proposal::ParameterChange>,
    #[prost(message, optional, tag = "8")]
    pub community_pool_spend: ::core::option::Option<proposal::CommunityPoolSpend>,
    #[prost(message, optional, tag = "9")]
    pub upgrade_plan: ::core::option::Option<proposal::UpgradePlan>,
}
/// Nested message and enum types in `Proposal`.
pub mod proposal {
    /// A signaling proposal is meant to register a vote on-chain, but does not have an automatic
    /// effect when passed.
    ///
    /// It optionally contains a reference to a commit which contains code to upgrade the chain.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Signaling {
        /// The commit to be voted upon, if any is relevant.
        #[prost(string, tag = "1")]
        pub commit: ::prost::alloc::string::String,
    }
    impl ::prost::Name for Signaling {
        const NAME: &'static str = "Signaling";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.Proposal.{}", Self::NAME
            )
        }
    }
    /// An emergency proposal can be passed instantaneously by a 2/3 majority of validators, without
    /// waiting for the voting period to expire.
    ///
    /// If the boolean `halt_chain` is set to `true`, then the chain will halt immediately when the
    /// proposal is passed.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Emergency {
        /// If `true`, the chain will halt immediately when the proposal is passed.
        #[prost(bool, tag = "1")]
        pub halt_chain: bool,
    }
    impl ::prost::Name for Emergency {
        const NAME: &'static str = "Emergency";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.Proposal.{}", Self::NAME
            )
        }
    }
    /// A parameter change proposal describes a replacement of the app parameters, which should take
    /// effect when the proposal is passed.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ParameterChange {
        /// The old app parameters to be replaced: even if the proposal passes, the update will not be
        /// applied if the app parameters have changed *at all* from these app parameters. Usually,
        /// this should be set to the current app parameters at time of proposal.
        #[prost(message, optional, tag = "1")]
        pub old_parameters: ::core::option::Option<super::ChangedAppParameters>,
        /// The new app parameters to be set: the *entire* app parameters will be replaced with these
        /// at the time the proposal is passed, for every component's parameters that is set. If a component's
        /// parameters are not set, then they were not changed by the proposal, and will not be updated.
        #[prost(message, optional, tag = "2")]
        pub new_parameters: ::core::option::Option<super::ChangedAppParameters>,
    }
    impl ::prost::Name for ParameterChange {
        const NAME: &'static str = "ParameterChange";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.Proposal.{}", Self::NAME
            )
        }
    }
    /// A Community Pool spend proposal describes zero or more transactions to execute on behalf of the Community Pool, with
    /// access to its funds, and zero or more scheduled transactions from previous passed proposals to
    /// cancel.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CommunityPoolSpend {
        /// The transaction plan to be executed at the time the proposal is passed. This must be a
        /// transaction plan which can be executed by the Community Pool, which means it can't require any witness
        /// data or authorization signatures, but it may use the `CommunityPoolSpend` action.
        #[prost(message, optional, tag = "2")]
        pub transaction_plan: ::core::option::Option<::pbjson_types::Any>,
    }
    impl ::prost::Name for CommunityPoolSpend {
        const NAME: &'static str = "CommunityPoolSpend";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.Proposal.{}", Self::NAME
            )
        }
    }
    /// An upgrade plan describes a candidate upgrade to be executed at a certain height. If passed, the chain
    /// will halt at the specified height.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct UpgradePlan {
        #[prost(uint64, tag = "1")]
        pub height: u64,
    }
    impl ::prost::Name for UpgradePlan {
        const NAME: &'static str = "UpgradePlan";
        const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.governance.v1alpha1.Proposal.{}", Self::NAME
            )
        }
    }
}
impl ::prost::Name for Proposal {
    const NAME: &'static str = "Proposal";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalInfoRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The proposal id to request information on.
    #[prost(uint64, tag = "2")]
    pub proposal_id: u64,
}
impl ::prost::Name for ProposalInfoRequest {
    const NAME: &'static str = "ProposalInfoRequest";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalInfoResponse {
    /// The block height at which the proposal started voting.
    #[prost(uint64, tag = "1")]
    pub start_block_height: u64,
    /// The position of the state commitment tree at which the proposal is considered to have started voting.
    #[prost(uint64, tag = "2")]
    pub start_position: u64,
}
impl ::prost::Name for ProposalInfoResponse {
    const NAME: &'static str = "ProposalInfoResponse";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalDataRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The proposal id to request information on.
    #[prost(uint64, tag = "2")]
    pub proposal_id: u64,
}
impl ::prost::Name for ProposalDataRequest {
    const NAME: &'static str = "ProposalDataRequest";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalDataResponse {
    /// The proposal metadata.
    #[prost(message, optional, tag = "1")]
    pub proposal: ::core::option::Option<Proposal>,
    /// The block height at which the proposal started voting.
    #[prost(uint64, tag = "2")]
    pub start_block_height: u64,
    /// The block height at which the proposal ends voting.
    #[prost(uint64, tag = "3")]
    pub end_block_height: u64,
    /// The position of the state commitment tree at which the proposal is considered to have started voting.
    #[prost(uint64, tag = "4")]
    pub start_position: u64,
    /// The current state of the proposal.
    #[prost(message, optional, tag = "5")]
    pub state: ::core::option::Option<ProposalState>,
    /// The deposit amount paid for the proposal.
    #[prost(message, optional, tag = "6")]
    pub proposal_deposit_amount: ::core::option::Option<
        super::super::super::num::v1alpha1::Amount,
    >,
}
impl ::prost::Name for ProposalDataResponse {
    const NAME: &'static str = "ProposalDataResponse";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// Requests the validator rate data for a proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalRateDataRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The proposal id to request information on.
    #[prost(uint64, tag = "2")]
    pub proposal_id: u64,
}
impl ::prost::Name for ProposalRateDataRequest {
    const NAME: &'static str = "ProposalRateDataRequest";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// The rate data for a single validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalRateDataResponse {
    #[prost(message, optional, tag = "1")]
    pub rate_data: ::core::option::Option<super::super::stake::v1alpha1::RateData>,
}
impl ::prost::Name for ProposalRateDataResponse {
    const NAME: &'static str = "ProposalRateDataResponse";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// Requests the list of all proposals.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalListRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// Whether to include proposals that are no longer active.;
    ///
    /// TODO: we could filter by starting block height here?
    #[prost(bool, tag = "2")]
    pub inactive: bool,
}
impl ::prost::Name for ProposalListRequest {
    const NAME: &'static str = "ProposalListRequest";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// The data for a single proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalListResponse {
    /// The proposal metadata.
    #[prost(message, optional, tag = "1")]
    pub proposal: ::core::option::Option<Proposal>,
    /// The block height at which the proposal started voting.
    #[prost(uint64, tag = "2")]
    pub start_block_height: u64,
    /// The block height at which the proposal ends voting.
    #[prost(uint64, tag = "3")]
    pub end_block_height: u64,
    /// The position of the state commitment tree at which the proposal is considered to have started voting.
    #[prost(uint64, tag = "4")]
    pub start_position: u64,
    /// The current state of the proposal.
    #[prost(message, optional, tag = "5")]
    pub state: ::core::option::Option<ProposalState>,
}
impl ::prost::Name for ProposalListResponse {
    const NAME: &'static str = "ProposalListResponse";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// Requests the list of all validator votes for a given proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorVotesRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The proposal id to request information on.
    #[prost(uint64, tag = "2")]
    pub proposal_id: u64,
}
impl ::prost::Name for ValidatorVotesRequest {
    const NAME: &'static str = "ValidatorVotesRequest";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// The data for a single validator vote.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorVotesResponse {
    /// The vote.
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<Vote>,
    /// The validator identity.
    #[prost(message, optional, tag = "2")]
    pub identity_key: ::core::option::Option<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
}
impl ::prost::Name for ValidatorVotesResponse {
    const NAME: &'static str = "ValidatorVotesResponse";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// Governance configuration data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GovernanceParameters {
    /// The number of blocks during which a proposal is voted on.
    #[prost(uint64, tag = "1")]
    pub proposal_voting_blocks: u64,
    /// The deposit required to create a proposal.
    #[prost(message, optional, tag = "2")]
    pub proposal_deposit_amount: ::core::option::Option<
        super::super::super::num::v1alpha1::Amount,
    >,
    /// The quorum required for a proposal to be considered valid, as a fraction of the total stake
    /// weight of the network.
    #[prost(string, tag = "3")]
    pub proposal_valid_quorum: ::prost::alloc::string::String,
    /// The threshold for a proposal to pass voting, as a ratio of "yes" votes over "no" votes.
    #[prost(string, tag = "4")]
    pub proposal_pass_threshold: ::prost::alloc::string::String,
    /// The threshold for a proposal to be slashed, regardless of whether the "yes" and "no" votes
    /// would have passed it, as a ratio of "no" votes over all total votes.
    #[prost(string, tag = "5")]
    pub proposal_slash_threshold: ::prost::alloc::string::String,
}
impl ::prost::Name for GovernanceParameters {
    const NAME: &'static str = "GovernanceParameters";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// Governance genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContent {
    /// Governance parameters.
    #[prost(message, optional, tag = "1")]
    pub governance_params: ::core::option::Option<GovernanceParameters>,
}
impl ::prost::Name for GenesisContent {
    const NAME: &'static str = "GenesisContent";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// Note: must be kept in sync with AppParameters.
/// Each field here is optional.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChangedAppParameters {
    /// Chain module parameters.
    #[prost(message, optional, tag = "1")]
    pub chain_params: ::core::option::Option<
        super::super::chain::v1alpha1::ChainParameters,
    >,
    /// Community Pool module parameters.
    #[prost(message, optional, tag = "2")]
    pub community_pool_params: ::core::option::Option<
        super::super::community_pool::v1alpha1::CommunityPoolParameters,
    >,
    /// Governance module parameters.
    #[prost(message, optional, tag = "3")]
    pub governance_params: ::core::option::Option<GovernanceParameters>,
    /// IBC module parameters.
    #[prost(message, optional, tag = "4")]
    pub ibc_params: ::core::option::Option<super::super::ibc::v1alpha1::IbcParameters>,
    /// Stake module parameters.
    #[prost(message, optional, tag = "5")]
    pub stake_params: ::core::option::Option<
        super::super::stake::v1alpha1::StakeParameters,
    >,
    /// Fee module parameters.
    #[prost(message, optional, tag = "6")]
    pub fee_params: ::core::option::Option<super::super::fee::v1alpha1::FeeParameters>,
    /// Distributions module parameters.
    #[prost(message, optional, tag = "7")]
    pub distributions_params: ::core::option::Option<
        super::super::distributions::v1alpha1::DistributionsParameters,
    >,
}
impl ::prost::Name for ChangedAppParameters {
    const NAME: &'static str = "ChangedAppParameters";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChangedAppParametersSet {
    /// The set of app parameters at the time the proposal was submitted.
    #[prost(message, optional, tag = "1")]
    pub old: ::core::option::Option<ChangedAppParameters>,
    /// The new set of parameters the proposal is trying to enact.
    #[prost(message, optional, tag = "2")]
    pub new: ::core::option::Option<ChangedAppParameters>,
}
impl ::prost::Name for ChangedAppParametersSet {
    const NAME: &'static str = "ChangedAppParametersSet";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VotingPowerAtProposalStartRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The proposal id to request information on.
    #[prost(uint64, tag = "2")]
    pub proposal_id: u64,
    /// The validator identity key to request information on.
    #[prost(message, optional, tag = "3")]
    pub identity_key: ::core::option::Option<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
}
impl ::prost::Name for VotingPowerAtProposalStartRequest {
    const NAME: &'static str = "VotingPowerAtProposalStartRequest";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VotingPowerAtProposalStartResponse {
    /// The voting power for the given identity key at the start of the proposal.
    /// TODO: since we don't support optional fields in our protos any more,
    /// this will be set to 0 if the validator was not active at the start of the proposal.
    /// Is this potentially an issue?
    #[prost(uint64, tag = "1")]
    pub voting_power: u64,
}
impl ::prost::Name for VotingPowerAtProposalStartResponse {
    const NAME: &'static str = "VotingPowerAtProposalStartResponse";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllTalliedDelegatorVotesForProposalRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The proposal id to request information on.
    #[prost(uint64, tag = "2")]
    pub proposal_id: u64,
}
impl ::prost::Name for AllTalliedDelegatorVotesForProposalRequest {
    const NAME: &'static str = "AllTalliedDelegatorVotesForProposalRequest";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AllTalliedDelegatorVotesForProposalResponse {
    /// The tally of delegator votes for a given validator for the proposal.
    #[prost(message, optional, tag = "1")]
    pub tally: ::core::option::Option<Tally>,
    /// The validator identity associated with the tally.
    #[prost(message, optional, tag = "2")]
    pub identity_key: ::core::option::Option<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
}
impl ::prost::Name for AllTalliedDelegatorVotesForProposalResponse {
    const NAME: &'static str = "AllTalliedDelegatorVotesForProposalResponse";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NextProposalIdRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
}
impl ::prost::Name for NextProposalIdRequest {
    const NAME: &'static str = "NextProposalIdRequest";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NextProposalIdResponse {
    /// The next proposal ID.
    #[prost(uint64, tag = "1")]
    pub next_proposal_id: u64,
}
impl ::prost::Name for NextProposalIdResponse {
    const NAME: &'static str = "NextProposalIdResponse";
    const PACKAGE: &'static str = "penumbra.core.component.governance.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.governance.v1alpha1.{}", Self::NAME
        )
    }
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod query_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query operations for the governance component.
    #[derive(Debug, Clone)]
    pub struct QueryServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QueryServiceClient<tonic::transport::Channel> {
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
    impl<T> QueryServiceClient<T>
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
        ) -> QueryServiceClient<InterceptedService<T, F>>
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
            QueryServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn proposal_info(
            &mut self,
            request: impl tonic::IntoRequest<super::ProposalInfoRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProposalInfoResponse>,
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/ProposalInfo",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.governance.v1alpha1.QueryService",
                        "ProposalInfo",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn proposal_list(
            &mut self,
            request: impl tonic::IntoRequest<super::ProposalListRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::ProposalListResponse>>,
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/ProposalList",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.governance.v1alpha1.QueryService",
                        "ProposalList",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        pub async fn proposal_data(
            &mut self,
            request: impl tonic::IntoRequest<super::ProposalDataRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProposalDataResponse>,
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/ProposalData",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.governance.v1alpha1.QueryService",
                        "ProposalData",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn next_proposal_id(
            &mut self,
            request: impl tonic::IntoRequest<super::NextProposalIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NextProposalIdResponse>,
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/NextProposalId",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.governance.v1alpha1.QueryService",
                        "NextProposalId",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn validator_votes(
            &mut self,
            request: impl tonic::IntoRequest<super::ValidatorVotesRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::ValidatorVotesResponse>>,
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/ValidatorVotes",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.governance.v1alpha1.QueryService",
                        "ValidatorVotes",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        pub async fn voting_power_at_proposal_start(
            &mut self,
            request: impl tonic::IntoRequest<super::VotingPowerAtProposalStartRequest>,
        ) -> std::result::Result<
            tonic::Response<super::VotingPowerAtProposalStartResponse>,
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/VotingPowerAtProposalStart",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.governance.v1alpha1.QueryService",
                        "VotingPowerAtProposalStart",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn all_tallied_delegator_votes_for_proposal(
            &mut self,
            request: impl tonic::IntoRequest<
                super::AllTalliedDelegatorVotesForProposalRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<
                tonic::codec::Streaming<
                    super::AllTalliedDelegatorVotesForProposalResponse,
                >,
            >,
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/AllTalliedDelegatorVotesForProposal",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.governance.v1alpha1.QueryService",
                        "AllTalliedDelegatorVotesForProposal",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        /// Used for computing voting power ?
        pub async fn proposal_rate_data(
            &mut self,
            request: impl tonic::IntoRequest<super::ProposalRateDataRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::ProposalRateDataResponse>>,
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/ProposalRateData",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.governance.v1alpha1.QueryService",
                        "ProposalRateData",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod query_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with QueryServiceServer.
    #[async_trait]
    pub trait QueryService: Send + Sync + 'static {
        async fn proposal_info(
            &self,
            request: tonic::Request<super::ProposalInfoRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProposalInfoResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the ProposalList method.
        type ProposalListStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::ProposalListResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn proposal_list(
            &self,
            request: tonic::Request<super::ProposalListRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::ProposalListStream>,
            tonic::Status,
        >;
        async fn proposal_data(
            &self,
            request: tonic::Request<super::ProposalDataRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProposalDataResponse>,
            tonic::Status,
        >;
        async fn next_proposal_id(
            &self,
            request: tonic::Request<super::NextProposalIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NextProposalIdResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the ValidatorVotes method.
        type ValidatorVotesStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::ValidatorVotesResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn validator_votes(
            &self,
            request: tonic::Request<super::ValidatorVotesRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::ValidatorVotesStream>,
            tonic::Status,
        >;
        async fn voting_power_at_proposal_start(
            &self,
            request: tonic::Request<super::VotingPowerAtProposalStartRequest>,
        ) -> std::result::Result<
            tonic::Response<super::VotingPowerAtProposalStartResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the AllTalliedDelegatorVotesForProposal method.
        type AllTalliedDelegatorVotesForProposalStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<
                    super::AllTalliedDelegatorVotesForProposalResponse,
                    tonic::Status,
                >,
            >
            + Send
            + 'static;
        async fn all_tallied_delegator_votes_for_proposal(
            &self,
            request: tonic::Request<super::AllTalliedDelegatorVotesForProposalRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::AllTalliedDelegatorVotesForProposalStream>,
            tonic::Status,
        >;
        /// Server streaming response type for the ProposalRateData method.
        type ProposalRateDataStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<
                    super::ProposalRateDataResponse,
                    tonic::Status,
                >,
            >
            + Send
            + 'static;
        /// Used for computing voting power ?
        async fn proposal_rate_data(
            &self,
            request: tonic::Request<super::ProposalRateDataRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::ProposalRateDataStream>,
            tonic::Status,
        >;
    }
    /// Query operations for the governance component.
    #[derive(Debug)]
    pub struct QueryServiceServer<T: QueryService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: QueryService> QueryServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for QueryServiceServer<T>
    where
        T: QueryService,
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/ProposalInfo" => {
                    #[allow(non_camel_case_types)]
                    struct ProposalInfoSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::ProposalInfoRequest>
                    for ProposalInfoSvc<T> {
                        type Response = super::ProposalInfoResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ProposalInfoRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::proposal_info(&inner, request).await
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
                        let method = ProposalInfoSvc(inner);
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/ProposalList" => {
                    #[allow(non_camel_case_types)]
                    struct ProposalListSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::ServerStreamingService<super::ProposalListRequest>
                    for ProposalListSvc<T> {
                        type Response = super::ProposalListResponse;
                        type ResponseStream = T::ProposalListStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ProposalListRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::proposal_list(&inner, request).await
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
                        let method = ProposalListSvc(inner);
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/ProposalData" => {
                    #[allow(non_camel_case_types)]
                    struct ProposalDataSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::ProposalDataRequest>
                    for ProposalDataSvc<T> {
                        type Response = super::ProposalDataResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ProposalDataRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::proposal_data(&inner, request).await
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
                        let method = ProposalDataSvc(inner);
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/NextProposalId" => {
                    #[allow(non_camel_case_types)]
                    struct NextProposalIdSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::NextProposalIdRequest>
                    for NextProposalIdSvc<T> {
                        type Response = super::NextProposalIdResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NextProposalIdRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::next_proposal_id(&inner, request).await
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
                        let method = NextProposalIdSvc(inner);
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/ValidatorVotes" => {
                    #[allow(non_camel_case_types)]
                    struct ValidatorVotesSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::ServerStreamingService<super::ValidatorVotesRequest>
                    for ValidatorVotesSvc<T> {
                        type Response = super::ValidatorVotesResponse;
                        type ResponseStream = T::ValidatorVotesStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ValidatorVotesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::validator_votes(&inner, request).await
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
                        let method = ValidatorVotesSvc(inner);
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/VotingPowerAtProposalStart" => {
                    #[allow(non_camel_case_types)]
                    struct VotingPowerAtProposalStartSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<
                        super::VotingPowerAtProposalStartRequest,
                    > for VotingPowerAtProposalStartSvc<T> {
                        type Response = super::VotingPowerAtProposalStartResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::VotingPowerAtProposalStartRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::voting_power_at_proposal_start(
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
                        let method = VotingPowerAtProposalStartSvc(inner);
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/AllTalliedDelegatorVotesForProposal" => {
                    #[allow(non_camel_case_types)]
                    struct AllTalliedDelegatorVotesForProposalSvc<T: QueryService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: QueryService,
                    > tonic::server::ServerStreamingService<
                        super::AllTalliedDelegatorVotesForProposalRequest,
                    > for AllTalliedDelegatorVotesForProposalSvc<T> {
                        type Response = super::AllTalliedDelegatorVotesForProposalResponse;
                        type ResponseStream = T::AllTalliedDelegatorVotesForProposalStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::AllTalliedDelegatorVotesForProposalRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::all_tallied_delegator_votes_for_proposal(
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
                        let method = AllTalliedDelegatorVotesForProposalSvc(inner);
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
                "/penumbra.core.component.governance.v1alpha1.QueryService/ProposalRateData" => {
                    #[allow(non_camel_case_types)]
                    struct ProposalRateDataSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::ServerStreamingService<
                        super::ProposalRateDataRequest,
                    > for ProposalRateDataSvc<T> {
                        type Response = super::ProposalRateDataResponse;
                        type ResponseStream = T::ProposalRateDataStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ProposalRateDataRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::proposal_rate_data(&inner, request)
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
                        let method = ProposalRateDataSvc(inner);
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
    impl<T: QueryService> Clone for QueryServiceServer<T> {
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
    impl<T: QueryService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: QueryService> tonic::server::NamedService for QueryServiceServer<T> {
        const NAME: &'static str = "penumbra.core.component.governance.v1alpha1.QueryService";
    }
}
