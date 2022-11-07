/// Describes a validator's configuration data.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    /// The validator's identity verification key.
    #[prost(message, optional, tag="1")]
    pub identity_key: ::core::option::Option<super::super::crypto::v1alpha1::IdentityKey>,
    /// The validator's consensus pubkey for use in Tendermint (Ed25519).
    #[prost(bytes="vec", tag="2")]
    #[serde(with = "crate::serializers::base64str")]
    pub consensus_key: ::prost::alloc::vec::Vec<u8>,
    /// The validator's (human-readable) name.
    #[prost(string, tag="3")]
    pub name: ::prost::alloc::string::String,
    /// The validator's website.
    #[prost(string, tag="4")]
    pub website: ::prost::alloc::string::String,
    /// The validator's description.
    #[prost(string, tag="5")]
    pub description: ::prost::alloc::string::String,
    /// Whether the validator is enabled or not.
    ///
    /// Disabled validators cannot be delegated to, and immediately begin unbonding.
    #[prost(bool, tag="8")]
    pub enabled: bool,
    /// A list of funding streams describing the validator's commission.
    #[prost(message, repeated, tag="6")]
    pub funding_streams: ::prost::alloc::vec::Vec<FundingStream>,
    /// The sequence number determines which validator data takes priority, and
    /// prevents replay attacks.  The chain only accepts new validator definitions
    /// with increasing sequence numbers.
    #[prost(uint32, tag="7")]
    pub sequence_number: u32,
    /// The validator's governance key.
    #[prost(message, optional, tag="9")]
    pub governance_key: ::core::option::Option<super::super::crypto::v1alpha1::GovernanceKey>,
}
/// For storing the list of keys of known validators.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorList {
    #[prost(message, repeated, tag="1")]
    pub validator_keys: ::prost::alloc::vec::Vec<super::super::crypto::v1alpha1::IdentityKey>,
}
/// A portion of a validator's commission.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FundingStream {
    /// The destination address for the funding stream.
    #[prost(string, tag="1")]
    pub address: ::prost::alloc::string::String,
    /// The portion of the staking reward for the entire delegation pool
    /// allocated to this funding stream, specified in basis points.
    #[prost(uint32, tag="2")]
    pub rate_bps: u32,
}
/// Describes the reward and exchange rates and voting power for a validator in some epoch.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RateData {
    #[prost(message, optional, tag="1")]
    pub identity_key: ::core::option::Option<super::super::crypto::v1alpha1::IdentityKey>,
    #[prost(uint64, tag="2")]
    pub epoch_index: u64,
    #[prost(uint64, tag="4")]
    pub validator_reward_rate: u64,
    #[prost(uint64, tag="5")]
    pub validator_exchange_rate: u64,
}
/// Describes the base reward and exchange rates in some epoch.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BaseRateData {
    #[prost(uint64, tag="1")]
    pub epoch_index: u64,
    #[prost(uint64, tag="2")]
    pub base_reward_rate: u64,
    #[prost(uint64, tag="3")]
    pub base_exchange_rate: u64,
}
/// Describes the current state of a validator on-chain
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorStatus {
    #[prost(message, optional, tag="1")]
    pub identity_key: ::core::option::Option<super::super::crypto::v1alpha1::IdentityKey>,
    #[prost(message, optional, tag="2")]
    pub state: ::core::option::Option<ValidatorState>,
    #[prost(uint64, tag="3")]
    pub voting_power: u64,
    #[prost(message, optional, tag="4")]
    pub bonding_state: ::core::option::Option<BondingState>,
}
/// Describes the unbonding state of a validator's stake pool.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BondingState {
    #[prost(enumeration="bonding_state::BondingStateEnum", tag="1")]
    pub state: i32,
    #[prost(uint64, optional, tag="2")]
    pub unbonding_epoch: ::core::option::Option<u64>,
}
/// Nested message and enum types in `BondingState`.
pub mod bonding_state {
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum BondingStateEnum {
        Bonded = 0,
        Unbonding = 1,
        Unbonded = 2,
    }
    impl BondingStateEnum {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                BondingStateEnum::Bonded => "BONDED",
                BondingStateEnum::Unbonding => "UNBONDING",
                BondingStateEnum::Unbonded => "UNBONDED",
            }
        }
    }
}
/// Describes the state of a validator
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorState {
    #[prost(enumeration="validator_state::ValidatorStateEnum", tag="1")]
    pub state: i32,
}
/// Nested message and enum types in `ValidatorState`.
pub mod validator_state {
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum ValidatorStateEnum {
        Inactive = 0,
        Active = 1,
        Jailed = 2,
        Tombstoned = 3,
        Disabled = 4,
    }
    impl ValidatorStateEnum {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                ValidatorStateEnum::Inactive => "INACTIVE",
                ValidatorStateEnum::Active => "ACTIVE",
                ValidatorStateEnum::Jailed => "JAILED",
                ValidatorStateEnum::Tombstoned => "TOMBSTONED",
                ValidatorStateEnum::Disabled => "DISABLED",
            }
        }
    }
}
/// Combines all validator info into a single packet.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorInfo {
    #[prost(message, optional, tag="1")]
    pub validator: ::core::option::Option<Validator>,
    #[prost(message, optional, tag="2")]
    pub status: ::core::option::Option<ValidatorStatus>,
    #[prost(message, optional, tag="3")]
    pub rate_data: ::core::option::Option<RateData>,
}
/// A transaction action (re)defining a validator.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorDefinition {
    /// The configuration data for the validator.
    #[prost(message, optional, tag="1")]
    pub validator: ::core::option::Option<Validator>,
    /// A signature by the validator's identity key over the validator data.
    #[prost(bytes="vec", tag="2")]
    #[serde(with = "crate::serializers::hexstr")]
    pub auth_sig: ::prost::alloc::vec::Vec<u8>,
}
/// A transaction action adding stake to a validator's delegation pool.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Delegate {
    /// The identity key of the validator to delegate to.
    #[prost(message, optional, tag="1")]
    pub validator_identity: ::core::option::Option<super::super::crypto::v1alpha1::IdentityKey>,
    /// The index of the epoch in which this delegation was performed.
    /// The delegation takes effect in the next epoch.
    #[prost(uint64, tag="2")]
    pub epoch_index: u64,
    /// The delegation amount, in units of unbonded stake.
    /// TODO: use flow aggregation to hide this, replacing it with bytes amount_ciphertext;
    #[prost(message, optional, tag="3")]
    pub unbonded_amount: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// The amount of delegation tokens produced by this action.
    ///
    /// This is implied by the validator's exchange rate in the specified epoch
    /// (and should be checked in transaction validation!), but including it allows
    /// stateless verification that the transaction is internally consistent.
    #[prost(message, optional, tag="4")]
    pub delegation_amount: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
}
/// A transaction action withdrawing stake from a validator's delegation pool.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Undelegate {
    /// The identity key of the validator to undelegate from.
    #[prost(message, optional, tag="1")]
    pub validator_identity: ::core::option::Option<super::super::crypto::v1alpha1::IdentityKey>,
    /// The index of the epoch in which this undelegation was performed.
    #[prost(uint64, tag="2")]
    pub epoch_index: u64,
    /// The amount to undelegate, in units of unbonded stake.
    #[prost(message, optional, tag="3")]
    pub unbonded_amount: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// The amount of delegation tokens consumed by this action.
    ///
    /// This is implied by the validator's exchange rate in the specified epoch
    /// (and should be checked in transaction validation!), but including it allows
    /// stateless verification that the transaction is internally consistent.
    #[prost(message, optional, tag="4")]
    pub delegation_amount: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
}
/// A list of pending delegations and undelegations.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegationChanges {
    #[prost(message, repeated, tag="1")]
    pub delegations: ::prost::alloc::vec::Vec<Delegate>,
    #[prost(message, repeated, tag="2")]
    pub undelegations: ::prost::alloc::vec::Vec<Undelegate>,
}
/// Track's a validator's uptime.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Uptime {
    #[prost(uint64, tag="1")]
    pub as_of_block_height: u64,
    #[prost(uint32, tag="2")]
    pub window_len: u32,
    #[prost(bytes="vec", tag="3")]
    #[serde(with = "crate::serializers::base64str")]
    pub bitvec: ::prost::alloc::vec::Vec<u8>,
}
/// Tracks our view of Tendermint's view of the validator set, so we can keep it
/// from getting confused.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CurrentConsensusKeys {
    #[prost(message, repeated, tag="1")]
    pub consensus_keys: ::prost::alloc::vec::Vec<super::super::crypto::v1alpha1::ConsensusKey>,
}
