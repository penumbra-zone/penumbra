/// Specifies fees paid by a transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fee {
    /// The amount of the token used to pay fees.
    #[prost(message, optional, tag = "1")]
    pub amount: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
    /// If present, the asset ID of the token used to pay fees.
    /// If absent, specifies the staking token implicitly.
    #[prost(message, optional, tag = "2")]
    pub asset_id: ::core::option::Option<super::super::super::asset::v1alpha1::AssetId>,
}
impl ::prost::Name for Fee {
    const NAME: &'static str = "Fee";
    const PACKAGE: &'static str = "penumbra.core.component.fee.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.fee.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GasPrices {
    /// The price per unit block space in terms of the staking token, with an implicit 1,000 denominator.
    #[prost(uint64, tag = "1")]
    pub block_space_price: u64,
    /// The price per unit compact block space in terms of the staking token, with an implicit 1,000 denominator.
    #[prost(uint64, tag = "2")]
    pub compact_block_space_price: u64,
    /// The price per unit verification cost in terms of the staking token, with an implicit 1,000 denominator.
    #[prost(uint64, tag = "3")]
    pub verification_price: u64,
    /// The price per unit execution cost in terms of the staking token, with an implicit 1,000 denominator.
    #[prost(uint64, tag = "4")]
    pub execution_price: u64,
}
impl ::prost::Name for GasPrices {
    const NAME: &'static str = "GasPrices";
    const PACKAGE: &'static str = "penumbra.core.component.fee.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.fee.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FeeTier {
    /// The selected fee tier.
    #[prost(enumeration = "fee_tier::Tier", tag = "1")]
    pub fee_tier: i32,
}
/// Nested message and enum types in `FeeTier`.
pub mod fee_tier {
    /// The tier for the fee.
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
    pub enum Tier {
        Low = 0,
        Medium = 1,
        High = 2,
    }
    impl Tier {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Tier::Low => "TIER_LOW",
                Tier::Medium => "TIER_MEDIUM",
                Tier::High => "TIER_HIGH",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "TIER_LOW" => Some(Self::Low),
                "TIER_MEDIUM" => Some(Self::Medium),
                "TIER_HIGH" => Some(Self::High),
                _ => None,
            }
        }
    }
}
impl ::prost::Name for FeeTier {
    const NAME: &'static str = "FeeTier";
    const PACKAGE: &'static str = "penumbra.core.component.fee.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.fee.v1alpha1.{}", Self::NAME)
    }
}
/// Fee component configuration data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FeeParameters {}
impl ::prost::Name for FeeParameters {
    const NAME: &'static str = "FeeParameters";
    const PACKAGE: &'static str = "penumbra.core.component.fee.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.fee.v1alpha1.{}", Self::NAME)
    }
}
/// Fee-specific genesis content.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContent {
    /// The FeeParameters present at genesis.
    #[prost(message, optional, tag = "1")]
    pub fee_params: ::core::option::Option<FeeParameters>,
    /// The initial gas prices.
    #[prost(message, optional, tag = "2")]
    pub gas_prices: ::core::option::Option<GasPrices>,
}
impl ::prost::Name for GenesisContent {
    const NAME: &'static str = "GenesisContent";
    const PACKAGE: &'static str = "penumbra.core.component.fee.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.fee.v1alpha1.{}", Self::NAME)
    }
}
