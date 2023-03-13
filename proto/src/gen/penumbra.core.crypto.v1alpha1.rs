/// Specifies fees paid by a transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fee {
    /// The amount of the token used to pay fees.
    #[prost(message, optional, tag = "1")]
    pub amount: ::core::option::Option<Amount>,
    /// If present, the asset ID of the token used to pay fees.
    /// If absent, specifies the staking token implicitly.
    #[prost(message, optional, tag = "2")]
    pub asset_id: ::core::option::Option<AssetId>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Address {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendKey {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendVerificationKey {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FullViewingKey {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountId {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Diversifier {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddressIndex {
    #[prost(uint32, tag = "2")]
    pub account: u32,
    #[prost(bytes = "vec", tag = "3")]
    pub randomizer: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StateCommitment {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BalanceCommitment {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetId {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Amount {
    #[prost(uint64, tag = "1")]
    pub lo: u64,
    #[prost(uint64, tag = "2")]
    pub hi: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Denom {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Value {
    #[prost(message, optional, tag = "1")]
    pub amount: ::core::option::Option<Amount>,
    #[prost(message, optional, tag = "2")]
    pub asset_id: ::core::option::Option<AssetId>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MerkleRoot {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Asset {
    #[prost(message, optional, tag = "1")]
    pub id: ::core::option::Option<AssetId>,
    #[prost(message, optional, tag = "2")]
    pub denom: ::core::option::Option<Denom>,
}
/// A validator's identity key (decaf377-rdsa spendauth verification key).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdentityKey {
    #[prost(bytes = "vec", tag = "1")]
    pub ik: ::prost::alloc::vec::Vec<u8>,
}
/// A validator's governance key (decaf377-rdsa spendauth verification key).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GovernanceKey {
    #[prost(bytes = "vec", tag = "1")]
    pub gk: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusKey {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Note {
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<Value>,
    #[prost(bytes = "vec", tag = "2")]
    pub rseed: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub address: ::core::option::Option<Address>,
}
/// An encrypted note.
/// 132 = 1(type) + 11(d) + 8(amount) + 32(asset_id) + 32(rcm) + 32(pk_d) + 16(MAC) bytes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NoteCiphertext {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Nullifier {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendAuthSignature {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BindingSignature {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// The body of an output description, including only the minimal
/// data required to scan and process the output.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NotePayload {
    /// The note commitment for the output note. 32 bytes.
    #[prost(message, optional, tag = "1")]
    pub note_commitment: ::core::option::Option<StateCommitment>,
    /// The encoding of an ephemeral public key. 32 bytes.
    #[prost(bytes = "bytes", tag = "2")]
    pub ephemeral_key: ::prost::bytes::Bytes,
    /// An encryption of the newly created note.
    /// 132 = 1(type) + 11(d) + 8(amount) + 32(asset_id) + 32(rcm) + 32(pk_d) + 16(MAC) bytes.
    #[prost(message, optional, tag = "3")]
    pub encrypted_note: ::core::option::Option<NoteCiphertext>,
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
/// A clue for use with Fuzzy Message Detection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Clue {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// An authorization hash for a Penumbra transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EffectHash {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// A Penumbra ZK output proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkOutputProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// A Penumbra ZK spend proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkSpendProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// A Penumbra ZK swap proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkSwapProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
