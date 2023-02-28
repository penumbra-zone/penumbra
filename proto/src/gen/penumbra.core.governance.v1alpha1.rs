#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalSubmit {
    /// The proposal to be submitted.
    #[prost(message, optional, tag = "1")]
    pub proposal: ::core::option::Option<Proposal>,
    /// The amount of the proposal deposit.
    #[prost(message, optional, tag = "3")]
    pub deposit_amount: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorVote {
    /// The effecting data for the vote.
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<ValidatorVoteBody>,
    /// The vote authorization signature is authorizing data.
    #[prost(message, optional, tag = "2")]
    pub auth_sig: ::core::option::Option<
        super::super::crypto::v1alpha1::SpendAuthSignature,
    >,
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
        super::super::crypto::v1alpha1::IdentityKey,
    >,
    /// The validator governance key.
    #[prost(message, optional, tag = "4")]
    pub governance_key: ::core::option::Option<
        super::super::crypto::v1alpha1::GovernanceKey,
    >,
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
        super::super::crypto::v1alpha1::SpendAuthSignature,
    >,
    /// The vote proof is authorizing data.
    #[prost(bytes = "vec", tag = "3")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
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
    pub value: ::core::option::Option<super::super::crypto::v1alpha1::Value>,
    /// The amount of the delegation note, in unbonded penumbra.
    #[prost(message, optional, tag = "5")]
    pub unbonded_amount: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// The nullifier of the input note.
    #[prost(bytes = "vec", tag = "6")]
    pub nullifier: ::prost::alloc::vec::Vec<u8>,
    /// The randomized validating key for the spend authorization signature.
    #[prost(bytes = "vec", tag = "7")]
    pub rk: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalDepositClaim {
    /// The proposal to claim the deposit for.
    #[prost(uint64, tag = "1")]
    pub proposal: u64,
    /// The expected deposit amount.
    #[prost(message, optional, tag = "2")]
    pub deposit_amount: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// The outcome of the proposal.
    #[prost(message, optional, tag = "3")]
    pub outcome: ::core::option::Option<ProposalOutcome>,
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
    pub staked_note: ::core::option::Option<super::super::crypto::v1alpha1::Note>,
    /// The position of that delegation note.
    #[prost(uint64, tag = "5")]
    pub staked_note_position: u64,
    /// The unbonded amount equivalent to the delegation note.
    #[prost(message, optional, tag = "6")]
    pub unbonded_amount: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// The randomizer to use for the proof of spend capability.
    #[prost(bytes = "vec", tag = "7")]
    pub randomizer: ::prost::alloc::vec::Vec<u8>,
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
/// A chain parameter that can be modified by governance.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MutableChainParameter {
    /// The identifier of the parameter, used for submitting change proposals.
    #[prost(string, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    /// A textual description of the parameter and valid values.
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
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
    /// The proposal has been withdrawn but the voting period is not yet concluded.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Withdrawn {
        /// The reason for the withdrawal.
        #[prost(string, tag = "1")]
        pub reason: ::prost::alloc::string::String,
    }
    /// The voting period has ended, and the proposal has been assigned an outcome.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Finished {
        #[prost(message, optional, tag = "1")]
        pub outcome: ::core::option::Option<super::ProposalOutcome>,
    }
    /// The voting period has ended, and the original proposer has claimed their deposit.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Claimed {
        #[prost(message, optional, tag = "1")]
        pub outcome: ::core::option::Option<super::ProposalOutcome>,
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
/// The outcome of a concluded proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalOutcome {
    #[prost(oneof = "proposal_outcome::Outcome", tags = "1, 2, 3")]
    pub outcome: ::core::option::Option<proposal_outcome::Outcome>,
}
/// Nested message and enum types in `ProposalOutcome`.
pub mod proposal_outcome {
    /// The proposal was passed.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Passed {}
    /// The proposal did not pass.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Failed {
        /// The proposal was withdrawn during the voting period.
        #[prost(string, optional, tag = "1")]
        pub withdrawn_with_reason: ::core::option::Option<
            ::prost::alloc::string::String,
        >,
    }
    /// The proposal did not pass, and was slashed.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Slashed {
        /// The proposal was withdrawn during the voting period.
        #[prost(string, optional, tag = "1")]
        pub withdrawn_with_reason: ::core::option::Option<
            ::prost::alloc::string::String,
        >,
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
    pub dao_spend: ::core::option::Option<proposal::DaoSpend>,
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
        #[prost(string, optional, tag = "1")]
        pub commit: ::core::option::Option<::prost::alloc::string::String>,
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
    /// A parameter change proposal describes a replacement of the chain parameters, which should take
    /// effect when the proposal is passed.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ParameterChange {
        /// The old chain parameters to be replaced: even if the proposal passes, the update will not be
        /// applied if the chain parameters have changed *at all* from these chain parameters. Usually,
        /// this should be set to the current chain parameters at time of proposal.
        #[prost(message, optional, tag = "1")]
        pub old_parameters: ::core::option::Option<
            super::super::super::chain::v1alpha1::ChainParameters,
        >,
        /// The new chain parameters to be set: the *entire* chain parameters will be replaced with these
        /// at the time the proposal is passed.
        #[prost(message, optional, tag = "2")]
        pub new_parameters: ::core::option::Option<
            super::super::super::chain::v1alpha1::ChainParameters,
        >,
    }
    /// A DAO spend proposal describes zero or more transactions to execute on behalf of the DAO, with
    /// access to its funds, and zero or more scheduled transactions from previous passed proposals to
    /// cancel.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct DaoSpend {
        /// The transaction plan to be executed at the time the proposal is passed. This must be a
        /// transaction plan which can be executed by the DAO, which means it can't require any witness
        /// data or authorization signatures, but it may use the `DaoSpend` action.
        #[prost(message, optional, tag = "2")]
        pub transaction_plan: ::core::option::Option<::pbjson_types::Any>,
    }
}
