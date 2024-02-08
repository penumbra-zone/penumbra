use penumbra_dex::{swap::SwapPlaintext, BatchSwapOutputData};
use penumbra_proto::{view::v1 as pb, DomainType};
use penumbra_sct::{CommitmentSource, Nullifier};
use penumbra_tct as tct;
use std::convert::{TryFrom, TryInto};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "pb::SwapRecord", into = "pb::SwapRecord")]
pub struct SwapRecord {
    pub swap_commitment: tct::StateCommitment,
    pub swap: SwapPlaintext,
    pub position: tct::Position,
    pub nullifier: Nullifier,
    pub output_data: BatchSwapOutputData,
    pub height_claimed: Option<u64>,
    pub source: CommitmentSource,
}

impl DomainType for SwapRecord {
    type Proto = pb::SwapRecord;
}
impl From<SwapRecord> for pb::SwapRecord {
    fn from(msg: SwapRecord) -> Self {
        pb::SwapRecord {
            swap_commitment: Some(msg.swap_commitment.into()),
            swap: Some(msg.swap.into()),
            position: msg.position.into(),
            nullifier: Some(msg.nullifier.into()),
            output_data: Some(msg.output_data.into()),
            height_claimed: msg.height_claimed.unwrap_or(0),
            source: Some(msg.source.into()),
        }
    }
}

impl TryFrom<pb::SwapRecord> for SwapRecord {
    type Error = anyhow::Error;
    fn try_from(value: pb::SwapRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            swap_commitment: value
                .swap_commitment
                .ok_or_else(|| anyhow::anyhow!("missing swap_commitment"))?
                .try_into()?,
            swap: value
                .swap
                .ok_or_else(|| anyhow::anyhow!("missing swap"))?
                .try_into()?,
            position: value.position.into(),
            nullifier: value
                .nullifier
                .ok_or_else(|| anyhow::anyhow!("missing nullifier"))?
                .try_into()?,
            output_data: value
                .output_data
                .ok_or_else(|| anyhow::anyhow!("missing output_data"))?
                .try_into()?,
            height_claimed: if value.height_claimed > 0 {
                Some(value.height_claimed)
            } else {
                None
            },
            source: value
                .source
                .ok_or_else(|| anyhow::anyhow!("missing source"))?
                .try_into()?,
        })
    }
}
