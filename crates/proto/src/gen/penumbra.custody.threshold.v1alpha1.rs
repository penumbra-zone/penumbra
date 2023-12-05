/// A key one can use to verify signatures.
///
/// This key can also serve as a unique identifier for users.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerificationKey {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for VerificationKey {
    const NAME: &'static str = "VerificationKey";
    const PACKAGE: &'static str = "penumbra.custody.threshold.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.custody.threshold.v1alpha1.{}", Self::NAME)
    }
}
/// A signature proving that a message was created by the owner of a verification key.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Signature {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Signature {
    const NAME: &'static str = "Signature";
    const PACKAGE: &'static str = "penumbra.custody.threshold.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.custody.threshold.v1alpha1.{}", Self::NAME)
    }
}
/// The message the coordinator sends in round 1 of the signing protocol.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CoordinatorRound1 {
    /// The plan that the coordinator would like the followers to sign.
    #[prost(message, optional, tag = "1")]
    pub plan: ::core::option::Option<
        super::super::super::core::transaction::v1alpha1::TransactionPlan,
    >,
}
impl ::prost::Name for CoordinatorRound1 {
    const NAME: &'static str = "CoordinatorRound1";
    const PACKAGE: &'static str = "penumbra.custody.threshold.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.custody.threshold.v1alpha1.{}", Self::NAME)
    }
}
/// The message the coordinator sends in round 2 of the signing protocol.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CoordinatorRound2 {
    /// The underlying signing packages being sent to the followers, for each signature.
    #[prost(message, repeated, tag = "1")]
    pub signing_packages: ::prost::alloc::vec::Vec<
        coordinator_round2::PartialSigningPackage,
    >,
}
/// Nested message and enum types in `CoordinatorRound2`.
pub mod coordinator_round2 {
    /// A commitment along with a FROST identifier.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct IdentifiedCommitments {
        /// The serialization of a FROST identifier.
        #[prost(bytes = "vec", tag = "1")]
        pub identifier: ::prost::alloc::vec::Vec<u8>,
        /// The commitments this person has produced for this round of signing.
        #[prost(message, optional, tag = "2")]
        pub commitments: ::core::option::Option<
            super::super::super::super::crypto::decaf377_frost::v1alpha1::SigningCommitments,
        >,
    }
    impl ::prost::Name for IdentifiedCommitments {
        const NAME: &'static str = "IdentifiedCommitments";
        const PACKAGE: &'static str = "penumbra.custody.threshold.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.custody.threshold.v1alpha1.CoordinatorRound2.{}", Self::NAME
            )
        }
    }
    /// A FROST signing package without a message.
    ///
    /// We structure things this way because the message is derived from the transaction plan.
    /// FROST expects the signing package to include the identified commitments *and*
    /// the message, but we have no need to include the message.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct PartialSigningPackage {
        #[prost(message, repeated, tag = "1")]
        pub all_commitments: ::prost::alloc::vec::Vec<IdentifiedCommitments>,
    }
    impl ::prost::Name for PartialSigningPackage {
        const NAME: &'static str = "PartialSigningPackage";
        const PACKAGE: &'static str = "penumbra.custody.threshold.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.custody.threshold.v1alpha1.CoordinatorRound2.{}", Self::NAME
            )
        }
    }
}
impl ::prost::Name for CoordinatorRound2 {
    const NAME: &'static str = "CoordinatorRound2";
    const PACKAGE: &'static str = "penumbra.custody.threshold.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.custody.threshold.v1alpha1.{}", Self::NAME)
    }
}
/// The first message the followers send back to the coordinator when signing.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FollowerRound1 {
    #[prost(message, optional, tag = "1")]
    pub inner: ::core::option::Option<follower_round1::Inner>,
    /// The verification key identifying the sender.
    #[prost(message, optional, tag = "2")]
    pub pk: ::core::option::Option<VerificationKey>,
    /// A signature over the proto-encoded bytes of inner.
    #[prost(message, optional, tag = "3")]
    pub sig: ::core::option::Option<Signature>,
}
/// Nested message and enum types in `FollowerRound1`.
pub mod follower_round1 {
    /// The inner message that will be signed by the follower.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Inner {
        /// One signing commitment pair for each signature requested by the plan, in order.
        #[prost(message, repeated, tag = "1")]
        pub commitments: ::prost::alloc::vec::Vec<
            super::super::super::super::crypto::decaf377_frost::v1alpha1::SigningCommitments,
        >,
    }
    impl ::prost::Name for Inner {
        const NAME: &'static str = "Inner";
        const PACKAGE: &'static str = "penumbra.custody.threshold.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.custody.threshold.v1alpha1.FollowerRound1.{}", Self::NAME
            )
        }
    }
}
impl ::prost::Name for FollowerRound1 {
    const NAME: &'static str = "FollowerRound1";
    const PACKAGE: &'static str = "penumbra.custody.threshold.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.custody.threshold.v1alpha1.{}", Self::NAME)
    }
}
/// The second message the followers send back to the coordinator when signing.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FollowerRound2 {
    #[prost(message, optional, tag = "1")]
    pub inner: ::core::option::Option<follower_round2::Inner>,
    /// The verification key identifying the sender.
    #[prost(message, optional, tag = "2")]
    pub pk: ::core::option::Option<VerificationKey>,
    /// A signature over the proto-encoded bytes of inner.
    #[prost(message, optional, tag = "3")]
    pub sig: ::core::option::Option<Signature>,
}
/// Nested message and enum types in `FollowerRound2`.
pub mod follower_round2 {
    /// The inner message that will be signed by the follower.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Inner {
        /// One share for each signature requested by the plan, in order.
        #[prost(message, repeated, tag = "1")]
        pub shares: ::prost::alloc::vec::Vec<
            super::super::super::super::crypto::decaf377_frost::v1alpha1::SignatureShare,
        >,
    }
    impl ::prost::Name for Inner {
        const NAME: &'static str = "Inner";
        const PACKAGE: &'static str = "penumbra.custody.threshold.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.custody.threshold.v1alpha1.FollowerRound2.{}", Self::NAME
            )
        }
    }
}
impl ::prost::Name for FollowerRound2 {
    const NAME: &'static str = "FollowerRound2";
    const PACKAGE: &'static str = "penumbra.custody.threshold.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.custody.threshold.v1alpha1.{}", Self::NAME)
    }
}
/// The first message we broadcast in the DKG protocol.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DkgRound1 {
    /// The package we're sending to other people
    #[prost(message, optional, tag = "1")]
    pub pkg: ::core::option::Option<
        super::super::super::crypto::decaf377_frost::v1alpha1::DkgRound1Package,
    >,
    /// A commitment to a share of the nullifier-deriving key
    #[prost(bytes = "vec", tag = "2")]
    pub nullifier_commitment: ::prost::alloc::vec::Vec<u8>,
    /// An encryption key for the second round.
    #[prost(bytes = "vec", tag = "3")]
    pub epk: ::prost::alloc::vec::Vec<u8>,
    /// A verification key establishing an identity for the sender of this message.
    #[prost(bytes = "vec", tag = "4")]
    pub vk: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for DkgRound1 {
    const NAME: &'static str = "DKGRound1";
    const PACKAGE: &'static str = "penumbra.custody.threshold.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.custody.threshold.v1alpha1.{}", Self::NAME)
    }
}
/// The second message we broadcast in the DKG protocol.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DkgRound2 {
    #[prost(message, optional, tag = "1")]
    pub inner: ::core::option::Option<dkg_round2::Inner>,
    /// The verification key identifying the sender.
    #[prost(bytes = "vec", tag = "2")]
    pub vk: ::prost::alloc::vec::Vec<u8>,
    /// A signature over the proto-encoded inner message.
    #[prost(bytes = "vec", tag = "3")]
    pub sig: ::prost::alloc::vec::Vec<u8>,
}
/// Nested message and enum types in `DKGRound2`.
pub mod dkg_round2 {
    /// A round2 package, encrypted, along with an identifier for the recipient.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct TargetedPackage {
        /// A verification key identifying the recipient.
        #[prost(bytes = "vec", tag = "1")]
        pub vk: ::prost::alloc::vec::Vec<u8>,
        /// The ciphertext of an encrypted frost package for round 2.
        #[prost(bytes = "vec", tag = "2")]
        pub encrypted_package: ::prost::alloc::vec::Vec<u8>,
    }
    impl ::prost::Name for TargetedPackage {
        const NAME: &'static str = "TargetedPackage";
        const PACKAGE: &'static str = "penumbra.custody.threshold.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.custody.threshold.v1alpha1.DKGRound2.{}", Self::NAME
            )
        }
    }
    /// An inner message that will be signed.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Inner {
        /// Encrypted packages for each recipient.
        #[prost(message, repeated, tag = "1")]
        pub encrypted_packages: ::prost::alloc::vec::Vec<TargetedPackage>,
        /// An opening of the share of the nullifier-deriving key commitment
        #[prost(bytes = "vec", tag = "2")]
        pub nullifier: ::prost::alloc::vec::Vec<u8>,
    }
    impl ::prost::Name for Inner {
        const NAME: &'static str = "Inner";
        const PACKAGE: &'static str = "penumbra.custody.threshold.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.custody.threshold.v1alpha1.DKGRound2.{}", Self::NAME
            )
        }
    }
}
impl ::prost::Name for DkgRound2 {
    const NAME: &'static str = "DKGRound2";
    const PACKAGE: &'static str = "penumbra.custody.threshold.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.custody.threshold.v1alpha1.{}", Self::NAME)
    }
}
