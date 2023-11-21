use penumbra_proto::{view::v1alpha1 as pb, DomainType};

#[derive(Clone, Copy, Debug)]
pub struct StatusStreamResponse {
    pub latest_known_block_height: u64,
    pub full_sync_height: u64,
    pub partial_sync_height: u64,
}

impl DomainType for StatusStreamResponse {
    type Proto = pb::StatusStreamResponse;
}

impl TryFrom<pb::StatusStreamResponse> for StatusStreamResponse {
    type Error = anyhow::Error;

    fn try_from(proto: pb::StatusStreamResponse) -> Result<Self, Self::Error> {
        Ok(StatusStreamResponse {
            latest_known_block_height: proto.latest_known_block_height,
            full_sync_height: proto.full_sync_height,
            partial_sync_height: proto.partial_sync_height,
        })
    }
}

impl From<StatusStreamResponse> for pb::StatusStreamResponse {
    fn from(msg: StatusStreamResponse) -> Self {
        pb::StatusStreamResponse {
            latest_known_block_height: msg.latest_known_block_height,
            full_sync_height: msg.full_sync_height,
            partial_sync_height: msg.partial_sync_height,
        }
    }
}
