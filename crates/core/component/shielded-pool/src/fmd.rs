use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use decaf377_fmd::{Clue, Precision};
use penumbra_proto::{
    core::component::shielded_pool::v1::{self as pb},
    DomainType, StateWriteProto,
};
use penumbra_txhash::TransactionId;
use serde::{Deserialize, Serialize};

pub mod state_key;

/// How long users have to switch to updated parameters.
pub const FMD_GRACE_PERIOD_BLOCKS: u64 = 1 << 4;
/// How often we update the params.
pub const FMD_UPDATE_FREQUENCY_BLOCKS: u64 = 1 << 6;
/// How many blocks we expect per day, approximately.
const _BLOCKS_PER_DAY: u64 = 1 << 13;

pub fn should_update_fmd_params(height: u64) -> bool {
    height % FMD_UPDATE_FREQUENCY_BLOCKS == 0
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::FmdParameters", into = "pb::FmdParameters")]
pub struct Parameters {
    /// FMD Precision.
    pub precision: Precision,
    /// The block height at which these parameters became effective.
    pub as_of_block_height: u64,
}

impl DomainType for Parameters {
    type Proto = pb::FmdParameters;
}

impl TryFrom<pb::FmdParameters> for Parameters {
    type Error = anyhow::Error;

    fn try_from(msg: pb::FmdParameters) -> Result<Self> {
        Ok(Parameters {
            precision: msg.precision_bits.try_into()?,
            as_of_block_height: msg.as_of_block_height,
        })
    }
}

impl From<Parameters> for pb::FmdParameters {
    fn from(params: Parameters) -> Self {
        pb::FmdParameters {
            precision_bits: params.precision.bits() as u32,
            as_of_block_height: params.as_of_block_height,
        }
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            precision: Precision::default(),
            as_of_block_height: 1,
        }
    }
}

/// Meta parameters are an algorithm for dynamically choosing FMD parameters.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::FmdMetaParameters", into = "pb::FmdMetaParameters")]
pub enum MetaParameters {
    /// Used a fixed precision forever.
    Fixed(Precision),
}

impl TryFrom<pb::FmdMetaParameters> for MetaParameters {
    type Error = anyhow::Error;

    fn try_from(value: pb::FmdMetaParameters) -> Result<Self> {
        match value.algorithm.ok_or(anyhow!("missing algorithm"))? {
            pb::fmd_meta_parameters::Algorithm::FixedPrecisionBits(p) => {
                Ok(MetaParameters::Fixed(Precision::new(p as u8)?))
            }
        }
    }
}

impl From<MetaParameters> for pb::FmdMetaParameters {
    fn from(value: MetaParameters) -> Self {
        match value {
            MetaParameters::Fixed(p) => pb::FmdMetaParameters {
                algorithm: Some(pb::fmd_meta_parameters::Algorithm::FixedPrecisionBits(
                    p.bits().into(),
                )),
            },
        }
    }
}

impl DomainType for MetaParameters {
    type Proto = pb::FmdMetaParameters;
}

impl Default for MetaParameters {
    fn default() -> Self {
        Self::Fixed(Precision::default())
    }
}

impl MetaParameters {
    pub fn updated_fmd_params(&self, _old: &Parameters, height: u64, _clue_count_delta: (u64, u64)) -> Parameters {
        match *self {
            MetaParameters::Fixed(precision) => Parameters {
                precision,
                as_of_block_height: height,
            },
        }
    }
}

#[async_trait]
trait ClueWriteExt: StateWrite {
    fn put_current_clue_count(&mut self, count: u64) {
        self.put_raw(
            state_key::clue_count::current().to_string(),
            count.to_be_bytes().to_vec(),
        )
    }

    fn put_previous_clue_count(&mut self, count: u64) {
        self.put_raw(
            state_key::clue_count::previous().to_string(),
            count.to_be_bytes().to_vec(),
        )
    }
}

impl<T: StateWrite + ?Sized> ClueWriteExt for T {}

#[async_trait]
trait ClueReadExt: StateRead {
    async fn get_current_clue_count(&self) -> Result<u64> {
        Ok(u64::from_be_bytes(
            self.get_raw(state_key::clue_count::current())
                .await?
                .ok_or(anyhow!("no current clue count"))?
                .as_slice()
                .try_into()?,
        ))
    }

    async fn get_previous_clue_count(&self) -> Result<u64> {
        Ok(u64::from_be_bytes(
            self.get_raw(state_key::clue_count::previous())
                .await?
                .ok_or(anyhow!("no current clue count"))?
                .as_slice()
                .try_into()?,
        ))
    }
}

impl<T: StateRead + ?Sized> ClueReadExt for T {}

#[async_trait]
pub trait ClueManager: StateRead + StateWrite {
    async fn record_clue(&mut self, clue: Clue, tx: TransactionId) -> Result<()> {
        // Update count
        {
            let count = self.get_current_clue_count().await?;
            self.put_current_clue_count(count.saturating_add(1));
        }
        self.record_proto(pb::EventClue {
            clue: Some(clue.into()),
            tx: Some(tx.into())
        });
        Ok(())
    }
}

impl<T: StateRead + StateWrite> ClueManager for T {}

#[async_trait]
pub(crate) trait ClueManagerInternal: ClueManager {
    fn init(&mut self) {
        self.put_current_clue_count(0);
        self.put_previous_clue_count(0);
    }

    /// Flush the clue counts, returning the previous and current counts
    async fn flush_clue_count(&mut self) -> Result<(u64, u64)> {
        let previous = self.get_previous_clue_count().await?;
        let current = self.get_current_clue_count().await?;
        self.put_previous_clue_count(current);
        self.put_current_clue_count(0);
        Ok((previous, current))
    }
}

impl<T: ClueManager> ClueManagerInternal for T {}
