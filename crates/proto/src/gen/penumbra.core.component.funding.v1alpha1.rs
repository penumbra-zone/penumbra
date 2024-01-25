/// Funding component configuration data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FundingParameters {}
impl ::prost::Name for FundingParameters {
    const NAME: &'static str = "FundingParameters";
    const PACKAGE: &'static str = "penumbra.core.component.funding.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.funding.v1alpha1.{}", Self::NAME
        )
    }
}
/// Genesis data for the funding component.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContent {
    #[prost(message, optional, tag = "1")]
    pub funding_params: ::core::option::Option<FundingParameters>,
}
impl ::prost::Name for GenesisContent {
    const NAME: &'static str = "GenesisContent";
    const PACKAGE: &'static str = "penumbra.core.component.funding.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.funding.v1alpha1.{}", Self::NAME
        )
    }
}
