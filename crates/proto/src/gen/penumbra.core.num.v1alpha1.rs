/// The quantity of a particular Asset. Represented as a 128-bit unsigned integer,
/// split over two fields, `lo` and `hi`, representing the low- and high-order bytes
/// of the 128-bit value, respectively. Clients must assemble these bits in their
/// implementation into a `uint128` or comparable data structure, in order to model
/// the Amount accurately.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Amount {
    #[prost(uint64, tag = "1")]
    pub lo: u64,
    #[prost(uint64, tag = "2")]
    pub hi: u64,
}
impl ::prost::Name for Amount {
    const NAME: &'static str = "Amount";
    const PACKAGE: &'static str = "penumbra.core.num.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.num.v1alpha1.{}", Self::NAME)
    }
}
