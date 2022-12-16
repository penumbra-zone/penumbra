/// A vote on a proposal.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[serde(transparent)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vote {
    /// The vote.
    #[prost(enumeration = "vote::Vote", tag = "1")]
    #[serde(with = "crate::serializers::vote")]
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
        NoWithVeto = 4,
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
                Vote::NoWithVeto => "VOTE_NO_WITH_VETO",
            }
        }
    }
}
/// A chain parameter that can be modified by governance.
#[derive(::serde::Deserialize, ::serde::Serialize)]
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
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalState {
    /// The state of the proposal.
    #[prost(oneof = "proposal_state::State", tags = "2, 3, 4")]
    #[serde(flatten)]
    pub state: ::core::option::Option<proposal_state::State>,
}
/// Nested message and enum types in `ProposalState`.
pub mod proposal_state {
    /// Voting is in progress and the proposal has not yet concluded voting or been withdrawn.
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Voting {}
    /// The proposal has been withdrawn but the voting period is not yet concluded.
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Withdrawn {
        /// The reason for the withdrawal.
        #[prost(string, tag = "1")]
        pub reason: ::prost::alloc::string::String,
    }
    /// The voting period has ended, and the proposal has been assigned an outcome.
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Finished {
        #[prost(message, optional, tag = "1")]
        #[serde(flatten)]
        pub outcome: ::core::option::Option<super::ProposalOutcome>,
    }
    /// The state of the proposal.
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[serde(rename_all = "snake_case")]
    #[serde(tag = "state")]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum State {
        #[prost(message, tag = "2")]
        Voting(Voting),
        #[prost(message, tag = "3")]
        Withdrawn(Withdrawn),
        #[prost(message, tag = "4")]
        Finished(Finished),
    }
}
/// The outcome of a concluded proposal.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalOutcome {
    #[prost(oneof = "proposal_outcome::Outcome", tags = "1, 2, 3")]
    #[serde(flatten)]
    pub outcome: ::core::option::Option<proposal_outcome::Outcome>,
}
/// Nested message and enum types in `ProposalOutcome`.
pub mod proposal_outcome {
    /// The proposal was passed.
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Passed {}
    /// The proposal did not pass.
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Failed {
        /// The proposal was withdrawn during the voting period.
        #[prost(string, optional, tag = "1")]
        #[serde(skip_serializing_if = "Option::is_none", default)]
        pub withdrawn_with_reason: ::core::option::Option<
            ::prost::alloc::string::String,
        >,
    }
    /// The proposal did not pass, and was vetoed.
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Vetoed {
        /// The proposal was withdrawn during the voting period.
        #[prost(string, optional, tag = "1")]
        #[serde(skip_serializing_if = "Option::is_none", default)]
        pub withdrawn_with_reason: ::core::option::Option<
            ::prost::alloc::string::String,
        >,
    }
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[serde(rename_all = "snake_case")]
    #[serde(tag = "outcome")]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Outcome {
        #[prost(message, tag = "1")]
        Passed(Passed),
        #[prost(message, tag = "2")]
        Failed(Failed),
        #[prost(message, tag = "3")]
        Vetoed(Vetoed),
    }
}
/// A list of proposal ids.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalList {
    #[prost(uint64, repeated, tag = "1")]
    pub proposals: ::prost::alloc::vec::Vec<u64>,
}
