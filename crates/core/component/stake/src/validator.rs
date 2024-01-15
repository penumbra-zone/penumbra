//! Penumbra validators and related structures.

use penumbra_keys::Address;
use penumbra_proto::{penumbra::core::component::stake::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};
use serde_with::{serde_as, DisplayFromStr};

use crate::{FundingStream, FundingStreams, GovernanceKey, IdentityKey};

mod bonding;
mod definition;
mod info;
mod state;
mod status;

pub use bonding::State as BondingState;
pub use definition::Definition;
pub use info::Info;
pub use state::State;
pub use status::Status;

/// Describes a Penumbra validator's configuration data.
///
/// This data is unauthenticated; the
/// [`ValidatorDefinition`](crate::action::ValidatorDefiniition) action includes
/// a signature over the transaction with the validator's identity key.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Validator", into = "pb::Validator")]
pub struct Validator {
    /// The validator's identity verification key.
    pub identity_key: IdentityKey,

    /// The validator's governance verification key.
    pub governance_key: GovernanceKey,

    /// The validator's consensus key, used by Tendermint for signing blocks and
    /// other consensus operations.
    pub consensus_key: tendermint::PublicKey,

    /// The validator's (human-readable) name.
    pub name: String,

    /// The validator's website URL.
    pub website: String,

    /// The validator's description.
    pub description: String,

    /// Whether the validator is enabled or not.
    ///
    /// Disabled validators cannot be delegated to, and immediately begin unbonding.
    pub enabled: bool,

    /// The destinations for the validator's staking reward. The commission is implicitly defined
    /// by the configuration of funding_streams, the sum of FundingStream.rate_bps.
    ///
    // NOTE: unclaimed rewards are tracked by inserting reward notes for the last epoch into the
    // SCT at the beginning of each epoch
    pub funding_streams: FundingStreams,

    /// The sequence number determines which validator data takes priority, and
    /// prevents replay attacks.  The chain only accepts new
    /// [`ValidatorDefinition`]s with increasing sequence numbers, preventing a
    /// third party from replaying previously valid but stale configuration data
    /// as an update.
    pub sequence_number: u32,
}

#[serde_as]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ValidatorToml {
    /// The sequence number determines which validator data takes priority, and
    /// prevents replay attacks.  The chain only accepts new
    /// [`ValidatorDefinition`]s with increasing sequence numbers, preventing a
    /// third party from replaying previously valid but stale configuration data
    /// as an update.
    pub sequence_number: u32,

    /// Whether the validator is enabled or not.
    ///
    /// Disabled validators cannot be delegated to, and immediately begin unbonding.
    pub enabled: bool,

    /// The validator's (human-readable) name.
    pub name: String,

    /// The validator's website URL.
    pub website: String,

    /// The validator's description.
    pub description: String,

    /// The validator's identity verification key.
    #[serde_as(as = "DisplayFromStr")]
    pub identity_key: IdentityKey,

    /// The validator's governance verification key.
    #[serde_as(as = "DisplayFromStr")]
    pub governance_key: GovernanceKey,

    /// The validator's consensus key, used by Tendermint for signing blocks and
    /// other consensus operations.
    pub consensus_key: tendermint::PublicKey,

    /// The destinations for the validator's staking reward. The commission is implicitly defined
    /// by the configuration of funding_streams, the sum of FundingStream.rate_bps.
    ///
    // NOTE: unclaimed rewards are tracked by inserting reward notes for the last epoch into the
    // SCT at the beginning of each epoch
    #[serde(rename = "funding_stream")]
    pub funding_streams: Vec<FundingStreamToml>,
}

impl From<Validator> for ValidatorToml {
    fn from(v: Validator) -> Self {
        ValidatorToml {
            identity_key: v.identity_key,
            governance_key: v.governance_key,
            consensus_key: v.consensus_key,
            name: v.name,
            website: v.website,
            description: v.description,
            enabled: v.enabled,
            funding_streams: v.funding_streams.into_iter().map(Into::into).collect(),
            sequence_number: v.sequence_number,
        }
    }
}

impl TryFrom<ValidatorToml> for Validator {
    type Error = anyhow::Error;

    fn try_from(v: ValidatorToml) -> anyhow::Result<Self> {
        Ok(Validator {
            identity_key: v.identity_key,
            governance_key: v.governance_key,
            consensus_key: v.consensus_key,
            name: v.name,
            website: v.website,
            description: v.description,
            enabled: v.enabled,
            funding_streams: FundingStreams::try_from(
                v.funding_streams
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>(),
            )?,
            sequence_number: v.sequence_number,
        })
    }
}

/// Human-readable TOML-optimized version of a [`FundingStream`].
#[allow(clippy::large_enum_variant)]
#[serde_as]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FundingStreamToml {
    Address {
        #[serde(rename = "recipient")]
        #[serde_as(as = "DisplayFromStr")]
        address: Address,
        rate_bps: u16,
    },
    CommunityPool {
        recipient: CommunityPool,
        rate_bps: u16,
    },
}

// Unit struct solely to add a `recipient = "CommunityPool"` field to the TOML representation
#[derive(Debug, PartialEq, Eq, Clone, Deserialize_unit_struct, Serialize_unit_struct)]
pub struct CommunityPool;

impl From<FundingStream> for FundingStreamToml {
    fn from(f: FundingStream) -> Self {
        match f {
            FundingStream::ToAddress { address, rate_bps } => {
                FundingStreamToml::Address { address, rate_bps }
            }
            FundingStream::ToCommunityPool { rate_bps } => FundingStreamToml::CommunityPool {
                rate_bps,
                recipient: CommunityPool,
            },
        }
    }
}

impl From<FundingStreamToml> for FundingStream {
    fn from(f: FundingStreamToml) -> Self {
        match f {
            FundingStreamToml::Address { address, rate_bps } => {
                FundingStream::ToAddress { address, rate_bps }
            }
            FundingStreamToml::CommunityPool { rate_bps, .. } => {
                FundingStream::ToCommunityPool { rate_bps }
            }
        }
    }
}

impl DomainType for Validator {
    type Proto = pb::Validator;
}

impl From<Validator> for pb::Validator {
    fn from(v: Validator) -> Self {
        pb::Validator {
            identity_key: Some(v.identity_key.into()),
            governance_key: Some(v.governance_key.into()),
            consensus_key: v.consensus_key.to_bytes(),
            name: v.name,
            website: v.website,
            description: v.description,
            enabled: v.enabled,
            funding_streams: v.funding_streams.into_iter().map(Into::into).collect(),
            sequence_number: v.sequence_number,
        }
    }
}

impl TryFrom<pb::Validator> for Validator {
    type Error = anyhow::Error;
    fn try_from(v: pb::Validator) -> Result<Self, Self::Error> {
        Ok(Validator {
            identity_key: v
                .identity_key
                .ok_or_else(|| anyhow::anyhow!("missing identity key"))?
                .try_into()?,
            governance_key: v
                .governance_key
                .ok_or_else(|| anyhow::anyhow!("missing governance key"))?
                .try_into()?,
            consensus_key: tendermint::PublicKey::from_raw_ed25519(&v.consensus_key)
                .ok_or_else(|| anyhow::anyhow!("invalid ed25519 consensus pubkey"))?,
            name: v.name,
            website: v.website,
            description: v.description,
            enabled: v.enabled,
            funding_streams: v
                .funding_streams
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<FundingStream>, _>>()?
                .try_into()?,
            sequence_number: v.sequence_number,
        })
    }
}
