/// Distribution configuration data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DistributionsParameters {
    /// The amount of staking token issued per block.
    #[prost(uint64, tag = "1")]
    pub staking_issuance_per_block: u64,
}
impl ::prost::Name for DistributionsParameters {
    const NAME: &'static str = "DistributionsParameters";
    const PACKAGE: &'static str = "penumbra.core.component.distributions.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.distributions.v1alpha1.{}", Self::NAME
        )
    }
}
/// Genesis data for the distributions module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContent {
    #[prost(message, optional, tag = "1")]
    pub distributions_params: ::core::option::Option<DistributionsParameters>,
}
impl ::prost::Name for GenesisContent {
    const NAME: &'static str = "GenesisContent";
    const PACKAGE: &'static str = "penumbra.core.component.distributions.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.distributions.v1alpha1.{}", Self::NAME
        )
    }
}
