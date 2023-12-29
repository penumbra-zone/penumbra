/// A Penumbra address. An address in Penumbra is a Bech32m-encoded
/// string, with the human-readable prefix (HRP) `penumbrav2t`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Address {
    /// The bytes of the address. Must be represented as a series of
    /// `uint8` (i.e. values 0 through 255), with a length of 80 elements.
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
    /// Alternatively, a Bech32m-encoded string representation of the `inner`
    /// bytes.
    ///
    /// NOTE: implementations are not required to support parsing this field.
    /// Implementations should prefer to encode the bytes in all messages they
    /// produce. Implementations must not accept messages with both `inner` and
    /// `alt_bech32m` set.
    #[prost(string, tag = "2")]
    pub alt_bech32m: ::prost::alloc::string::String,
}
impl ::prost::Name for Address {
    const NAME: &'static str = "Address";
    const PACKAGE: &'static str = "penumbra.core.keys.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.keys.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddressView {
    #[prost(oneof = "address_view::AddressView", tags = "1, 2")]
    pub address_view: ::core::option::Option<address_view::AddressView>,
}
/// Nested message and enum types in `AddressView`.
pub mod address_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Visible {
        #[prost(message, optional, tag = "1")]
        pub address: ::core::option::Option<super::Address>,
        #[prost(message, optional, tag = "2")]
        pub index: ::core::option::Option<super::AddressIndex>,
        #[prost(message, optional, tag = "3")]
        pub wallet_id: ::core::option::Option<super::WalletId>,
    }
    impl ::prost::Name for Visible {
        const NAME: &'static str = "Visible";
        const PACKAGE: &'static str = "penumbra.core.keys.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.keys.v1alpha1.AddressView.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub address: ::core::option::Option<super::Address>,
    }
    impl ::prost::Name for Opaque {
        const NAME: &'static str = "Opaque";
        const PACKAGE: &'static str = "penumbra.core.keys.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.keys.v1alpha1.AddressView.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum AddressView {
        #[prost(message, tag = "1")]
        Visible(Visible),
        #[prost(message, tag = "2")]
        Opaque(Opaque),
    }
}
impl ::prost::Name for AddressView {
    const NAME: &'static str = "AddressView";
    const PACKAGE: &'static str = "penumbra.core.keys.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.keys.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PayloadKey {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for PayloadKey {
    const NAME: &'static str = "PayloadKey";
    const PACKAGE: &'static str = "penumbra.core.keys.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.keys.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendKey {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SpendKey {
    const NAME: &'static str = "SpendKey";
    const PACKAGE: &'static str = "penumbra.core.keys.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.keys.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FullViewingKey {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for FullViewingKey {
    const NAME: &'static str = "FullViewingKey";
    const PACKAGE: &'static str = "penumbra.core.keys.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.keys.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WalletId {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for WalletId {
    const NAME: &'static str = "WalletId";
    const PACKAGE: &'static str = "penumbra.core.keys.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.keys.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Diversifier {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Diversifier {
    const NAME: &'static str = "Diversifier";
    const PACKAGE: &'static str = "penumbra.core.keys.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.keys.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddressIndex {
    #[prost(uint32, tag = "2")]
    pub account: u32,
    #[prost(bytes = "vec", tag = "3")]
    pub randomizer: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for AddressIndex {
    const NAME: &'static str = "AddressIndex";
    const PACKAGE: &'static str = "penumbra.core.keys.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.keys.v1alpha1.{}", Self::NAME)
    }
}
/// A validator's identity key (decaf377-rdsa spendauth verification key).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdentityKey {
    #[prost(bytes = "vec", tag = "1")]
    pub ik: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for IdentityKey {
    const NAME: &'static str = "IdentityKey";
    const PACKAGE: &'static str = "penumbra.core.keys.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.keys.v1alpha1.{}", Self::NAME)
    }
}
/// A validator's governance key (decaf377-rdsa spendauth verification key).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GovernanceKey {
    #[prost(bytes = "vec", tag = "1")]
    pub gk: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for GovernanceKey {
    const NAME: &'static str = "GovernanceKey";
    const PACKAGE: &'static str = "penumbra.core.keys.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.keys.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusKey {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ConsensusKey {
    const NAME: &'static str = "ConsensusKey";
    const PACKAGE: &'static str = "penumbra.core.keys.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.keys.v1alpha1.{}", Self::NAME)
    }
}
