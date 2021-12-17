use anyhow::{anyhow, Error};

/// Epoch represents a given epoch for Penumbra and is used
/// for calculation of staking exchange rates.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Epoch(pub u64);

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
        Epoch(block_height / epoch_duration)
    }

    pub fn is_epoch_boundary(block_height: i64, epoch_duration: u64) -> Result<bool, Error> {
        if block_height < 0 {
            return Err(anyhow!("block height should never be negative"));
        }

        Ok(block_height.unsigned_abs() % epoch_duration == 0)
    }
}
