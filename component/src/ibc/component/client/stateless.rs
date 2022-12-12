pub mod create_client {
    use super::super::*;

    pub fn client_state_is_tendermint(msg: &MsgCreateClient) -> anyhow::Result<()> {
        if ics02_validation::is_tendermint_client_state(&msg.client_state) {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "MsgCreateClient: not a tendermint client state"
            ))
        }
    }

    pub fn consensus_state_is_tendermint(msg: &MsgCreateClient) -> anyhow::Result<()> {
        if ics02_validation::is_tendermint_consensus_state(&msg.consensus_state) {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "MsgCreateClient: not a tendermint consensus state"
            ))
        }
    }
}

pub mod update_client {
    use super::super::*;

    pub fn header_is_tendermint(msg: &MsgUpdateClient) -> anyhow::Result<()> {
        if ics02_validation::is_tendermint_header_state(&msg.header) {
            Ok(())
        } else {
            Err(anyhow::anyhow!("MsgUpdateClient: not a tendermint header"))
        }
    }
}
