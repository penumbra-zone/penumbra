#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifiableSecretSharingCommitment {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub elements: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DkgRound1Package {
    #[prost(message, optional, tag = "1")]
    pub commitment: ::core::option::Option<VerifiableSecretSharingCommitment>,
    #[prost(bytes = "vec", tag = "2")]
    pub proof_of_knowledge: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SigningShare {
    #[prost(bytes = "vec", tag = "1")]
    pub scalar: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DkgRound2Package {
    #[prost(message, optional, tag = "1")]
    pub signing_share: ::core::option::Option<SigningShare>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NonceCommitment {
    #[prost(bytes = "vec", tag = "1")]
    pub element: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SigningCommitments {
    #[prost(message, optional, tag = "1")]
    pub hiding: ::core::option::Option<NonceCommitment>,
    #[prost(message, optional, tag = "2")]
    pub binding: ::core::option::Option<NonceCommitment>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignatureShare {
    #[prost(bytes = "vec", tag = "1")]
    pub scalar: ::prost::alloc::vec::Vec<u8>,
}
