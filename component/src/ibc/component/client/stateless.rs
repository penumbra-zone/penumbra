pub mod create_client {
    use super::super::*;

    pub fn client_state_is_tendermint(msg: &MsgCreateClient) -> anyhow::Result<()> {
        match msg.client_state.type_url.as_str() {
            TENDERMINT_CLIENT_STATE_TYPE_URL => Ok(()),
            _ => Err(anyhow::anyhow!(
                "invalid client state: not a Tendermint client state"
            )),
        }
    }

    pub fn consensus_state_is_tendermint(msg: &MsgCreateClient) -> anyhow::Result<()> {
        match msg.consensus_state.type_url.as_str() {
            TENDERMINT_CONSENSUS_STATE_TYPE_URL => Ok(()),
            _ => Err(anyhow::anyhow!(
                "invalid consensus state: not a Tendermint consensus state"
            )),
        }
    }
}

pub mod update_client {
    use super::super::*;

    pub fn header_is_tendermint(msg: &MsgUpdateClient) -> anyhow::Result<()> {
        match msg.header.type_url.as_str() {
            TENDERMINT_HEADER_TYPE_URL => Ok(()),
            _ => Err(anyhow::anyhow!("invalid header: not a Tendermint header")),
        }
    }
}
