use pbjson_types::Timestamp;
use penumbra_tct as tct;
use tct::builder::{block, epoch};

use penumbra_proto::core::component::sct::v1 as pb;

use crate::CommitmentSource;

pub fn anchor(height: u64, anchor: tct::Root, timestamp: i64) -> pb::EventAnchor {
    pb::EventAnchor {
        height,
        anchor: Some(anchor.into()),
        timestamp: Some(Timestamp {
            seconds: timestamp,
            nanos: 0,
        }),
    }
}

pub fn block_root(height: u64, root: block::Root) -> pb::EventBlockRoot {
    pb::EventBlockRoot {
        height,
        root: Some(root.into()),
    }
}

pub fn block_timestamp(height: u64, timestamp: i64) -> pb::EventBlockTimestamp {
    pb::EventBlockTimestamp {
        height,
        timestamp: Some(Timestamp {
            seconds: timestamp,
            nanos: 0,
        }),
    }
}

pub fn epoch_root(index: u64, root: epoch::Root, timestamp: i64) -> pb::EventEpochRoot {
    pb::EventEpochRoot {
        index,
        root: Some(root.into()),
        timestamp: Some(Timestamp {
            seconds: timestamp,
            nanos: 0,
        }),
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
