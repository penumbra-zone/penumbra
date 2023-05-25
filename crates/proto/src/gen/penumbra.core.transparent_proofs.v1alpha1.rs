/// A Penumbra transparent SwapClaimProof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapClaimProof {
    /// The swap being claimed
    #[prost(message, optional, tag = "1")]
    pub swap_plaintext: ::core::option::Option<
        super::super::dex::v1alpha1::SwapPlaintext,
    >,
    /// Inclusion proof for the swap commitment
    #[prost(message, optional, tag = "4")]
    pub swap_commitment_proof: ::core::option::Option<
        super::super::crypto::v1alpha1::StateCommitmentProof,
    >,
    /// The nullifier key used to derive the swap nullifier
    #[prost(bytes = "vec", tag = "6")]
    pub nk: ::prost::alloc::vec::Vec<u8>,
    /// *
    /// @exclude
    /// Describes output amounts
    #[prost(message, optional, tag = "20")]
    pub lambda_1_i: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    #[prost(message, optional, tag = "21")]
    pub lambda_2_i: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
}
