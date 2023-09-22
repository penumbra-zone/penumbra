/// An authorization hash for a Penumbra transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EffectHash {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// Global chain configuration data, such as chain ID, epoch duration, etc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChainParameters {
    /// The identifier of the chain.
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The duration of each epoch, in number of blocks.
    #[prost(uint64, tag = "2")]
    pub epoch_duration: u64,
    /// The number of epochs an unbonding note for before being released.
    #[prost(uint64, tag = "3")]
    pub unbonding_epochs: u64,
    /// The maximum number of validators in the consensus set.
    #[prost(uint64, tag = "4")]
    pub active_validator_limit: u64,
    /// The base reward rate, expressed in basis points of basis points
    #[prost(uint64, tag = "9")]
    pub base_reward_rate: u64,
    /// The penalty for slashing due to misbehavior.
    #[prost(uint64, tag = "5")]
    pub slashing_penalty_misbehavior: u64,
    /// The penalty for slashing due to downtime.
    #[prost(uint64, tag = "10")]
    pub slashing_penalty_downtime: u64,
    /// The number of blocks in the window to check for downtime.
    #[prost(uint64, tag = "11")]
    pub signed_blocks_window_len: u64,
    /// The maximum number of blocks in the window each validator can miss signing without slashing.
    #[prost(uint64, tag = "12")]
    pub missed_blocks_maximum: u64,
    /// Whether IBC (forming connections, processing IBC packets) is enabled.
    #[prost(bool, tag = "6")]
    pub ibc_enabled: bool,
    /// Whether inbound ICS-20 transfers are enabled
    #[prost(bool, tag = "7")]
    pub inbound_ics20_transfers_enabled: bool,
    /// Whether outbound ICS-20 transfers are enabled
    #[prost(bool, tag = "8")]
    pub outbound_ics20_transfers_enabled: bool,
    /// The number of blocks during which a proposal is voted on.
    #[prost(uint64, tag = "20")]
    pub proposal_voting_blocks: u64,
    /// The deposit required to create a proposal.
    #[prost(message, optional, tag = "21")]
    pub proposal_deposit_amount: ::core::option::Option<
        super::super::super::num::v1alpha1::Amount,
    >,
    /// The quorum required for a proposal to be considered valid, as a fraction of the total stake
    /// weight of the network.
    #[prost(string, tag = "22")]
    pub proposal_valid_quorum: ::prost::alloc::string::String,
    /// The threshold for a proposal to pass voting, as a ratio of "yes" votes over "no" votes.
    #[prost(string, tag = "23")]
    pub proposal_pass_threshold: ::prost::alloc::string::String,
    /// The threshold for a proposal to be slashed, regardless of whether the "yes" and "no" votes
    /// would have passed it, as a ratio of "no" votes over all total votes.
    #[prost(string, tag = "24")]
    pub proposal_slash_threshold: ::prost::alloc::string::String,
    /// Whether DAO spend proposals are enabled.
    #[prost(bool, tag = "25")]
    pub dao_spend_proposals_enabled: bool,
}
/// The ratio between two numbers, used in governance to describe vote thresholds and quorums.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ratio {
    /// The numerator.
    #[prost(uint64, tag = "1")]
    pub numerator: u64,
    /// The denominator.
    #[prost(uint64, tag = "2")]
    pub denominator: u64,
}
/// Parameters for Fuzzy Message Detection
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FmdParameters {
    #[prost(uint32, tag = "1")]
    pub precision_bits: u32,
    #[prost(uint64, tag = "2")]
    pub as_of_block_height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KnownAssets {
    #[prost(message, repeated, tag = "1")]
    pub assets: ::prost::alloc::vec::Vec<
        super::super::super::asset::v1alpha1::DenomMetadata,
    >,
}
/// A spicy transaction ID
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NoteSource {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// A NoteSource paired with the height at which the note was spent
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendInfo {
    #[prost(message, optional, tag = "1")]
    pub note_source: ::core::option::Option<NoteSource>,
    #[prost(uint64, tag = "2")]
    pub spend_height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisAppState {
    #[prost(oneof = "genesis_app_state::GenesisAppState", tags = "1, 2")]
    pub genesis_app_state: ::core::option::Option<genesis_app_state::GenesisAppState>,
}
/// Nested message and enum types in `GenesisAppState`.
pub mod genesis_app_state {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum GenesisAppState {
        #[prost(message, tag = "1")]
        GenesisContent(super::GenesisContent),
        #[prost(bytes, tag = "2")]
        GenesisCheckpoint(::prost::alloc::vec::Vec<u8>),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContent {
    #[prost(message, optional, tag = "1")]
    pub chain_params: ::core::option::Option<ChainParameters>,
    #[prost(message, repeated, tag = "2")]
    pub validators: ::prost::alloc::vec::Vec<super::super::stake::v1alpha1::Validator>,
    #[prost(message, repeated, tag = "3")]
    pub allocations: ::prost::alloc::vec::Vec<genesis_content::Allocation>,
}
/// Nested message and enum types in `GenesisContent`.
pub mod genesis_content {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Allocation {
        #[prost(message, optional, tag = "1")]
        pub amount: ::core::option::Option<
            super::super::super::super::num::v1alpha1::Amount,
        >,
        #[prost(string, tag = "2")]
        pub denom: ::prost::alloc::string::String,
        #[prost(message, optional, tag = "3")]
        pub address: ::core::option::Option<
            super::super::super::super::keys::v1alpha1::Address,
        >,
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Epoch {
    #[prost(uint64, tag = "1")]
    pub index: u64,
    #[prost(uint64, tag = "2")]
    pub start_height: u64,
}
