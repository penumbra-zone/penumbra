/// A Penumbra transparent Spend Proof.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendProof {
    /// Auxiliary inputs
    #[prost(message, optional, tag="1")]
    pub note_commitment_proof: ::core::option::Option<super::super::crypto::v1alpha1::NoteCommitmentProof>,
    /// *
    /// @exclude
    /// From the note being spent
    #[prost(message, optional, tag="2")]
    pub note: ::core::option::Option<super::super::crypto::v1alpha1::Note>,
    #[prost(bytes="vec", tag="6")]
    pub v_blinding: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="9")]
    pub spend_auth_randomizer: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="10")]
    pub ak: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="11")]
    pub nk: ::prost::alloc::vec::Vec<u8>,
}
/// A Penumbra transparent output proof.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutputProof {
    /// Auxiliary inputs
    #[prost(message, optional, tag="1")]
    pub note: ::core::option::Option<super::super::crypto::v1alpha1::Note>,
    #[prost(bytes="vec", tag="5")]
    pub v_blinding: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="7")]
    pub esk: ::prost::alloc::vec::Vec<u8>,
}
/// A Penumbra transparent SwapClaimProof.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapClaimProof {
    #[prost(message, optional, tag="2")]
    pub claim_address: ::core::option::Option<super::super::crypto::v1alpha1::Address>,
    /// Inclusion proof for the Swap NFT
    #[prost(message, optional, tag="4")]
    pub note_commitment_proof: ::core::option::Option<super::super::crypto::v1alpha1::NoteCommitmentProof>,
    #[prost(bytes="vec", tag="5")]
    pub note_blinding: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub nk: ::prost::alloc::vec::Vec<u8>,
    /// *
    /// @exclude
    /// Describes opening of Swap NFT asset ID for commitment verification
    #[prost(message, optional, tag="10")]
    pub trading_pair: ::core::option::Option<super::super::dex::v1alpha1::TradingPair>,
    /// uint64 fee = 7; // fee is public data so not included in client's submitted SwapClaimProof
    /// The user's contribution to the swap
    #[prost(uint64, tag="11")]
    pub delta_1_i: u64,
    #[prost(uint64, tag="12")]
    pub delta_2_i: u64,
    /// *
    /// @exclude
    /// Describes output amounts
    #[prost(uint64, tag="20")]
    pub lambda_1_i: u64,
    #[prost(uint64, tag="21")]
    pub lambda_2_i: u64,
    /// *
    /// @exclude
    /// Describes first output note (lambda 1)
    #[prost(bytes="vec", tag="31")]
    pub esk_1: ::prost::alloc::vec::Vec<u8>,
    /// *
    /// @exclude
    /// Describes second output note (lambda 2)
    #[prost(bytes="vec", tag="41")]
    pub esk_2: ::prost::alloc::vec::Vec<u8>,
    /// Swap blinding factor
    #[prost(bytes="vec", tag="42")]
    pub swap_blinding: ::prost::alloc::vec::Vec<u8>,
}
/// A Penumbra transparent SwapProof.
///
/// *
/// @exclude
/// Describes swap inputs
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapProof {
    /// Input amount of asset 1
    #[prost(uint64, tag="1")]
    pub delta_1: u64,
    /// Id of asset 1
    #[prost(bytes="vec", tag="2")]
    pub t1: ::prost::alloc::vec::Vec<u8>,
    /// Input amount of asset 2
    #[prost(uint64, tag="3")]
    pub delta_2: u64,
    /// Id of asset 2
    #[prost(bytes="vec", tag="4")]
    pub t2: ::prost::alloc::vec::Vec<u8>,
    /// Fee
    #[prost(message, optional, tag="10")]
    pub fee: ::core::option::Option<super::super::crypto::v1alpha1::Fee>,
    /// Fee blinding factor.
    #[prost(bytes="vec", tag="11")]
    pub fee_blinding: ::prost::alloc::vec::Vec<u8>,
    // *
    // @exclude
    // Blinding factors for value commitments
    // TODO: not included until flow encryption is available

    // bytes delta_1_blinding = 20;
    // bytes delta_2_blinding = 21;

    // *
    // @exclude
    // Data about the output note recording the Swap NFT.

    /// Address associated with the swap.
    #[prost(message, optional, tag="40")]
    pub claim_address: ::core::option::Option<super::super::crypto::v1alpha1::Address>,
    /// Note blinding factor
    #[prost(bytes="vec", tag="42")]
    pub note_blinding: ::prost::alloc::vec::Vec<u8>,
    /// Ephemeral secret key
    #[prost(bytes="vec", tag="43")]
    pub esk: ::prost::alloc::vec::Vec<u8>,
    /// Swap blinding factor
    #[prost(bytes="vec", tag="44")]
    pub swap_blinding: ::prost::alloc::vec::Vec<u8>,
}
