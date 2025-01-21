use anyhow::Result;
use penumbra_sdk_proto::{penumbra::core::component::stake::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};
use tendermint::PublicKey;

/// Data structure used to track our view of Tendermint's view of the validator set,
/// so we can keep Tendermint from getting confused about duplicate deletions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(
    try_from = "pb::CurrentConsensusKeys",
    into = "pb::CurrentConsensusKeys"
)]
pub struct CurrentConsensusKeys {
    pub consensus_keys: Vec<PublicKey>,
}

impl DomainType for CurrentConsensusKeys {
    type Proto = pb::CurrentConsensusKeys;
}

impl From<CurrentConsensusKeys> for pb::CurrentConsensusKeys {
    fn from(value: CurrentConsensusKeys) -> pb::CurrentConsensusKeys {
        pb::CurrentConsensusKeys {
            consensus_keys: value.consensus_keys.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<pb::CurrentConsensusKeys> for CurrentConsensusKeys {
    type Error = anyhow::Error;
    fn try_from(value: pb::CurrentConsensusKeys) -> Result<CurrentConsensusKeys> {
        Ok(CurrentConsensusKeys {
            consensus_keys: value
                .consensus_keys
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_>>()?,
        })
    }
}
