/// A Penumbra ZK undelegate claim proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkUndelegateClaimProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ZkUndelegateClaimProof {
    const NAME: &'static str = "ZKUndelegateClaimProof";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Describes a validator's configuration data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    /// The validator's identity verification key.
    #[prost(message, optional, tag = "1")]
    pub identity_key: ::core::option::Option<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
    /// The validator's consensus pubkey for use in Tendermint (Ed25519).
    #[prost(bytes = "vec", tag = "2")]
    pub consensus_key: ::prost::alloc::vec::Vec<u8>,
    /// The validator's (human-readable) name.
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
    /// The validator's website.
    #[prost(string, tag = "4")]
    pub website: ::prost::alloc::string::String,
    /// The validator's description.
    #[prost(string, tag = "5")]
    pub description: ::prost::alloc::string::String,
    /// Whether the validator is enabled or not.
    ///
    /// Disabled validators cannot be delegated to, and immediately begin unbonding.
    #[prost(bool, tag = "8")]
    pub enabled: bool,
    /// A list of funding streams describing the validator's commission.
    #[prost(message, repeated, tag = "6")]
    pub funding_streams: ::prost::alloc::vec::Vec<FundingStream>,
    /// The sequence number determines which validator data takes priority, and
    /// prevents replay attacks.  The chain only accepts new validator definitions
    /// with increasing sequence numbers.
    #[prost(uint32, tag = "7")]
    pub sequence_number: u32,
    /// The validator's governance key.
    #[prost(message, optional, tag = "9")]
    pub governance_key: ::core::option::Option<
        super::super::super::keys::v1alpha1::GovernanceKey,
    >,
}
impl ::prost::Name for Validator {
    const NAME: &'static str = "Validator";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// For storing the list of keys of known validators.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorList {
    #[prost(message, repeated, tag = "1")]
    pub validator_keys: ::prost::alloc::vec::Vec<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
}
impl ::prost::Name for ValidatorList {
    const NAME: &'static str = "ValidatorList";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// A portion of a validator's commission.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FundingStream {
    /// The recipient of the funding stream.
    #[prost(oneof = "funding_stream::Recipient", tags = "1, 2")]
    pub recipient: ::core::option::Option<funding_stream::Recipient>,
}
/// Nested message and enum types in `FundingStream`.
pub mod funding_stream {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ToAddress {
        /// The destination address for the funding stream.
        #[prost(string, tag = "1")]
        pub address: ::prost::alloc::string::String,
        /// The portion of the staking reward for the entire delegation pool
        /// allocated to this funding stream, specified in basis points.
        #[prost(uint32, tag = "2")]
        pub rate_bps: u32,
    }
    impl ::prost::Name for ToAddress {
        const NAME: &'static str = "ToAddress";
        const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.stake.v1alpha1.FundingStream.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ToCommunityPool {
        /// The portion of the staking reward for the entire delegation pool
        /// allocated to this funding stream, specified in basis points.
        #[prost(uint32, tag = "2")]
        pub rate_bps: u32,
    }
    impl ::prost::Name for ToCommunityPool {
        const NAME: &'static str = "ToCommunityPool";
        const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.stake.v1alpha1.FundingStream.{}", Self::NAME
            )
        }
    }
    /// The recipient of the funding stream.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Recipient {
        #[prost(message, tag = "1")]
        ToAddress(ToAddress),
        #[prost(message, tag = "2")]
        ToCommunityPool(ToCommunityPool),
    }
}
impl ::prost::Name for FundingStream {
    const NAME: &'static str = "FundingStream";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Describes the reward and exchange rates and voting power for a validator in some epoch.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RateData {
    #[prost(message, optional, tag = "1")]
    pub identity_key: ::core::option::Option<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
    #[prost(uint64, tag = "2")]
    pub epoch_index: u64,
    #[prost(uint64, tag = "4")]
    pub validator_reward_rate: u64,
    #[prost(uint64, tag = "5")]
    pub validator_exchange_rate: u64,
}
impl ::prost::Name for RateData {
    const NAME: &'static str = "RateData";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Describes the base reward and exchange rates in some epoch.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BaseRateData {
    #[prost(uint64, tag = "1")]
    pub epoch_index: u64,
    #[prost(uint64, tag = "2")]
    pub base_reward_rate: u64,
    #[prost(uint64, tag = "3")]
    pub base_exchange_rate: u64,
}
impl ::prost::Name for BaseRateData {
    const NAME: &'static str = "BaseRateData";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Describes the current state of a validator on-chain
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorStatus {
    #[prost(message, optional, tag = "1")]
    pub identity_key: ::core::option::Option<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
    #[prost(message, optional, tag = "2")]
    pub state: ::core::option::Option<ValidatorState>,
    #[prost(uint64, tag = "3")]
    pub voting_power: u64,
    #[prost(message, optional, tag = "4")]
    pub bonding_state: ::core::option::Option<BondingState>,
}
impl ::prost::Name for ValidatorStatus {
    const NAME: &'static str = "ValidatorStatus";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Describes the unbonding state of a validator's stake pool.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BondingState {
    #[prost(enumeration = "bonding_state::BondingStateEnum", tag = "1")]
    pub state: i32,
    #[prost(uint64, tag = "2")]
    pub unbonding_epoch: u64,
}
/// Nested message and enum types in `BondingState`.
pub mod bonding_state {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum BondingStateEnum {
        Unspecified = 0,
        Bonded = 1,
        Unbonding = 2,
        Unbonded = 3,
    }
    impl BondingStateEnum {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                BondingStateEnum::Unspecified => "BONDING_STATE_ENUM_UNSPECIFIED",
                BondingStateEnum::Bonded => "BONDING_STATE_ENUM_BONDED",
                BondingStateEnum::Unbonding => "BONDING_STATE_ENUM_UNBONDING",
                BondingStateEnum::Unbonded => "BONDING_STATE_ENUM_UNBONDED",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "BONDING_STATE_ENUM_UNSPECIFIED" => Some(Self::Unspecified),
                "BONDING_STATE_ENUM_BONDED" => Some(Self::Bonded),
                "BONDING_STATE_ENUM_UNBONDING" => Some(Self::Unbonding),
                "BONDING_STATE_ENUM_UNBONDED" => Some(Self::Unbonded),
                _ => None,
            }
        }
    }
}
impl ::prost::Name for BondingState {
    const NAME: &'static str = "BondingState";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Describes the state of a validator
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorState {
    #[prost(enumeration = "validator_state::ValidatorStateEnum", tag = "1")]
    pub state: i32,
}
/// Nested message and enum types in `ValidatorState`.
pub mod validator_state {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum ValidatorStateEnum {
        Unspecified = 0,
        Inactive = 1,
        Active = 2,
        Jailed = 3,
        Tombstoned = 4,
        Disabled = 5,
    }
    impl ValidatorStateEnum {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                ValidatorStateEnum::Unspecified => "VALIDATOR_STATE_ENUM_UNSPECIFIED",
                ValidatorStateEnum::Inactive => "VALIDATOR_STATE_ENUM_INACTIVE",
                ValidatorStateEnum::Active => "VALIDATOR_STATE_ENUM_ACTIVE",
                ValidatorStateEnum::Jailed => "VALIDATOR_STATE_ENUM_JAILED",
                ValidatorStateEnum::Tombstoned => "VALIDATOR_STATE_ENUM_TOMBSTONED",
                ValidatorStateEnum::Disabled => "VALIDATOR_STATE_ENUM_DISABLED",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "VALIDATOR_STATE_ENUM_UNSPECIFIED" => Some(Self::Unspecified),
                "VALIDATOR_STATE_ENUM_INACTIVE" => Some(Self::Inactive),
                "VALIDATOR_STATE_ENUM_ACTIVE" => Some(Self::Active),
                "VALIDATOR_STATE_ENUM_JAILED" => Some(Self::Jailed),
                "VALIDATOR_STATE_ENUM_TOMBSTONED" => Some(Self::Tombstoned),
                "VALIDATOR_STATE_ENUM_DISABLED" => Some(Self::Disabled),
                _ => None,
            }
        }
    }
}
impl ::prost::Name for ValidatorState {
    const NAME: &'static str = "ValidatorState";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Combines all validator info into a single packet.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorInfo {
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<Validator>,
    #[prost(message, optional, tag = "2")]
    pub status: ::core::option::Option<ValidatorStatus>,
    #[prost(message, optional, tag = "3")]
    pub rate_data: ::core::option::Option<RateData>,
}
impl ::prost::Name for ValidatorInfo {
    const NAME: &'static str = "ValidatorInfo";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// A transaction action (re)defining a validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorDefinition {
    /// The configuration data for the validator.
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<Validator>,
    /// A signature by the validator's identity key over the validator data.
    #[prost(bytes = "vec", tag = "2")]
    pub auth_sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ValidatorDefinition {
    const NAME: &'static str = "ValidatorDefinition";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// A transaction action adding stake to a validator's delegation pool.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Delegate {
    /// The identity key of the validator to delegate to.
    #[prost(message, optional, tag = "1")]
    pub validator_identity: ::core::option::Option<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
    /// The index of the epoch in which this delegation was performed.
    /// The delegation takes effect in the next epoch.
    #[prost(uint64, tag = "2")]
    pub epoch_index: u64,
    /// The delegation amount, in units of unbonded stake.
    /// TODO: use flow aggregation to hide this, replacing it with bytes amount_ciphertext;
    #[prost(message, optional, tag = "3")]
    pub unbonded_amount: ::core::option::Option<
        super::super::super::num::v1alpha1::Amount,
    >,
    /// The amount of delegation tokens produced by this action.
    ///
    /// This is implied by the validator's exchange rate in the specified epoch
    /// (and should be checked in transaction validation!), but including it allows
    /// stateless verification that the transaction is internally consistent.
    #[prost(message, optional, tag = "4")]
    pub delegation_amount: ::core::option::Option<
        super::super::super::num::v1alpha1::Amount,
    >,
}
impl ::prost::Name for Delegate {
    const NAME: &'static str = "Delegate";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// A transaction action withdrawing stake from a validator's delegation pool.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Undelegate {
    /// The identity key of the validator to undelegate from.
    #[prost(message, optional, tag = "1")]
    pub validator_identity: ::core::option::Option<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
    /// The index of the epoch in which this undelegation was performed.
    #[prost(uint64, tag = "2")]
    pub start_epoch_index: u64,
    /// The amount to undelegate, in units of unbonding tokens.
    #[prost(message, optional, tag = "3")]
    pub unbonded_amount: ::core::option::Option<
        super::super::super::num::v1alpha1::Amount,
    >,
    /// The amount of delegation tokens consumed by this action.
    ///
    /// This is implied by the validator's exchange rate in the specified epoch
    /// (and should be checked in transaction validation!), but including it allows
    /// stateless verification that the transaction is internally consistent.
    #[prost(message, optional, tag = "4")]
    pub delegation_amount: ::core::option::Option<
        super::super::super::num::v1alpha1::Amount,
    >,
}
impl ::prost::Name for Undelegate {
    const NAME: &'static str = "Undelegate";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// A transaction action finishing an undelegation, converting (slashable)
/// "unbonding tokens" to (unslashable) staking tokens.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UndelegateClaim {
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<UndelegateClaimBody>,
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for UndelegateClaim {
    const NAME: &'static str = "UndelegateClaim";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UndelegateClaimBody {
    /// The identity key of the validator to finish undelegating from.
    #[prost(message, optional, tag = "1")]
    pub validator_identity: ::core::option::Option<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
    /// The epoch in which unbonding began, used to verify the penalty.
    #[prost(uint64, tag = "2")]
    pub start_epoch_index: u64,
    /// The penalty applied to undelegation, in bps^2 (10e-8).
    /// In the happy path (no slashing), this is 0.
    #[prost(message, optional, tag = "3")]
    pub penalty: ::core::option::Option<Penalty>,
    /// The action's contribution to the transaction's value balance.
    #[prost(message, optional, tag = "4")]
    pub balance_commitment: ::core::option::Option<
        super::super::super::asset::v1alpha1::BalanceCommitment,
    >,
}
impl ::prost::Name for UndelegateClaimBody {
    const NAME: &'static str = "UndelegateClaimBody";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UndelegateClaimPlan {
    /// The identity key of the validator to finish undelegating from.
    #[prost(message, optional, tag = "1")]
    pub validator_identity: ::core::option::Option<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
    /// The epoch in which unbonding began, used to verify the penalty.
    #[prost(uint64, tag = "2")]
    pub start_epoch_index: u64,
    /// The penalty applied to undelegation, in bps^2 (10e-8).
    /// In the happy path (no slashing), this is 0.
    #[prost(message, optional, tag = "4")]
    pub penalty: ::core::option::Option<Penalty>,
    /// The amount of unbonding tokens to claim.
    /// This is a bare number because its denom is determined by the preceding data.
    #[prost(message, optional, tag = "5")]
    pub unbonding_amount: ::core::option::Option<
        super::super::super::num::v1alpha1::Amount,
    >,
    /// The blinding factor to use for the balance commitment.
    #[prost(bytes = "vec", tag = "6")]
    pub balance_blinding: ::prost::alloc::vec::Vec<u8>,
    /// The first blinding factor to use for the ZK undelegate claim proof.
    #[prost(bytes = "vec", tag = "7")]
    pub proof_blinding_r: ::prost::alloc::vec::Vec<u8>,
    /// The second blinding factor to use for the ZK undelegate claim proof.
    #[prost(bytes = "vec", tag = "8")]
    pub proof_blinding_s: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for UndelegateClaimPlan {
    const NAME: &'static str = "UndelegateClaimPlan";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// A list of pending delegations and undelegations.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegationChanges {
    #[prost(message, repeated, tag = "1")]
    pub delegations: ::prost::alloc::vec::Vec<Delegate>,
    #[prost(message, repeated, tag = "2")]
    pub undelegations: ::prost::alloc::vec::Vec<Undelegate>,
}
impl ::prost::Name for DelegationChanges {
    const NAME: &'static str = "DelegationChanges";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Track's a validator's uptime.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Uptime {
    #[prost(uint64, tag = "1")]
    pub as_of_block_height: u64,
    #[prost(uint32, tag = "2")]
    pub window_len: u32,
    #[prost(bytes = "vec", tag = "3")]
    pub bitvec: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Uptime {
    const NAME: &'static str = "Uptime";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Tracks our view of Tendermint's view of the validator set, so we can keep it
/// from getting confused.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CurrentConsensusKeys {
    #[prost(message, repeated, tag = "1")]
    pub consensus_keys: ::prost::alloc::vec::Vec<
        super::super::super::keys::v1alpha1::ConsensusKey,
    >,
}
impl ::prost::Name for CurrentConsensusKeys {
    const NAME: &'static str = "CurrentConsensusKeys";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Tracks slashing penalties applied to a validator in some epoch.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Penalty {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Penalty {
    const NAME: &'static str = "Penalty";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Requests information on the chain's validators.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorInfoRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// Whether or not to return inactive validators
    #[prost(bool, tag = "2")]
    pub show_inactive: bool,
}
impl ::prost::Name for ValidatorInfoRequest {
    const NAME: &'static str = "ValidatorInfoRequest";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub validator_info: ::core::option::Option<ValidatorInfo>,
}
impl ::prost::Name for ValidatorInfoResponse {
    const NAME: &'static str = "ValidatorInfoResponse";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorStatusRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub identity_key: ::core::option::Option<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
}
impl ::prost::Name for ValidatorStatusRequest {
    const NAME: &'static str = "ValidatorStatusRequest";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorStatusResponse {
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<ValidatorStatus>,
}
impl ::prost::Name for ValidatorStatusResponse {
    const NAME: &'static str = "ValidatorStatusResponse";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Requests the compounded penalty for a validator over a range of epochs.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorPenaltyRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub identity_key: ::core::option::Option<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
    #[prost(uint64, tag = "3")]
    pub start_epoch_index: u64,
    #[prost(uint64, tag = "4")]
    pub end_epoch_index: u64,
}
impl ::prost::Name for ValidatorPenaltyRequest {
    const NAME: &'static str = "ValidatorPenaltyRequest";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorPenaltyResponse {
    #[prost(message, optional, tag = "1")]
    pub penalty: ::core::option::Option<Penalty>,
}
impl ::prost::Name for ValidatorPenaltyResponse {
    const NAME: &'static str = "ValidatorPenaltyResponse";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CurrentValidatorRateRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub identity_key: ::core::option::Option<
        super::super::super::keys::v1alpha1::IdentityKey,
    >,
}
impl ::prost::Name for CurrentValidatorRateRequest {
    const NAME: &'static str = "CurrentValidatorRateRequest";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CurrentValidatorRateResponse {
    #[prost(message, optional, tag = "1")]
    pub data: ::core::option::Option<RateData>,
}
impl ::prost::Name for CurrentValidatorRateResponse {
    const NAME: &'static str = "CurrentValidatorRateResponse";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Staking configuration data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StakeParameters {
    /// The number of epochs an unbonding note for before being released.
    #[prost(uint64, tag = "1")]
    pub unbonding_epochs: u64,
    /// The maximum number of validators in the consensus set.
    #[prost(uint64, tag = "2")]
    pub active_validator_limit: u64,
    /// The base reward rate, expressed in basis points of basis points
    #[prost(uint64, tag = "3")]
    pub base_reward_rate: u64,
    /// The penalty for slashing due to misbehavior.
    #[prost(uint64, tag = "4")]
    pub slashing_penalty_misbehavior: u64,
    /// The penalty for slashing due to downtime.
    #[prost(uint64, tag = "5")]
    pub slashing_penalty_downtime: u64,
    /// The number of blocks in the window to check for downtime.
    #[prost(uint64, tag = "6")]
    pub signed_blocks_window_len: u64,
    /// The maximum number of blocks in the window each validator can miss signing without slashing.
    #[prost(uint64, tag = "7")]
    pub missed_blocks_maximum: u64,
}
impl ::prost::Name for StakeParameters {
    const NAME: &'static str = "StakeParameters";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Genesis data for the staking component.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContent {
    /// The configuration parameters for the staking component present at genesis
    #[prost(message, optional, tag = "1")]
    pub stake_params: ::core::option::Option<StakeParameters>,
    /// The list of validators present at genesis.
    #[prost(message, repeated, tag = "2")]
    pub validators: ::prost::alloc::vec::Vec<Validator>,
}
impl ::prost::Name for GenesisContent {
    const NAME: &'static str = "GenesisContent";
    const PACKAGE: &'static str = "penumbra.core.component.stake.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.stake.v1alpha1.{}", Self::NAME)
    }
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod query_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query operations for the staking component.
    #[derive(Debug, Clone)]
    pub struct QueryServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QueryServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> QueryServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> QueryServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            QueryServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Queries the current validator set, with filtering.
        pub async fn validator_info(
            &mut self,
            request: impl tonic::IntoRequest<super::ValidatorInfoRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::ValidatorInfoResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.stake.v1alpha1.QueryService/ValidatorInfo",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.stake.v1alpha1.QueryService",
                        "ValidatorInfo",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        pub async fn validator_status(
            &mut self,
            request: impl tonic::IntoRequest<super::ValidatorStatusRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ValidatorStatusResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.stake.v1alpha1.QueryService/ValidatorStatus",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.stake.v1alpha1.QueryService",
                        "ValidatorStatus",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn validator_penalty(
            &mut self,
            request: impl tonic::IntoRequest<super::ValidatorPenaltyRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ValidatorPenaltyResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.stake.v1alpha1.QueryService/ValidatorPenalty",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.stake.v1alpha1.QueryService",
                        "ValidatorPenalty",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn current_validator_rate(
            &mut self,
            request: impl tonic::IntoRequest<super::CurrentValidatorRateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CurrentValidatorRateResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.stake.v1alpha1.QueryService/CurrentValidatorRate",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.stake.v1alpha1.QueryService",
                        "CurrentValidatorRate",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod query_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with QueryServiceServer.
    #[async_trait]
    pub trait QueryService: Send + Sync + 'static {
        /// Server streaming response type for the ValidatorInfo method.
        type ValidatorInfoStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::ValidatorInfoResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Queries the current validator set, with filtering.
        async fn validator_info(
            &self,
            request: tonic::Request<super::ValidatorInfoRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::ValidatorInfoStream>,
            tonic::Status,
        >;
        async fn validator_status(
            &self,
            request: tonic::Request<super::ValidatorStatusRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ValidatorStatusResponse>,
            tonic::Status,
        >;
        async fn validator_penalty(
            &self,
            request: tonic::Request<super::ValidatorPenaltyRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ValidatorPenaltyResponse>,
            tonic::Status,
        >;
        async fn current_validator_rate(
            &self,
            request: tonic::Request<super::CurrentValidatorRateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CurrentValidatorRateResponse>,
            tonic::Status,
        >;
    }
    /// Query operations for the staking component.
    #[derive(Debug)]
    pub struct QueryServiceServer<T: QueryService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: QueryService> QueryServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for QueryServiceServer<T>
    where
        T: QueryService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/penumbra.core.component.stake.v1alpha1.QueryService/ValidatorInfo" => {
                    #[allow(non_camel_case_types)]
                    struct ValidatorInfoSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::ServerStreamingService<super::ValidatorInfoRequest>
                    for ValidatorInfoSvc<T> {
                        type Response = super::ValidatorInfoResponse;
                        type ResponseStream = T::ValidatorInfoStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ValidatorInfoRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::validator_info(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ValidatorInfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.core.component.stake.v1alpha1.QueryService/ValidatorStatus" => {
                    #[allow(non_camel_case_types)]
                    struct ValidatorStatusSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::ValidatorStatusRequest>
                    for ValidatorStatusSvc<T> {
                        type Response = super::ValidatorStatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ValidatorStatusRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::validator_status(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ValidatorStatusSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.core.component.stake.v1alpha1.QueryService/ValidatorPenalty" => {
                    #[allow(non_camel_case_types)]
                    struct ValidatorPenaltySvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::ValidatorPenaltyRequest>
                    for ValidatorPenaltySvc<T> {
                        type Response = super::ValidatorPenaltyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ValidatorPenaltyRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::validator_penalty(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ValidatorPenaltySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.core.component.stake.v1alpha1.QueryService/CurrentValidatorRate" => {
                    #[allow(non_camel_case_types)]
                    struct CurrentValidatorRateSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::CurrentValidatorRateRequest>
                    for CurrentValidatorRateSvc<T> {
                        type Response = super::CurrentValidatorRateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CurrentValidatorRateRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::current_validator_rate(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CurrentValidatorRateSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: QueryService> Clone for QueryServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: QueryService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: QueryService> tonic::server::NamedService for QueryServiceServer<T> {
        const NAME: &'static str = "penumbra.core.component.stake.v1alpha1.QueryService";
    }
}
