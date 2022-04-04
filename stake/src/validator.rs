use std::ops::Deref;

use penumbra_crypto::rdsa::{Signature, SpendAuth};
use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::{FundingStream, IdentityKey};

/// Describes a Penumbra validator's configuration data.
///
/// This data is unauthenticated; the [`ValidatorDefinition`] structure includes
/// a signature over the transaction with the validator's identity key.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Validator", into = "pb::Validator")]
pub struct Validator {
    /// The validator's identity verification key.
    pub identity_key: IdentityKey,

    /// The validator's consensus key, used by Tendermint for signing blocks and
    /// other consensus operations.
    pub consensus_key: tendermint::PublicKey,

    /// The validator's (human-readable) name.
    pub name: String,

    /// The validator's website URL.
    pub website: String,

    /// The validator's description.
    pub description: String,

    /// The destinations for the validator's staking reward. The commission is implicitly defined
    /// by the configuration of funding_streams, the sum of FundingStream.rate_bps.
    ///
    // NOTE: unclaimed rewards are tracked by inserting reward notes for the last epoch into the
    // NCT at the beginning of each epoch
    pub funding_streams: FundingStreams,

    /// The sequence number determines which validator data takes priority, and
    /// prevents replay attacks.  The chain only accepts new
    /// [`ValidatorDefinition`]s with increasing sequence numbers, preventing a
    /// third party from replaying previously valid but stale configuration data
    /// as an update.
    pub sequence_number: u32,
}

/// A list of validators.
///
/// This is a newtype wrapper for a Vec that allows us to define a proto type.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorList", into = "pb::ValidatorList")]
pub struct ValidatorList(pub Vec<IdentityKey>);

impl Protobuf<pb::ValidatorList> for ValidatorList {}

impl TryFrom<pb::ValidatorList> for ValidatorList {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ValidatorList) -> Result<Self, Self::Error> {
        Ok(ValidatorList(
            msg.validator_keys
                .iter()
                .map(|key| key.clone().try_into())
                .collect::<anyhow::Result<Vec<_>>>()?,
        ))
    }
}

impl From<ValidatorList> for pb::ValidatorList {
    fn from(vk: ValidatorList) -> Self {
        pb::ValidatorList {
            validator_keys: vk.0.iter().map(|v| v.clone().into()).collect(),
        }
    }
}

/// A set of funding streams to which validators send rewards.
///
/// The total commission of a validator is the sum of the individual reward rate of the
/// [`FundingStream`]s, and cannot exceed 10000bps (100%). This property is guaranteed by the
/// `TryFrom<Vec<FundingStream>` implementation for [`FundingStreams`], which checks the sum, and is
/// the only way to build a non-empty [`FundingStreams`].
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct FundingStreams {
    funding_streams: Vec<FundingStream>,
}

impl FundingStreams {
    pub fn new() -> Self {
        Self {
            funding_streams: Vec::new(),
        }
    }
}

impl TryFrom<Vec<FundingStream>> for FundingStreams {
    type Error = anyhow::Error;

    fn try_from(funding_streams: Vec<FundingStream>) -> Result<Self, Self::Error> {
        if funding_streams.iter().map(|fs| fs.rate_bps).sum::<u16>() > 10_000 {
            return Err(anyhow::anyhow!(
                "sum of funding rates exceeds 100% (10000bps)"
            ));
        }

        Ok(Self { funding_streams })
    }
}

impl From<FundingStreams> for Vec<FundingStream> {
    fn from(funding_streams: FundingStreams) -> Self {
        funding_streams.funding_streams
    }
}

impl AsRef<[FundingStream]> for FundingStreams {
    fn as_ref(&self) -> &[FundingStream] {
        &self.funding_streams
    }
}

impl IntoIterator for FundingStreams {
    type Item = FundingStream;
    type IntoIter = std::vec::IntoIter<FundingStream>;

    fn into_iter(self) -> Self::IntoIter {
        self.funding_streams.into_iter()
    }
}

/// Authenticated configuration data for a validator.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorDefinition", into = "pb::ValidatorDefinition")]
pub struct ValidatorDefinition {
    pub validator: Validator,
    pub auth_sig: Signature<SpendAuth>,
}

impl std::cmp::Ord for ValidatorDefinition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // This is a total ordering on validator definitions, because the
        // signatures can only be equal if the validator definitions are
        // themselves equal.
        self.validator
            .sequence_number
            .cmp(&other.validator.sequence_number)
            .then_with(|| self.auth_sig.to_bytes().cmp(&other.auth_sig.to_bytes()))
    }
}

impl std::cmp::PartialOrd for ValidatorDefinition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A ValidatorDefinition that has had stateful and stateless validation applied
/// and is ready for inclusion into the validator set.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct VerifiedValidatorDefinition(ValidatorDefinition);

impl From<ValidatorDefinition> for VerifiedValidatorDefinition {
    fn from(v: ValidatorDefinition) -> Self {
        VerifiedValidatorDefinition(v)
    }
}

/// Implementing Deref for VerifiedValidatorDefinition allows us to use the
/// inner ValidatorDefinition cleanly.
impl Deref for VerifiedValidatorDefinition {
    type Target = ValidatorDefinition;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::cmp::Ord for VerifiedValidatorDefinition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl std::cmp::PartialOrd for VerifiedValidatorDefinition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Protobuf<pb::Validator> for Validator {}

impl From<Validator> for pb::Validator {
    fn from(v: Validator) -> Self {
        pb::Validator {
            identity_key: Some(v.identity_key.into()),
            consensus_key: v.consensus_key.to_bytes(),
            name: v.name,
            website: v.website,
            description: v.description,
            funding_streams: v.funding_streams.into_iter().map(Into::into).collect(),
            sequence_number: v.sequence_number,
        }
    }
}

impl From<ValidatorDefinition> for Validator {
    fn from(v: ValidatorDefinition) -> Self {
        v.validator
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
            consensus_key: tendermint::PublicKey::from_raw_ed25519(&v.consensus_key)
                .ok_or_else(|| anyhow::anyhow!("invalid ed25519 consensus pubkey"))?,
            name: v.name,
            website: v.website,
            description: v.description,
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

impl Protobuf<pb::ValidatorDefinition> for ValidatorDefinition {}

impl From<ValidatorDefinition> for pb::ValidatorDefinition {
    fn from(v: ValidatorDefinition) -> Self {
        pb::ValidatorDefinition {
            validator: Some(v.validator.into()),
            auth_sig: v.auth_sig.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::ValidatorDefinition> for ValidatorDefinition {
    type Error = anyhow::Error;
    fn try_from(v: pb::ValidatorDefinition) -> Result<Self, Self::Error> {
        Ok(ValidatorDefinition {
            validator: v
                .validator
                .ok_or_else(|| anyhow::anyhow!("missing validator field in proto"))?
                .try_into()?,
            auth_sig: v.auth_sig.as_slice().try_into()?,
        })
    }
}
