use anyhow::{anyhow, Error};

use tendermint::block;

/// Epoch represents a given epoch for Penumbra and is used
/// for calculation of staking exchange rates.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Epoch {
    pub index: u64,
    pub duration: u64,
}

impl Epoch {
    /// from_blockheight instantiates a new `Epoch` from a given
    /// block height. Due to the implementation in tendermint using
    /// signed representation for block height, we provide this
    /// as well as an unsigned implemention (`from_blockheight_unsigned`)
    pub fn from_blockheight(block_height: i64, epoch_duration: u64) -> Result<Self, Error> {
        if block_height < 0 {
            return Err(anyhow!("block height should never be negative"));
        }

        Ok(Epoch::from_blockheight_unsigned(
            block_height.unsigned_abs(),
            epoch_duration,
        ))
    }

    /// from_blockheight_unsigned instantiates a new `Epoch` from a given
    /// unsigned block height. Due to the implementation in tendermint using
    /// signed representation for block height, we provide this
    /// as well as a signed implemention (`from_blockheight`)
    pub fn from_blockheight_unsigned(block_height: u64, epoch_duration: u64) -> Self {
        Epoch {
            index: block_height / epoch_duration,
            duration: epoch_duration,
        }
    }

    /// Indicates the starting block height for this epoch (inclusive)
    pub fn start_height(&self) -> block::Height {
        block::Height::try_from(self.index * self.duration).expect("able to parse block height")
    }

    /// Indicates the ending block height for this epoch (inclusive)
    pub fn end_height(&self) -> block::Height {
        block::Height::try_from((self.index + 1) * self.duration - 1)
            .expect("able to parse block height")
    }

    pub fn is_epoch_boundary(block_height: i64, epoch_duration: u64) -> Result<bool, Error> {
        if block_height < 0 {
            return Err(anyhow!("block height should never be negative"));
        }

        Ok(block_height.unsigned_abs() % epoch_duration == 0)
    }
}
