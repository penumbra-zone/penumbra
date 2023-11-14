/// A commitment to a polynomial, as a list of group elements.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifiableSecretSharingCommitment {
    /// Each of these bytes should be the serialization of a group element.
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub elements: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
/// The public package sent in round 1 of the DKG protocol.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DkgRound1Package {
    /// A commitment to the polynomial for secret sharing.
    #[prost(message, optional, tag = "1")]
    pub commitment: ::core::option::Option<VerifiableSecretSharingCommitment>,
    /// A proof of knowledge of the underlying secret being shared.
    #[prost(bytes = "vec", tag = "2")]
    pub proof_of_knowledge: ::prost::alloc::vec::Vec<u8>,
}
/// A share of the final signing key.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SigningShare {
    /// These bytes should be a valid scalar.
    #[prost(bytes = "vec", tag = "1")]
    pub scalar: ::prost::alloc::vec::Vec<u8>,
}
/// The per-participant package sent in round 2 of the DKG protocol.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DkgRound2Package {
    /// This is the share we're sending to that participant.
    #[prost(message, optional, tag = "1")]
    pub signing_share: ::core::option::Option<SigningShare>,
}
/// Represents a commitment to a nonce value.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NonceCommitment {
    /// These bytes should be a valid group element.
    #[prost(bytes = "vec", tag = "1")]
    pub element: ::prost::alloc::vec::Vec<u8>,
}
/// Represents the commitments to nonces needed for signing.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SigningCommitments {
    /// One nonce to hide them.
    #[prost(message, optional, tag = "1")]
    pub hiding: ::core::option::Option<NonceCommitment>,
    /// Another to bind them.
    #[prost(message, optional, tag = "2")]
    pub binding: ::core::option::Option<NonceCommitment>,
}
/// A share of the final signature. These get aggregated to make the actual thing.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignatureShare {
    /// These bytes should be a valid scalar.
    #[prost(bytes = "vec", tag = "1")]
    pub scalar: ::prost::alloc::vec::Vec<u8>,
}
