use anyhow::{anyhow, Context as _};
use pbjson_types::Timestamp;
use penumbra_sdk_tct as tct;
use tct::builder::{block, epoch};

use penumbra_sdk_proto::{
    core::component::sct::v1::{self as pb},
    DomainType, Name as _,
};

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

#[derive(Debug, Clone)]
pub struct EventBlockRoot {
    pub height: u64,
    pub root: block::Root,
    pub timestamp_seconds: i64,
}

impl TryFrom<pb::EventBlockRoot> for EventBlockRoot {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventBlockRoot) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventBlockRoot) -> anyhow::Result<EventBlockRoot> {
            Ok(EventBlockRoot {
                height: value.height,
                root: value.root.ok_or(anyhow!("missing `root`"))?.try_into()?,
                timestamp_seconds: value
                    .timestamp
                    .ok_or(anyhow!("missing `timestamp`"))?
                    .seconds,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventBlockRoot::NAME))
    }
}

impl From<EventBlockRoot> for pb::EventBlockRoot {
    fn from(value: EventBlockRoot) -> Self {
        Self {
            height: value.height,
            root: Some(value.root.into()),
            timestamp: Some(Timestamp {
                seconds: value.timestamp_seconds,
                nanos: 0,
            }),
        }
    }
}

impl DomainType for EventBlockRoot {
    type Proto = pb::EventBlockRoot;
}

#[derive(Debug, Clone)]
pub struct EventEpochRoot {
    pub index: u64,
    pub root: epoch::Root,
    pub timestamp_seconds: i64,
}

impl From<EventEpochRoot> for pb::EventEpochRoot {
    fn from(value: EventEpochRoot) -> Self {
        pb::EventEpochRoot {
            index: value.index,
            root: Some(value.root.into()),
            timestamp: Some(Timestamp {
                seconds: value.timestamp_seconds,
                nanos: 0,
            }),
        }
    }
}

impl TryFrom<pb::EventEpochRoot> for EventEpochRoot {
    type Error = anyhow::Error;

    fn try_from(value: pb::EventEpochRoot) -> Result<Self, Self::Error> {
        fn inner(value: pb::EventEpochRoot) -> anyhow::Result<EventEpochRoot> {
            Ok(EventEpochRoot {
                index: value.index,
                root: value.root.ok_or(anyhow!("missing `root`"))?.try_into()?,
                timestamp_seconds: value
                    .timestamp
                    .ok_or(anyhow!("missing `timestamp`"))?
                    .seconds,
            })
        }
        inner(value).context(format!("parsing {}", pb::EventEpochRoot::NAME))
    }
}

impl DomainType for EventEpochRoot {
    type Proto = pb::EventEpochRoot;
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
