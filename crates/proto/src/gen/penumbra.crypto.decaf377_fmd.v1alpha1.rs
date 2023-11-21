/// A clue for use with Fuzzy Message Detection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Clue {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Clue {
    const NAME: &'static str = "Clue";
    const PACKAGE: &'static str = "penumbra.crypto.decaf377_fmd.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.crypto.decaf377_fmd.v1alpha1.{}", Self::NAME)
    }
}
