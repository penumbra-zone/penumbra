use penumbra_proto::{chain as pb, Protobuf};

#[derive(Clone, Debug)]
pub struct ChainParams {
    pub chain_id: String,
    pub epoch_duration: u64,
}

impl Protobuf<pb::ChainParams> for ChainParams {}

impl From<pb::ChainParams> for ChainParams {
    fn from(msg: pb::ChainParams) -> Self {
        ChainParams {
            chain_id: msg.chain_id,
            epoch_duration: msg.epoch_duration,
        }
    }
}

impl From<ChainParams> for pb::ChainParams {
    fn from(params: ChainParams) -> Self {
        pb::ChainParams {
            chain_id: params.chain_id,
            epoch_duration: params.epoch_duration,
        }
    }
}
