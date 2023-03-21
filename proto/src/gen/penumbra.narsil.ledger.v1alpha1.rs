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
/// A subkey a shard uses to sign messages sent to the ledger.
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
/// A signature over a message sent to the ledger by a shard.
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
/// A self-authenticating `ShardDescription`, signed with the `ShardIdentityKey`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShardOperator {
    #[prost(message, optional, tag = "1")]
    pub description: ::core::option::Option<ShardDescription>,
    #[prost(bytes = "vec", tag = "2")]
    pub sig: ::prost::alloc::vec::Vec<u8>,
}
/// The genesis data describing the set of shard operators who jointly control
/// the Narsil instance.
///
/// The genesis data does not specify the threshold key shares themselves,
/// because these will be computed as the ledger boots up and the shard operators
/// perform the DKG to generate the shared key, described by the `ShardInfo`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisData {
    /// The set of shard operators (implicitly specifying the `n` in `t-of-n`).
    #[prost(message, repeated, tag = "1")]
    pub operators: ::prost::alloc::vec::Vec<ShardOperator>,
    /// The number of shards required to sign a message (the `t` in `t-of-n`).
    #[prost(uint32, tag = "2")]
    pub threshold: u32,
}
/// Describes the Penumbra account group jointly controlled by the Narsil instance.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountGroupInfo {
    /// The full viewing key for the shared account.
    ///
    /// In the Penumbra key hierarchy, this is the highest-authority key below
    /// spend authority, and allows deriving all subkeys for all accounts in the
    /// account group.  It is replicated across all shards.
    ///
    /// The spend verification key component is the `PK` in the FROST I-D.
    #[prost(message, optional, tag = "1")]
    pub full_viewing_key: ::core::option::Option<
        super::super::super::core::crypto::v1alpha1::FullViewingKey,
    >,
    /// Describes the participants in the account group.
    #[prost(message, repeated, tag = "2")]
    pub participants: ::prost::alloc::vec::Vec<ShardInfo>,
}
/// Describes a single shard of the Narsil instance.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShardInfo {
    /// The index of the shard, used for FROST accounting purposes.
    #[prost(uint32, tag = "1")]
    pub index: u32,
    /// The shard verification key, corresponding to `PK_i` in the FROST I-D.
    #[prost(message, optional, tag = "2")]
    pub shard_verification_key: ::core::option::Option<
        super::super::super::core::crypto::v1alpha1::SpendVerificationKey,
    >,
    /// The shard operator's identity key, used to identify the operator of this shard.
    #[prost(message, optional, tag = "3")]
    pub identity_key: ::core::option::Option<ShardIdentityKey>,
}
/// Transaction authorization requests are identified by the proposed
/// transaction's effect hash.
///
/// This acts as a form of content addressing, providing a number of useful
/// behaviors:
///
/// - Multiple users can request authorization of the same `TransactionPlan`, and
///    the ledger can stack their pre-authorizations until some threshold is met.
/// - Rather than having to hold open a connection, clients can re-request
///    authorization of the same `TransactionPlan` after it has been signed, and the
///    ledger can immediately return the already-existing authorization data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestIndex {
    #[prost(message, optional, tag = "1")]
    pub effect_hash: ::core::option::Option<
        super::super::super::core::transaction::v1alpha1::EffectHash,
    >,
}
/// Identifies a particular signing ceremony.
///
/// Ceremonies are identified first by request index and then by a sub-index for
/// the ceremony.  This allows failed or timed-out ceremonies to be repeated.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CeremonyIndex {
    #[prost(message, optional, tag = "1")]
    pub request_index: ::core::option::Option<RequestIndex>,
    #[prost(uint64, tag = "2")]
    pub ceremony_index: u64,
}
/// A committee of shards assigned to carry out a particular signing ceremony.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Committee {
    #[prost(message, optional, tag = "1")]
    pub ceremony: ::core::option::Option<CeremonyIndex>,
    #[prost(message, repeated, tag = "2")]
    pub participants: ::prost::alloc::vec::Vec<ShardInfo>,
}
/// Records a failed ceremony and the reason why it failed.
///
/// TODO: consider filling these in with structured info about the failure
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CeremonyFailure {
    #[prost(oneof = "ceremony_failure::Failure", tags = "1, 2, 3, 4")]
    pub failure: ::core::option::Option<ceremony_failure::Failure>,
}
/// Nested message and enum types in `CeremonyFailure`.
pub mod ceremony_failure {
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
///
/// The ceremony steps are described in the FROST I-D:
/// <https://www.ietf.org/archive/id/draft-irtf-cfrg-frost-11.html>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CeremonyState {
    #[prost(oneof = "ceremony_state::State", tags = "1, 2, 3, 4, 5")]
    pub state: ::core::option::Option<ceremony_state::State>,
}
/// Nested message and enum types in `CeremonyState`.
pub mod ceremony_state {
    /// A ceremony that has not yet started.
    ///
    /// For instance, a request could be queued until sufficient pre-authorizations were recorded on the ledger.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Pending {}
    /// A ceremony that has started round 1.
    ///
    /// The committee has been chosen and the ledger is waiting to record round 1 contributions from all committee members.
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
    /// A ceremony that has started round 2.
    ///
    /// The committee has been chosen, all round 1 commitments have been recorded, and the ledger is waiting to record round 1 contributions from all committee members.
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
    /// A ceremony that has successfully finished.
    ///
    /// The transcript of the ceremony is recorded along with the resulting `AuthorizationData`.
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
            super::super::super::super::core::transaction::v1alpha1::AuthorizationData,
        >,
    }
    /// A ceremony that failed.
    ///
    /// The transcript of the ceremony is recorded along with the reason for the failure.
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
/// A packet of data sent to the Narsil ledger.
///
/// This structure is what Narsil uses as a Tendermint transaction.  However, we
/// use the word "packet" rather than "transaction" here so that it's always
/// unambiguous whether we're referring to data posted to the Penumbra chain or
/// to a Narsil instance.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NarsilPacket {
    #[prost(oneof = "narsil_packet::Packet", tags = "1, 2, 3, 1000, 1001")]
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
        AuthorizeRequest(
            super::super::super::super::custody::v1alpha1::AuthorizeRequest,
        ),
        /// A shard's round 1 contribution to a signing ceremony
        #[prost(message, tag = "2")]
        AuthorizeCommitment(super::AuthorizeCommitment),
        /// A shard's round 2 contribution to a signing ceremony
        #[prost(message, tag = "3")]
        AuthorizeShare(super::AuthorizeShare),
        /// A shard operator's round 1 contribution to the DKG.
        #[prost(message, tag = "1000")]
        DkgRound1(super::DkgRound1),
        /// A shard operator's round 2 contribution to the DKG.
        #[prost(message, tag = "1001")]
        DkgRound2(super::DkgRound2),
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
///
/// This bundle of messages is signed with the shard's `ShardMessageKey` to
/// prevent tampering (e.g., reordering of the internal FROST messages, etc).
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
///
/// This bundle of messages is signed with the shard's `ShardMessageKey` to
/// prevent tampering (e.g., reordering of the internal FROST messages, etc).
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
/// A shard operator's round 1 contribution to the DKG ceremony.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DkgRound1 {
    #[prost(bytes = "vec", tag = "1")]
    pub payload: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub signer: ::core::option::Option<ShardMessageKey>,
    #[prost(message, optional, tag = "3")]
    pub signature: ::core::option::Option<ShardMessageSignature>,
}
/// A shard operator's round 2 contribution to the DKG ceremony.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DkgRound2 {
    #[prost(bytes = "vec", tag = "1")]
    pub payload: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub signer: ::core::option::Option<ShardMessageKey>,
    #[prost(message, optional, tag = "3")]
    pub signature: ::core::option::Option<ShardMessageSignature>,
}
/// The data recorded on-chain about the current state of the DKG ceremony.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DkgState {}
/// Nested message and enum types in `DkgState`.
pub mod dkg_state {
    /// The DKG has started round 1, and the ledger is waiting to record contributions from shard operators.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct StartedRound1 {
        /// A list of round 1 messages received so far (begins empty).
        #[prost(message, repeated, tag = "1")]
        pub round_1_messages: ::prost::alloc::vec::Vec<super::DkgRound1>,
    }
    /// The DKG has started round 2, and the ledger is waiting to record contributions from shard operators.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct StartedRound2 {
        /// A list of messages received during round 1.
        #[prost(message, repeated, tag = "1")]
        pub round_1_messages: ::prost::alloc::vec::Vec<super::DkgRound1>,
        /// A list of round 2 messages received so far (begins empty).
        #[prost(message, repeated, tag = "2")]
        pub round_2_messages: ::prost::alloc::vec::Vec<super::DkgRound2>,
    }
    /// The DKG has finished successfully, producing the jointly-controlled `AccountGroupInfo`.
    ///
    /// Unlike the signing ceremony, we don't record a failure case here: if the DKG fails, we abort the entire ledger.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Finished {
        /// A list of messages received during round 1.
        #[prost(message, repeated, tag = "1")]
        pub round_1_messages: ::prost::alloc::vec::Vec<super::DkgRound1>,
        /// A list of messages received during round 2.
        #[prost(message, repeated, tag = "2")]
        pub round_2_messages: ::prost::alloc::vec::Vec<super::DkgRound2>,
        /// The jointly-controlled `AccountGroupInfo` resulting from the DKG.
        #[prost(message, optional, tag = "3")]
        pub account_group_info: ::core::option::Option<super::AccountGroupInfo>,
    }
}
