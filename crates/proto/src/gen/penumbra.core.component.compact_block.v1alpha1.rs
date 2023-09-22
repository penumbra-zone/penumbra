/// Contains the minimum data needed to update client state.
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
    pub nullifiers: ::prost::alloc::vec::Vec<super::super::sct::v1alpha1::Nullifier>,
    /// The block root of this block.
    #[prost(message, optional, tag = "4")]
    pub block_root: ::core::option::Option<
        super::super::super::super::crypto::tct::v1alpha1::MerkleRoot,
    >,
    /// The epoch root of this epoch (only present when the block is the last in an epoch).
    #[prost(message, optional, tag = "17")]
    pub epoch_root: ::core::option::Option<
        super::super::super::super::crypto::tct::v1alpha1::MerkleRoot,
    >,
    /// If a proposal started voting in this block, this is set to `true`.
    #[prost(bool, tag = "20")]
    pub proposal_started: bool,
    /// Latest Fuzzy Message Detection parameters.
    #[prost(message, optional, tag = "100")]
    pub fmd_parameters: ::core::option::Option<
        super::super::chain::v1alpha1::FmdParameters,
    >,
    /// Price data for swaps executed in this block.
    #[prost(message, repeated, tag = "5")]
    pub swap_outputs: ::prost::alloc::vec::Vec<
        super::super::dex::v1alpha1::BatchSwapOutputData,
    >,
    /// Updated chain parameters, if they have changed.
    #[prost(message, optional, tag = "6")]
    pub chain_parameters: ::core::option::Option<
        super::super::chain::v1alpha1::ChainParameters,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatePayload {
    #[prost(oneof = "state_payload::StatePayload", tags = "1, 2, 3")]
    pub state_payload: ::core::option::Option<state_payload::StatePayload>,
}
/// Nested message and enum types in `StatePayload`.
pub mod state_payload {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RolledUp {
        #[prost(message, optional, tag = "1")]
        pub commitment: ::core::option::Option<
            super::super::super::super::super::crypto::tct::v1alpha1::StateCommitment,
        >,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Note {
        #[prost(message, optional, tag = "1")]
        pub source: ::core::option::Option<
            super::super::super::chain::v1alpha1::NoteSource,
        >,
        #[prost(message, optional, tag = "2")]
        pub note: ::core::option::Option<
            super::super::super::shielded_pool::v1alpha1::NotePayload,
        >,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Swap {
        #[prost(message, optional, tag = "1")]
        pub source: ::core::option::Option<
            super::super::super::chain::v1alpha1::NoteSource,
        >,
        #[prost(message, optional, tag = "2")]
        pub swap: ::core::option::Option<
            super::super::super::dex::v1alpha1::SwapPayload,
        >,
    }
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
