/// The root identity key material for a shard operator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShardIdentityKey {
    /// An Ed25519 key.
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// The key that Tendermint will use to identify a validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusKey {
    /// An Ed25519 key.
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShardMessageKey {
    /// An Ed25519 key.
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// The threshold key share controlled by a shard operator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShardKey {
    /// A decaf377 scalar.
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShardMessageSignature {
    /// An Ed25519 signature.
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// A description of one of the operators of a threshold key share (shard).
///
/// The `ShardOperator` message doesn't have the threshold key share itself,
/// because the workflow is that the set of operators is going to be configured
/// first, as part of the genesis / chain configuration, and then the shards
/// perform DKG as the chain boots, using the chain as the messaging layer for
/// the DKG messages.  This means there's no interactive setup procedure for the
/// shard operators.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShardDescription {
    /// The shard operator's offline identity key material which is the root of their authority.
    #[prost(message, optional, tag = "1")]
    pub identity_key: ::core::option::Option<ShardIdentityKey>,
    /// A subkey used for signing messages sent to the ledger.
    #[prost(message, optional, tag = "2")]
    pub message_key: ::core::option::Option<ShardMessageKey>,
    /// The validator's consensus pubkey for use in Tendermint (ed25519)
    #[prost(message, optional, tag = "3")]
    pub consensus_key: ::core::option::Option<ConsensusKey>,
    /// A label for the shard.
    #[prost(string, tag = "4")]
    pub label: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShardOperator {
    #[prost(message, optional, tag = "1")]
    pub description: ::core::option::Option<ShardDescription>,
    #[prost(bytes = "vec", tag = "2")]
    pub sig: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisData {
    #[prost(message, repeated, tag = "1")]
    pub operators: ::prost::alloc::vec::Vec<ShardOperator>,
}
/// Describes the Penumbra account group jointly controlled by the Narsil instance.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountGroupInfo {
    /// The full viewing key for the shared account.
    ///
    /// The spend verification key component is the `PK` in the FROST I-D.
    #[prost(message, optional, tag = "1")]
    pub full_viewing_key: ::core::option::Option<
        super::super::core::crypto::v1alpha1::FullViewingKey,
    >,
    #[prost(message, repeated, tag = "2")]
    pub participants: ::prost::alloc::vec::Vec<ShardInfo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShardInfo {
    #[prost(uint32, tag = "1")]
    pub index: u32,
    /// The shard verification key, corresponding to `PK_i` in the FROST I-D
    #[prost(message, optional, tag = "2")]
    pub shard_verification_key: ::core::option::Option<
        super::super::core::crypto::v1alpha1::SpendVerificationKey,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestIndex {
    #[prost(message, optional, tag = "1")]
    pub effect_hash: ::core::option::Option<
        super::super::core::transaction::v1alpha1::EffectHash,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CeremonyIndex {
    #[prost(uint64, tag = "1")]
    pub ceremony_index: u64,
    #[prost(message, optional, tag = "2")]
    pub request_index: ::core::option::Option<RequestIndex>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Committee {
    #[prost(message, optional, tag = "1")]
    pub ceremony: ::core::option::Option<CeremonyIndex>,
    #[prost(message, repeated, tag = "2")]
    pub participants: ::prost::alloc::vec::Vec<ShardInfo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CeremonyFailure {
    #[prost(oneof = "ceremony_failure::Failure", tags = "1, 2, 3, 4")]
    pub failure: ::core::option::Option<ceremony_failure::Failure>,
}
/// Nested message and enum types in `CeremonyFailure`.
pub mod ceremony_failure {
    /// TODO: consider filling these in with structured info about the failure
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Timeout {}
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct BadCommitment {}
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct BadShare {}
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Canceled {}
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Failure {
        #[prost(message, tag = "1")]
        Timeout(Timeout),
        #[prost(message, tag = "2")]
        BadCommitment(BadCommitment),
        #[prost(message, tag = "3")]
        BadShare(BadShare),
        #[prost(message, tag = "4")]
        Canceled(Canceled),
    }
}
/// The data recorded on-chain about the current state of a signing ceremony.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CeremonyState {
    #[prost(oneof = "ceremony_state::State", tags = "1, 2, 3, 4, 5")]
    pub state: ::core::option::Option<ceremony_state::State>,
}
/// Nested message and enum types in `CeremonyState`.
pub mod ceremony_state {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Pending {}
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct StartedRound1 {
        /// The committee performing the ceremony.
        #[prost(message, optional, tag = "1")]
        pub committee: ::core::option::Option<super::Committee>,
        /// A list of commitment messages received so far (begins empty).
        #[prost(message, repeated, tag = "2")]
        pub commitments: ::prost::alloc::vec::Vec<super::AuthorizeCommitment>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct StartedRound2 {
        /// The committee performing the ceremony.
        #[prost(message, optional, tag = "1")]
        pub committee: ::core::option::Option<super::Committee>,
        /// A list of commitment messages received in round 1.
        #[prost(message, repeated, tag = "2")]
        pub commitments: ::prost::alloc::vec::Vec<super::AuthorizeCommitment>,
        /// A list of authorization share messages received so far (begins empty).
        #[prost(message, repeated, tag = "3")]
        pub shares: ::prost::alloc::vec::Vec<super::AuthorizeShare>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Finished {
        /// The committee performing the ceremony.
        #[prost(message, optional, tag = "1")]
        pub committee: ::core::option::Option<super::Committee>,
        /// A list of commitment messages received in round 1.
        #[prost(message, repeated, tag = "2")]
        pub commitments: ::prost::alloc::vec::Vec<super::AuthorizeCommitment>,
        /// A list of authorization share messages received in round 2.
        #[prost(message, repeated, tag = "3")]
        pub shares: ::prost::alloc::vec::Vec<super::AuthorizeShare>,
        /// The authorization data resulting from the ceremony.
        #[prost(message, optional, tag = "4")]
        pub auth_data: ::core::option::Option<
            super::super::super::core::transaction::v1alpha1::AuthorizationData,
        >,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Failed {
        /// The committee performing the ceremony.
        #[prost(message, optional, tag = "1")]
        pub committee: ::core::option::Option<super::Committee>,
        /// A list of commitment messages received in round 1, if any.
        #[prost(message, repeated, tag = "2")]
        pub commitments: ::prost::alloc::vec::Vec<super::AuthorizeCommitment>,
        /// A list of authorization share messages received in round 2, if any.
        #[prost(message, repeated, tag = "3")]
        pub shares: ::prost::alloc::vec::Vec<super::AuthorizeShare>,
        /// A description of the failure.
        #[prost(message, optional, tag = "4")]
        pub failure: ::core::option::Option<super::CeremonyFailure>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum State {
        #[prost(message, tag = "1")]
        Pending(Pending),
        #[prost(message, tag = "2")]
        StartedRound1(StartedRound1),
        #[prost(message, tag = "3")]
        StartedRound2(StartedRound2),
        #[prost(message, tag = "4")]
        Finished(Finished),
        #[prost(message, tag = "5")]
        Failed(Failed),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NarsilPacket {
    #[prost(oneof = "narsil_packet::Packet", tags = "1, 2, 3")]
    pub packet: ::core::option::Option<narsil_packet::Packet>,
}
/// Nested message and enum types in `NarsilPacket`.
pub mod narsil_packet {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Packet {
        /// An authorization request submitted to the Narsil cluster
        ///
        /// Packet handling:
        /// - check admission policy (black box / ignore for now)
        /// - index the request
        /// - start 1 or more committees to sign it
        #[prost(message, tag = "1")]
        AuthorizeRequest(super::super::super::custody::v1alpha1::AuthorizeRequest),
        /// A shard's round 1 contribution to a signing ceremony
        #[prost(message, tag = "2")]
        AuthorizeCommitment(super::AuthorizeCommitment),
        /// A shard's round 2 contribution to a signing ceremony
        #[prost(message, tag = "3")]
        AuthorizeShare(super::AuthorizeShare),
    }
}
/// A wrapper around the FROST commitment message, exchanged in round 1 of the
/// signing protocol for a single signature.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FrostCommitment {
    #[prost(bytes = "vec", tag = "1")]
    pub payload: ::prost::alloc::vec::Vec<u8>,
}
/// A wrapper around the FROST signature share, exchanged in round 2 of the
/// signing protocol for a single signature.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FrostSignatureShare {
    #[prost(bytes = "vec", tag = "1")]
    pub payload: ::prost::alloc::vec::Vec<u8>,
}
/// A Narsil shard's commitment message for a single ceremony, which may perform
/// multiple signatures (one for each spend in the `AuthorizeRequest`'s
/// `TransactionPlan`).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthorizeCommitment {
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<authorize_commitment::Body>,
    #[prost(message, optional, tag = "2")]
    pub signer: ::core::option::Option<ShardMessageKey>,
    #[prost(message, optional, tag = "3")]
    pub signature: ::core::option::Option<ShardMessageSignature>,
}
/// Nested message and enum types in `AuthorizeCommitment`.
pub mod authorize_commitment {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Body {
        #[prost(message, optional, tag = "1")]
        pub ceremony_index: ::core::option::Option<super::CeremonyIndex>,
        #[prost(message, repeated, tag = "2")]
        pub commitments: ::prost::alloc::vec::Vec<super::FrostCommitment>,
    }
}
/// A Narsil shard's signature share message for a single ceremony, which may perform
/// multiple signatures (one for each spend in the `AuthorizeRequest`'s
/// `TransactionPlan`).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthorizeShare {
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<authorize_share::Body>,
    #[prost(message, optional, tag = "2")]
    pub signer: ::core::option::Option<ShardMessageKey>,
    #[prost(message, optional, tag = "3")]
    pub signature: ::core::option::Option<ShardMessageSignature>,
}
/// Nested message and enum types in `AuthorizeShare`.
pub mod authorize_share {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Body {
        #[prost(message, optional, tag = "1")]
        pub ceremony_index: ::core::option::Option<super::CeremonyIndex>,
        #[prost(message, repeated, tag = "2")]
        pub commitments: ::prost::alloc::vec::Vec<super::FrostCommitment>,
    }
}
