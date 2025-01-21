use penumbra_sdk_proto::penumbra::core::component::sct::v1 as pb;
use penumbra_sdk_proto::DomainType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Epoch", into = "pb::Epoch")]
pub struct Epoch {
    pub index: u64,
    pub start_height: u64,
}

impl DomainType for Epoch {
    type Proto = pb::Epoch;
}

impl From<pb::Epoch> for Epoch {
    fn from(msg: pb::Epoch) -> Self {
        Epoch {
            index: msg.index,
            start_height: msg.start_height,
        }
    }
}

impl From<Epoch> for pb::Epoch {
    fn from(epoch: Epoch) -> Self {
        pb::Epoch {
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
