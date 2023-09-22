#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Note {
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::super::asset::v1alpha1::Value>,
    #[prost(bytes = "vec", tag = "2")]
    pub rseed: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub address: ::core::option::Option<super::super::super::keys::v1alpha1::Address>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NoteView {
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::super::asset::v1alpha1::ValueView>,
    #[prost(bytes = "vec", tag = "2")]
    pub rseed: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub address: ::core::option::Option<
        super::super::super::keys::v1alpha1::AddressView,
    >,
}
/// An encrypted note.
/// 132 = 1(type) + 11(d) + 8(amount) + 32(asset_id) + 32(rcm) + 32(pk_d) + 16(MAC) bytes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NoteCiphertext {
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
    pub note_commitment: ::core::option::Option<
        super::super::super::super::crypto::tct::v1alpha1::StateCommitment,
    >,
    /// The encoding of an ephemeral public key. 32 bytes.
    #[prost(bytes = "vec", tag = "2")]
    pub ephemeral_key: ::prost::alloc::vec::Vec<u8>,
    /// An encryption of the newly created note.
    /// 132 = 1(type) + 11(d) + 8(amount) + 32(asset_id) + 32(rcm) + 32(pk_d) + 16(MAC) bytes.
    #[prost(message, optional, tag = "3")]
    pub encrypted_note: ::core::option::Option<NoteCiphertext>,
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
/// A Penumbra ZK nullifier derivation proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkNullifierDerivationProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// Spends a shielded note.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Spend {
    /// The effecting data of the spend.
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<SpendBody>,
    /// The authorizing signature for the spend.
    #[prost(message, optional, tag = "2")]
    pub auth_sig: ::core::option::Option<
        super::super::super::super::crypto::decaf377_rdsa::v1alpha1::SpendAuthSignature,
    >,
    /// The proof that the spend is well-formed is authorizing data.
    #[prost(message, optional, tag = "3")]
    pub proof: ::core::option::Option<ZkSpendProof>,
}
/// The body of a spend description, containing only the effecting data
/// describing changes to the ledger, and not the authorizing data that allows
/// those changes to be performed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendBody {
    /// A commitment to the value of the input note.
    #[prost(message, optional, tag = "1")]
    pub balance_commitment: ::core::option::Option<
        super::super::super::asset::v1alpha1::BalanceCommitment,
    >,
    /// The nullifier of the input note.
    #[prost(bytes = "vec", tag = "3")]
    pub nullifier: ::prost::alloc::vec::Vec<u8>,
    /// The randomized validating key for the spend authorization signature.
    #[prost(bytes = "vec", tag = "4")]
    pub rk: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendView {
    #[prost(oneof = "spend_view::SpendView", tags = "1, 2")]
    pub spend_view: ::core::option::Option<spend_view::SpendView>,
}
/// Nested message and enum types in `SpendView`.
pub mod spend_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Visible {
        #[prost(message, optional, tag = "1")]
        pub spend: ::core::option::Option<super::Spend>,
        #[prost(message, optional, tag = "2")]
        pub note: ::core::option::Option<super::NoteView>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub spend: ::core::option::Option<super::Spend>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum SpendView {
        #[prost(message, tag = "1")]
        Visible(Visible),
        #[prost(message, tag = "2")]
        Opaque(Opaque),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendPlan {
    /// The plaintext note we plan to spend.
    #[prost(message, optional, tag = "1")]
    pub note: ::core::option::Option<Note>,
    /// The position of the note we plan to spend.
    #[prost(uint64, tag = "2")]
    pub position: u64,
    /// The randomizer to use for the spend.
    #[prost(bytes = "vec", tag = "3")]
    pub randomizer: ::prost::alloc::vec::Vec<u8>,
    /// The blinding factor to use for the value commitment.
    #[prost(bytes = "vec", tag = "4")]
    pub value_blinding: ::prost::alloc::vec::Vec<u8>,
    /// The first blinding factor to use for the ZK spend proof.
    #[prost(bytes = "vec", tag = "5")]
    pub proof_blinding_r: ::prost::alloc::vec::Vec<u8>,
    /// The second blinding factor to use for the ZK spend proof.
    #[prost(bytes = "vec", tag = "6")]
    pub proof_blinding_s: ::prost::alloc::vec::Vec<u8>,
}
/// Creates a new shielded note.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Output {
    /// The effecting data for the output.
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<OutputBody>,
    /// The output proof is authorizing data.
    #[prost(message, optional, tag = "2")]
    pub proof: ::core::option::Option<ZkOutputProof>,
}
/// The body of an output description, containing only the effecting data
/// describing changes to the ledger, and not the authorizing data that allows
/// those changes to be performed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutputBody {
    /// The minimal data required to scan and process the new output note.
    #[prost(message, optional, tag = "1")]
    pub note_payload: ::core::option::Option<NotePayload>,
    /// A commitment to the value of the output note. 32 bytes.
    #[prost(message, optional, tag = "2")]
    pub balance_commitment: ::core::option::Option<
        super::super::super::asset::v1alpha1::BalanceCommitment,
    >,
    /// An encrypted key for decrypting the memo.
    #[prost(bytes = "vec", tag = "3")]
    pub wrapped_memo_key: ::prost::alloc::vec::Vec<u8>,
    /// The key material used for note encryption, wrapped in encryption to the
    /// sender's outgoing viewing key. 80 bytes.
    #[prost(bytes = "vec", tag = "4")]
    pub ovk_wrapped_key: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutputView {
    #[prost(oneof = "output_view::OutputView", tags = "1, 2")]
    pub output_view: ::core::option::Option<output_view::OutputView>,
}
/// Nested message and enum types in `OutputView`.
pub mod output_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Visible {
        #[prost(message, optional, tag = "1")]
        pub output: ::core::option::Option<super::Output>,
        #[prost(message, optional, tag = "2")]
        pub note: ::core::option::Option<super::NoteView>,
        #[prost(message, optional, tag = "3")]
        pub payload_key: ::core::option::Option<
            super::super::super::super::keys::v1alpha1::PayloadKey,
        >,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub output: ::core::option::Option<super::Output>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum OutputView {
        #[prost(message, tag = "1")]
        Visible(Visible),
        #[prost(message, tag = "2")]
        Opaque(Opaque),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutputPlan {
    /// The value to send to this output.
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::super::asset::v1alpha1::Value>,
    /// The destination address to send it to.
    #[prost(message, optional, tag = "2")]
    pub dest_address: ::core::option::Option<
        super::super::super::keys::v1alpha1::Address,
    >,
    /// The rseed to use for the new note.
    #[prost(bytes = "vec", tag = "3")]
    pub rseed: ::prost::alloc::vec::Vec<u8>,
    /// The blinding factor to use for the value commitment.
    #[prost(bytes = "vec", tag = "4")]
    pub value_blinding: ::prost::alloc::vec::Vec<u8>,
    /// The first blinding factor to use for the ZK output proof.
    #[prost(bytes = "vec", tag = "5")]
    pub proof_blinding_r: ::prost::alloc::vec::Vec<u8>,
    /// The second blinding factor to use for the ZK output proof.
    #[prost(bytes = "vec", tag = "6")]
    pub proof_blinding_s: ::prost::alloc::vec::Vec<u8>,
}
