use penumbra_tct as tct;
use tct::builder::{block, epoch};

use penumbra_proto::core::component::sct::v1alpha1 as pb;

use crate::CommitmentSource;

pub fn anchor(height: u64, anchor: tct::Root) -> pb::EventAnchor {
    pb::EventAnchor {
        height,
        anchor: Some(anchor.into()),
    }
}

pub fn block_root(height: u64, root: block::Root) -> pb::EventBlockRoot {
    pb::EventBlockRoot {
        height,
        root: Some(root.into()),
    }
}

pub fn epoch_root(index: u64, root: epoch::Root) -> pb::EventEpochRoot {
    pb::EventEpochRoot {
        index,
        root: Some(root.into()),
    }
}

pub fn commitment(
    commitment: tct::StateCommitment,
    position: tct::Position,
    source: CommitmentSource,
) -> pb::EventCommitment {
    pb::EventCommitment {
        commitment: Some(commitment.into()),
        position: position.into(),
        source: Some(source.into()),
    }
}
