use penumbra_crypto::rdsa::{Signature, SpendAuth, VerificationKey};
use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::FundingStream;

/// Describes a Penumbra validator's configuration data.
///
/// This data is unauthenticated; the [`ValidatorDefinition`] structure includes
/// a signature over the configuration with the validator's identity key.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Validator", into = "pb::Validator")]
pub struct Validator {
    /// The validator's identity verification key.
    pub identity_key: VerificationKey<SpendAuth>,

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
    /// NOTE: sum(FundingRate.rate_bps) should not exceed 100% (10000bps. For now, we ignore this
    /// condition, in the future we should probably make it a slashable offense.
    // NOTE: unclaimed rewards are tracked by inserting reward notes for the last epoch into the
    // NCT at the beginning of each epoch
    pub funding_streams: Vec<FundingStream>,

    /// The sequence number determines which validator data takes priority, and
    /// prevents replay attacks.  The chain only accepts new
    /// [`ValidatorDefinition`]s with increasing sequence numbers, preventing a
    /// third party from replaying previously valid but stale configuration data
    /// as an update.
    pub sequence_number: u32,
}

/// Authenticated configuration data for a validator.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorDefinition", into = "pb::ValidatorDefinition")]
pub struct ValidatorDefinition {
    pub validator: Validator,
    pub auth_sig: Signature<SpendAuth>,
}

impl Protobuf<pb::Validator> for Validator {}

impl From<Validator> for pb::Validator {
    fn from(v: Validator) -> Self {
        pb::Validator {
            identity_key: v.identity_key.to_bytes().to_vec(),
            consensus_key: v.consensus_key.to_bytes(),
            name: v.name,
            website: v.website,
            description: v.description,
            funding_streams: v.funding_streams.into_iter().map(Into::into).collect(),
            sequence_number: v.sequence_number,
        }
    }
}

impl TryFrom<pb::Validator> for Validator {
    type Error = anyhow::Error;
    fn try_from(v: pb::Validator) -> Result<Self, Self::Error> {
        Ok(Validator {
            identity_key: v.identity_key.as_slice().try_into()?,
            consensus_key: tendermint::PublicKey::from_raw_ed25519(&v.consensus_key)
                .ok_or_else(|| anyhow::anyhow!("invalid ed25519 consensus pubkey"))?,
            name: v.name,
            website: v.website,
            description: v.description,
            funding_streams: v
                .funding_streams
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
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
