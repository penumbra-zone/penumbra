/// PublicKey defines the keys available for use with Tendermint Validators
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublicKey {
    #[prost(oneof="public_key::Sum", tags="1, 2")]
    pub sum: ::core::option::Option<public_key::Sum>,
}
/// Nested message and enum types in `PublicKey`.
pub mod public_key {
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(bytes, tag="1")]
        Ed25519(::prost::alloc::vec::Vec<u8>),
        #[prost(bytes, tag="2")]
        Secp256k1(::prost::alloc::vec::Vec<u8>),
    }
}
