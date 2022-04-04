use tendermint::block;

/// Epoch represents a given epoch for Penumbra and is used
/// for calculation of staking exchange rates.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Epoch {
    pub index: u64,
    pub duration: u64,
}

impl Epoch {
    /// Instantiates a new `Epoch` from a given block height and epoch duration.
    pub fn from_height(height: u64, epoch_duration: u64) -> Epoch {
        Epoch {
            index: height / epoch_duration,
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

    pub fn is_epoch_end(&self, height: u64) -> bool {
        self.end_height().value() == height
    }

    /// Returns the epoch following this one.
    pub fn next(&self) -> Self {
        Epoch {
            index: self.index + 1,
            duration: self.duration,
        }
    }

    /// Returns the epoch preceding this one.
    pub fn prev(&self) -> Self {
        Epoch {
            index: self.index - 1,
            duration: self.duration,
        }
    }
}
