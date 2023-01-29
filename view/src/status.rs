use penumbra_proto::{view::v1alpha1 as pb, Protobuf};

#[derive(Clone, Copy, Debug)]
pub struct StatusStreamResponse {
    pub latest_known_block_height: u64,
    pub sync_height: u64,
}

impl Protobuf for StatusStreamResponse {
    type Proto = pb::StatusStreamResponse;
}

impl TryFrom<pb::StatusStreamResponse> for StatusStreamResponse {
    type Error = anyhow::Error;

    fn try_from(proto: pb::StatusStreamResponse) -> Result<Self, Self::Error> {
        Ok(StatusStreamResponse {
            latest_known_block_height: proto.latest_known_block_height,
            sync_height: proto.sync_height,
        })
    }
}

impl From<StatusStreamResponse> for pb::StatusStreamResponse {
    fn from(msg: StatusStreamResponse) -> Self {
        pb::StatusStreamResponse {
            latest_known_block_height: msg.latest_known_block_height,
            sync_height: msg.sync_height,
        }
    }
}
