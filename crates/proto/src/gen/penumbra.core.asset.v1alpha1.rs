#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BalanceCommitment {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for BalanceCommitment {
    const NAME: &'static str = "BalanceCommitment";
    const PACKAGE: &'static str = "penumbra.core.asset.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.asset.v1alpha1.{}", Self::NAME)
    }
}
/// A Penumbra asset ID.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetId {
    /// The bytes of the asset ID.
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
    /// Alternatively, a Bech32m-encoded string representation of the `inner`
    /// bytes.
    ///
    /// NOTE: implementations are not required to support parsing this field.
    /// Implementations should prefer to encode the `inner` bytes in all messages they
    /// produce. Implementations must not accept messages with both `inner` and
    /// `alt_bech32m` set.  This field exists for convenience of RPC users.
    #[prost(string, tag = "2")]
    pub alt_bech32m: ::prost::alloc::string::String,
    /// Alternatively, a base denomination string which should be hashed to obtain the asset ID.
    ///
    /// NOTE: implementations are not required to support parsing this field.
    /// Implementations should prefer to encode the bytes in all messages they
    /// produce. Implementations must not accept messages with both `inner` and
    /// `alt_base_denom` set.  This field exists for convenience of RPC users.
    #[prost(string, tag = "3")]
    pub alt_base_denom: ::prost::alloc::string::String,
}
impl ::prost::Name for AssetId {
    const NAME: &'static str = "AssetId";
    const PACKAGE: &'static str = "penumbra.core.asset.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.asset.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Denom {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
impl ::prost::Name for Denom {
    const NAME: &'static str = "Denom";
    const PACKAGE: &'static str = "penumbra.core.asset.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.asset.v1alpha1.{}", Self::NAME)
    }
}
/// Describes metadata about a given asset.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Metadata {
    #[prost(string, tag = "1")]
    pub description: ::prost::alloc::string::String,
    /// denom_units represents the list of DenomUnit's for a given coin
    #[prost(message, repeated, tag = "2")]
    pub denom_units: ::prost::alloc::vec::Vec<DenomUnit>,
    /// base represents the base denom (should be the DenomUnit with exponent = 0).
    #[prost(string, tag = "3")]
    pub base: ::prost::alloc::string::String,
    /// display indicates the suggested denom that should be
    /// displayed in clients.
    #[prost(string, tag = "4")]
    pub display: ::prost::alloc::string::String,
    /// name defines the name of the token (eg: Cosmos Atom)
    #[prost(string, tag = "5")]
    pub name: ::prost::alloc::string::String,
    /// symbol is the token symbol usually shown on exchanges (eg: ATOM). This can
    /// be the same as the display.
    #[prost(string, tag = "6")]
    pub symbol: ::prost::alloc::string::String,
    /// the asset ID on Penumbra for this denomination.
    #[prost(message, optional, tag = "1984")]
    pub penumbra_asset_id: ::core::option::Option<AssetId>,
    #[prost(message, repeated, tag = "1985")]
    pub images: ::prost::alloc::vec::Vec<AssetImage>,
}
impl ::prost::Name for Metadata {
    const NAME: &'static str = "Metadata";
    const PACKAGE: &'static str = "penumbra.core.asset.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.asset.v1alpha1.{}", Self::NAME)
    }
}
/// DenomUnit represents a struct that describes a given denomination unit of the basic token.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DenomUnit {
    /// denom represents the string name of the given denom unit (e.g uatom).
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    /// exponent represents power of 10 exponent that one must
    /// raise the base_denom to in order to equal the given DenomUnit's denom
    /// 1 denom = 10^exponent base_denom
    /// (e.g. with a base_denom of uatom, one can create a DenomUnit of 'atom' with
    /// exponent = 6, thus: 1 atom = 10^6 uatom).
    #[prost(uint32, tag = "2")]
    pub exponent: u32,
    /// aliases is a list of string aliases for the given denom
    #[prost(string, repeated, tag = "3")]
    pub aliases: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for DenomUnit {
    const NAME: &'static str = "DenomUnit";
    const PACKAGE: &'static str = "penumbra.core.asset.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.asset.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Value {
    #[prost(message, optional, tag = "1")]
    pub amount: ::core::option::Option<super::super::num::v1alpha1::Amount>,
    #[prost(message, optional, tag = "2")]
    pub asset_id: ::core::option::Option<AssetId>,
}
impl ::prost::Name for Value {
    const NAME: &'static str = "Value";
    const PACKAGE: &'static str = "penumbra.core.asset.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.asset.v1alpha1.{}", Self::NAME)
    }
}
/// Represents a value of a known or unknown denomination.
///
/// Note: unlike some other View types, we don't just store the underlying
/// `Value` message together with an additional `Denom`.  Instead, we record
/// either an `Amount` and `Denom` (only) or an `Amount` and `AssetId`.  This is
/// because we don't want to allow a situation where the supplied `Denom` doesn't
/// match the `AssetId`, and a consumer of the API that doesn't check is tricked.
/// This way, the `Denom` will always match, because the consumer is forced to
/// recompute it themselves if they want it.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValueView {
    #[prost(oneof = "value_view::ValueView", tags = "1, 2")]
    pub value_view: ::core::option::Option<value_view::ValueView>,
}
/// Nested message and enum types in `ValueView`.
pub mod value_view {
    /// A value whose asset ID has a known denomination.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct KnownDenom {
        #[prost(message, optional, tag = "1")]
        pub amount: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
        #[prost(message, optional, tag = "2")]
        pub denom: ::core::option::Option<super::Metadata>,
    }
    impl ::prost::Name for KnownDenom {
        const NAME: &'static str = "KnownDenom";
        const PACKAGE: &'static str = "penumbra.core.asset.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.asset.v1alpha1.ValueView.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct UnknownDenom {
        #[prost(message, optional, tag = "1")]
        pub amount: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
        #[prost(message, optional, tag = "2")]
        pub asset_id: ::core::option::Option<super::AssetId>,
    }
    impl ::prost::Name for UnknownDenom {
        const NAME: &'static str = "UnknownDenom";
        const PACKAGE: &'static str = "penumbra.core.asset.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.asset.v1alpha1.ValueView.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum ValueView {
        #[prost(message, tag = "1")]
        KnownDenom(KnownDenom),
        #[prost(message, tag = "2")]
        UnknownDenom(UnknownDenom),
    }
}
impl ::prost::Name for ValueView {
    const NAME: &'static str = "ValueView";
    const PACKAGE: &'static str = "penumbra.core.asset.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.asset.v1alpha1.{}", Self::NAME)
    }
}
/// An image related to an asset.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetImage {
    /// The URI of the image in PNG format.
    #[prost(string, tag = "1")]
    pub png: ::prost::alloc::string::String,
    /// The URI of the image in SVG format.
    #[prost(string, tag = "2")]
    pub svg: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub theme: ::core::option::Option<asset_image::Theme>,
}
/// Nested message and enum types in `AssetImage`.
pub mod asset_image {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Theme {
        /// Should be in hex format, `^#\[0-9a-fA-F\]{6}$`.
        #[prost(string, tag = "1")]
        pub primary_color_hex: ::prost::alloc::string::String,
        #[prost(bool, tag = "2")]
        pub circle: bool,
        #[prost(bool, tag = "3")]
        pub dark_mode: bool,
    }
    impl ::prost::Name for Theme {
        const NAME: &'static str = "Theme";
        const PACKAGE: &'static str = "penumbra.core.asset.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.asset.v1alpha1.AssetImage.{}", Self::NAME
            )
        }
    }
}
impl ::prost::Name for AssetImage {
    const NAME: &'static str = "AssetImage";
    const PACKAGE: &'static str = "penumbra.core.asset.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.asset.v1alpha1.{}", Self::NAME)
    }
}
