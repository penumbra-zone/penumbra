/// Global chain configuration data, such as chain ID, epoch duration, etc.
#[derive(::serde::Deserialize, ::serde::Serialize)]
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
    #[prost(message, optional, tag = "5")]
    pub slashing_penalty_misbehavior: ::core::option::Option<
        super::super::stake::v1alpha1::Penalty,
    >,
    /// The penalty for slashing due to downtime.
    #[prost(message, optional, tag = "10")]
    pub slashing_penalty_downtime: ::core::option::Option<
        super::super::stake::v1alpha1::Penalty,
    >,
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
        super::super::crypto::v1alpha1::Amount,
    >,
    /// The quorum required for a proposal to be considered valid, as a fraction of the total stake
    /// weight of the network.
    #[prost(message, optional, tag = "22")]
    pub proposal_valid_quorum: ::core::option::Option<Ratio>,
    /// The threshold for a proposal to pass voting, as a ratio of "yes" votes over "no" votes.
    #[prost(message, optional, tag = "23")]
    pub proposal_pass_threshold: ::core::option::Option<Ratio>,
    /// The threshold for a proposal to be vetoed, regardless of whether the "yes" and "no" votes would
    /// have passed it, as a ratio of "no with veto" votes over all total votes.
    #[prost(message, optional, tag = "24")]
    pub proposal_veto_threshold: ::core::option::Option<Ratio>,
}
/// The ratio between two numbers, used in governance to describe vote thresholds and quorums.
#[derive(::serde::Deserialize, ::serde::Serialize)]
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
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FmdParameters {
    #[prost(uint32, tag = "1")]
    pub precision_bits: u32,
    #[prost(uint64, tag = "2")]
    pub as_of_block_height: u64,
}
/// TODO: delete with legacy code
/// Information about a given asset at a given time (as specified by block
/// height). Currently this only contains the total supply.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetInfo {
    #[prost(message, optional, tag = "1")]
    pub asset_id: ::core::option::Option<super::super::crypto::v1alpha1::AssetId>,
    #[prost(message, optional, tag = "2")]
    pub denom: ::core::option::Option<super::super::crypto::v1alpha1::Denom>,
    #[prost(uint64, tag = "3")]
    pub as_of_block_height: u64,
    #[prost(uint64, tag = "4")]
    pub total_supply: u64,
}
/// Contains the minimum data needed to update client state.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactBlock {
    #[prost(uint64, tag = "1")]
    pub height: u64,
    /// State payloads describing new state fragments.
    #[prost(message, repeated, tag = "2")]
    pub state_payloads: ::prost::alloc::vec::Vec<StatePayload>,
    /// Nullifiers identifying spent notes.
    #[prost(message, repeated, tag = "3")]
    pub nullifiers: ::prost::alloc::vec::Vec<super::super::crypto::v1alpha1::Nullifier>,
    /// The block root of this block.
    #[prost(message, optional, tag = "4")]
    pub block_root: ::core::option::Option<super::super::crypto::v1alpha1::MerkleRoot>,
    /// The epoch root of this epoch (only present when the block is the last in an epoch).
    #[prost(message, optional, tag = "17")]
    pub epoch_root: ::core::option::Option<super::super::crypto::v1alpha1::MerkleRoot>,
    /// If a proposal started voting in this block, this is set to `true`.
    #[prost(bool, tag = "20")]
    pub proposal_started: bool,
    /// Latest Fuzzy Message Detection parameters.
    #[prost(message, optional, tag = "100")]
    pub fmd_parameters: ::core::option::Option<FmdParameters>,
    /// Price data for swaps executed in this block.
    #[prost(message, repeated, tag = "5")]
    pub swap_outputs: ::prost::alloc::vec::Vec<
        super::super::dex::v1alpha1::BatchSwapOutputData,
    >,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatePayload {
    #[prost(oneof = "state_payload::StatePayload", tags = "1, 2, 3")]
    pub state_payload: ::core::option::Option<state_payload::StatePayload>,
}
/// Nested message and enum types in `StatePayload`.
pub mod state_payload {
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RolledUp {
        #[prost(message, optional, tag = "1")]
        pub commitment: ::core::option::Option<
            super::super::super::crypto::v1alpha1::StateCommitment,
        >,
    }
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Note {
        #[prost(message, optional, tag = "1")]
        pub source: ::core::option::Option<super::NoteSource>,
        #[prost(message, optional, tag = "2")]
        pub note: ::core::option::Option<
            super::super::super::crypto::v1alpha1::EncryptedNote,
        >,
    }
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Swap {
        #[prost(message, optional, tag = "1")]
        pub source: ::core::option::Option<super::NoteSource>,
        #[prost(message, optional, tag = "2")]
        pub swap: ::core::option::Option<
            super::super::super::dex::v1alpha1::SwapPayload,
        >,
    }
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum StatePayload {
        #[prost(message, tag = "1")]
        RolledUp(RolledUp),
        #[prost(message, tag = "2")]
        Note(Note),
        #[prost(message, tag = "3")]
        Swap(Swap),
    }
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[serde(transparent)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KnownAssets {
    #[prost(message, repeated, tag = "1")]
    pub assets: ::prost::alloc::vec::Vec<super::super::crypto::v1alpha1::Asset>,
}
/// A spicy transaction ID
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[serde(transparent)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NoteSource {
    #[prost(bytes = "vec", tag = "1")]
    #[serde(with = "crate::serializers::hexstr")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisAppState {
    #[prost(message, optional, tag = "1")]
    pub chain_params: ::core::option::Option<ChainParameters>,
    #[prost(message, repeated, tag = "2")]
    pub validators: ::prost::alloc::vec::Vec<super::super::stake::v1alpha1::Validator>,
    #[prost(message, repeated, tag = "3")]
    pub allocations: ::prost::alloc::vec::Vec<genesis_app_state::Allocation>,
}
/// Nested message and enum types in `GenesisAppState`.
pub mod genesis_app_state {
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Allocation {
        #[prost(uint64, tag = "1")]
        pub amount: u64,
        #[prost(string, tag = "2")]
        pub denom: ::prost::alloc::string::String,
        #[prost(message, optional, tag = "3")]
        pub address: ::core::option::Option<
            super::super::super::crypto::v1alpha1::Address,
        >,
    }
}
