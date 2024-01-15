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
    const PACKAGE: &'static str = "penumbra.core.txhash.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.txhash.v1alpha1.{}", Self::NAME)
    }
}
/// A transaction ID, the Sha256 hash of a transaction.
///
/// This is the hash of the plain byte encoding, used by Tendermint.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionId {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for TransactionId {
    const NAME: &'static str = "TransactionId";
    const PACKAGE: &'static str = "penumbra.core.txhash.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.txhash.v1alpha1.{}", Self::NAME)
    }
}
