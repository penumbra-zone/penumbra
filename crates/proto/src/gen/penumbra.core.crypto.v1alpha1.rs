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
        pub account_group_id: ::core::option::Option<super::AccountGroupId>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub address: ::core::option::Option<super::Address>,
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
pub struct AccountGroupId {
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Denom {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
/// DenomMetadata represents a struct that describes a basic token.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DenomMetadata {
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
    /// URI to a document (on or off-chain) that contains additional information. Optional.
    #[prost(string, tag = "7")]
    pub uri: ::prost::alloc::string::String,
    /// URIHash is a sha256 hash of a document pointed by URI. It's used to verify that
    /// the document didn't change. Optional.
    #[prost(string, tag = "8")]
    pub uri_hash: ::prost::alloc::string::String,
    /// the asset ID on Penumbra for this denomination.
    #[prost(message, optional, tag = "1984")]
    pub penumbra_asset_id: ::core::option::Option<AssetId>,
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Value {
    #[prost(message, optional, tag = "1")]
    pub amount: ::core::option::Option<Amount>,
    #[prost(message, optional, tag = "2")]
    pub asset_id: ::core::option::Option<AssetId>,
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
        pub amount: ::core::option::Option<super::Amount>,
        #[prost(message, optional, tag = "2")]
        pub denom: ::core::option::Option<super::DenomMetadata>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct UnknownDenom {
        #[prost(message, optional, tag = "1")]
        pub amount: ::core::option::Option<super::Amount>,
        #[prost(message, optional, tag = "2")]
        pub asset_id: ::core::option::Option<super::AssetId>,
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MerkleRoot {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NoteView {
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<ValueView>,
    #[prost(bytes = "vec", tag = "2")]
    pub rseed: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub address: ::core::option::Option<AddressView>,
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
/// A Penumbra ZK swap claim proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkSwapClaimProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// A Penumbra ZK undelegate claim proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkUndelegateClaimProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// A Penumbra ZK delegator vote proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkDelegatorVoteProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// A Penumbra ZK nullifier derivation proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkNullifierDerivationProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
