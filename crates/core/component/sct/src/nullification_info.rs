use anyhow::anyhow;
use penumbra_sdk_proto::{core::component::sct::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::NullificationInfo", into = "pb::NullificationInfo")]
pub struct NullificationInfo {
    pub id: [u8; 32],
    pub spend_height: u64,
}

impl From<NullificationInfo> for pb::NullificationInfo {
    fn from(value: NullificationInfo) -> Self {
        pb::NullificationInfo {
            id: value.id.to_vec(),
            spend_height: value.spend_height,
        }
    }
}

impl TryFrom<pb::NullificationInfo> for NullificationInfo {
    type Error = anyhow::Error;
    fn try_from(value: pb::NullificationInfo) -> anyhow::Result<Self> {
        Ok(Self {
            id: value
                .id
                .try_into()
                .map_err(|id: Vec<u8>| anyhow!("expected 32-byte id, got {} bytes", id.len()))?,
            spend_height: value.spend_height,
        })
    }
}

impl DomainType for NullificationInfo {
    type Proto = pb::NullificationInfo;
}
