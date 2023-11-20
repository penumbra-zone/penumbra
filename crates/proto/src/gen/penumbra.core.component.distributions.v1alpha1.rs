/// Distribution configuration data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DistributionsParameters {
    /// The amount of staking token issued per block.
    #[prost(uint64, tag = "1")]
    pub staking_issuance_per_block: u64,
}
/// Genesis data for the distributions module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContent {
    #[prost(message, optional, tag = "1")]
    pub distributions_params: ::core::option::Option<DistributionsParameters>,
}
