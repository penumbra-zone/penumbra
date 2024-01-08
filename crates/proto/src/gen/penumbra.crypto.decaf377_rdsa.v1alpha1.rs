#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendAuthSignature {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SpendAuthSignature {
    const NAME: &'static str = "SpendAuthSignature";
    const PACKAGE: &'static str = "penumbra.crypto.decaf377_rdsa.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.crypto.decaf377_rdsa.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BindingSignature {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for BindingSignature {
    const NAME: &'static str = "BindingSignature";
    const PACKAGE: &'static str = "penumbra.crypto.decaf377_rdsa.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.crypto.decaf377_rdsa.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendVerificationKey {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SpendVerificationKey {
    const NAME: &'static str = "SpendVerificationKey";
    const PACKAGE: &'static str = "penumbra.crypto.decaf377_rdsa.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.crypto.decaf377_rdsa.v1alpha1.{}", Self::NAME)
    }
}
