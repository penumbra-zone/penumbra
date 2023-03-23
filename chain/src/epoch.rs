use penumbra_proto::{core::chain::v1alpha1 as pb_chain, DomainType};
use serde::{Deserialize, Serialize};

/// Penumbra groups blocks into epochs and restricts validator changes to epoch boundaries.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb_chain::Epoch", into = "pb_chain::Epoch")]
pub struct Epoch {
    pub index: u64,
    pub start_height: u64,
}

impl DomainType for Epoch {
    type Proto = pb_chain::Epoch;
}

impl From<pb_chain::Epoch> for Epoch {
    fn from(msg: pb_chain::Epoch) -> Self {
        Epoch {
            index: msg.index,
            start_height: msg.start_height,
        }
    }
}

impl From<Epoch> for pb_chain::Epoch {
    fn from(epoch: Epoch) -> Self {
        pb_chain::Epoch {
            index: epoch.index,
            start_height: epoch.start_height,
        }
    }
}

impl Epoch {
    // Returns true if current_height is the scheduled last block of the epoch
    pub fn is_scheduled_epoch_end(&self, current_height: u64, epoch_duration: u64) -> bool {
        current_height - self.start_height >= epoch_duration - 1
    }
}
