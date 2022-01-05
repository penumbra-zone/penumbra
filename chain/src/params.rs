use penumbra_proto::{params, Protobuf};

#[derive(Clone, Debug)]
pub struct ChainParams {
    pub chain_id: String,
    pub epoch_duration: u64,
}

impl Protobuf<params::ChainParams> for ChainParams {}

impl From<params::ChainParams> for ChainParams {
    fn from(msg: params::ChainParams) -> Self {
        ChainParams {
            chain_id: msg.chain_id,
            epoch_duration: msg.epoch_duration,
        }
    }
}

impl From<ChainParams> for params::ChainParams {
    fn from(params: ChainParams) -> Self {
        params::ChainParams {
            chain_id: params.chain_id,
            epoch_duration: params.epoch_duration,
        }
    }
}
