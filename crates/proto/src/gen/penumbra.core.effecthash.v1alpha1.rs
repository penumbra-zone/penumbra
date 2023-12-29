/// The hash of a Penumbra transaction's *effecting data*, describing the effects
/// of the transaction on the chain state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EffectHash {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for EffectHash {
    const NAME: &'static str = "EffectHash";
    const PACKAGE: &'static str = "penumbra.core.effecthash.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.effecthash.v1alpha1.{}", Self::NAME)
    }
}
