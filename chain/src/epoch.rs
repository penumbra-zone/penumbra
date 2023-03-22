use penumbra_proto::{core::chain::v1alpha1 as pb_chain, DomainType};
use serde::{Deserialize, Serialize};
use tendermint::block;

/// Penumbra groups blocks into epochs and restricts validator changes to epoch boundaries.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb_chain::Epoch", into = "pb_chain::Epoch")]
pub struct Epoch {
    pub index: u64,
    pub start_height: u64,
    pub end_height: u64,
}

impl DomainType for Epoch {
    type Proto = pb_chain::Epoch;
}

impl From<pb_chain::Epoch> for Epoch {
    fn from(msg: pb_chain::Epoch) -> Self {
        Epoch {
            index: msg.index,
            start_height: msg.start_height,
            end_height: msg.end_height,
        }
    }
}

impl From<Epoch> for pb_chain::Epoch {
    fn from(epoch: Epoch) -> Self {
        pb_chain::Epoch {
            index: epoch.index,
            start_height: epoch.start_height,
            end_height: epoch.end_height,
        }
    }
}

impl Epoch {
    /// Indicates the starting block height for this epoch (inclusive)
    pub fn start_height(&self) -> block::Height {
        block::Height::try_from(self.start_height).expect("able to parse block height")
    }

    /// Indicates the ending block height for this epoch (inclusive)
    pub fn end_height(&self) -> block::Height {
        block::Height::try_from(self.end_height).expect("able to parse block height")
    }

    pub fn is_epoch_end(&self, height: u64) -> bool {
        self.end_height == height
    }
}
