#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StateCommitment {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for StateCommitment {
    const NAME: &'static str = "StateCommitment";
    const PACKAGE: &'static str = "penumbra.crypto.tct.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.crypto.tct.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MerkleRoot {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MerkleRoot {
    const NAME: &'static str = "MerkleRoot";
    const PACKAGE: &'static str = "penumbra.crypto.tct.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.crypto.tct.v1alpha1.{}", Self::NAME)
    }
}
/// An authentication path from a state commitment to the root of the state commitment tree.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StateCommitmentProof {
    #[prost(message, optional, tag = "1")]
    pub note_commitment: ::core::option::Option<StateCommitment>,
    #[prost(uint64, tag = "2")]
    pub position: u64,
    /// always length 24
    #[prost(message, repeated, tag = "3")]
    pub auth_path: ::prost::alloc::vec::Vec<MerklePathChunk>,
}
impl ::prost::Name for StateCommitmentProof {
    const NAME: &'static str = "StateCommitmentProof";
    const PACKAGE: &'static str = "penumbra.crypto.tct.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.crypto.tct.v1alpha1.{}", Self::NAME)
    }
}
/// A set of 3 sibling hashes in the auth path for some note commitment.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MerklePathChunk {
    #[prost(bytes = "vec", tag = "1")]
    pub sibling_1: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub sibling_2: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub sibling_3: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for MerklePathChunk {
    const NAME: &'static str = "MerklePathChunk";
    const PACKAGE: &'static str = "penumbra.crypto.tct.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.crypto.tct.v1alpha1.{}", Self::NAME)
    }
}
