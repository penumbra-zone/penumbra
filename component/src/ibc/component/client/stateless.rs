pub mod create_client {
    use super::super::*;

    pub fn client_state_is_tendermint(msg: &MsgCreateAnyClient) -> anyhow::Result<()> {
        downcast!(&msg.client_state => AnyClientState::Tendermint)
            .ok_or_else(|| anyhow::anyhow!("invalid client state: not a Tendermint client state"))
            .map(|_| ())
    }

    pub fn consensus_state_is_tendermint(msg: &MsgCreateAnyClient) -> anyhow::Result<()> {
        downcast!(&msg.consensus_state => AnyConsensusState::Tendermint)
            .ok_or_else(|| {
                anyhow::anyhow!("invalid consensus state: not a Tendermint consensus state")
            })
            .map(|_| ())
    }
}

pub mod update_client {
    use super::super::*;

    pub fn header_is_tendermint(msg: &MsgUpdateAnyClient) -> anyhow::Result<()> {
        downcast!(&msg.header => AnyHeader::Tendermint)
            .ok_or_else(|| anyhow::anyhow!("invalid header: not a Tendermint header"))
            .map(|_| ())
    }
}
