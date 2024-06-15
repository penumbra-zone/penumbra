/// Funding component configuration data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FundingParameters {}
impl ::prost::Name for FundingParameters {
    const NAME: &'static str = "FundingParameters";
    const PACKAGE: &'static str = "penumbra.core.component.funding.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.funding.v1.{}", Self::NAME)
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
    const PACKAGE: &'static str = "penumbra.core.component.funding.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.funding.v1.{}", Self::NAME)
    }
}
/// Indicates that a funding stream reward was paid.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventFundingStreamReward {
    /// The recipient of the funding stream reward.
    /// This is a string value for future extensibility.
    /// Currently it will be either "community-pool"
    /// or an address.
    #[prost(string, tag = "1")]
    pub recipient: ::prost::alloc::string::String,
    /// The epoch for which the reward was paid.
    #[prost(uint64, tag = "2")]
    pub epoch_index: u64,
    /// The amount of the reward, in staking tokens.
    #[prost(message, optional, tag = "3")]
    pub reward_amount: ::core::option::Option<super::super::super::num::v1::Amount>,
}
impl ::prost::Name for EventFundingStreamReward {
    const NAME: &'static str = "EventFundingStreamReward";
    const PACKAGE: &'static str = "penumbra.core.component.funding.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.funding.v1.{}", Self::NAME)
    }
}
