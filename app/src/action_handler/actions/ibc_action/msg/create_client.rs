use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc_types::core::ics02_client::client_state::ClientState;
use ibc_types::core::ics02_client::msgs::create_client::MsgCreateClient;
use ibc_types::core::ics24_host::identifier::ClientId;
use penumbra_storage::{StateRead, StateWrite};

use crate::action_handler::ActionHandler;
use crate::ibc::client::ics02_validation;
use crate::ibc::component::client::{StateReadExt as _, StateWriteExt as _};
use crate::ibc::{event, ClientCounter};

#[async_trait]
impl ActionHandler for MsgCreateClient {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        client_state_is_tendermint(self)?;
        consensus_state_is_tendermint(self)?;

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        // No-op: IBC actions merge check_stateful and execute.
        Ok(())
    }

    // execute IBC CreateClient.
    //
    //  we compute the client's ID (a concatenation of a monotonically increasing integer, the
    //  number of clients on Penumbra, and the client type) and commit the following to our state:
    // - client type
    // - consensus state
    // - processed time and height
    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        tracing::debug!(msg = ?self);
        let client_state =
            ics02_validation::get_tendermint_client_state(self.client_state.clone())?;

        // get the current client counter
        let id_counter = state.client_counter().await?;
        let client_id = ClientId::new(client_state.client_type(), id_counter.0)?;

        tracing::info!("creating client {:?}", client_id);

        let consensus_state =
            ics02_validation::get_tendermint_consensus_state(self.consensus_state.clone())?;

        // store the client data
        state.put_client(&client_id, client_state.clone());

        // store the genesis consensus state
        state
            .put_verified_consensus_state(
                client_state.latest_height(),
                client_id.clone(),
                consensus_state,
            )
            .await
            .unwrap();

        // increment client counter
        let counter = state.client_counter().await.unwrap_or(ClientCounter(0));
        state.put_client_counter(ClientCounter(counter.0 + 1));

        state.record(event::create_client(client_id, client_state));
        Ok(())
    }
}
fn client_state_is_tendermint(msg: &MsgCreateClient) -> anyhow::Result<()> {
    if ics02_validation::is_tendermint_client_state(&msg.client_state) {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "MsgCreateClient: not a tendermint client state"
        ))
    }
}

fn consensus_state_is_tendermint(msg: &MsgCreateClient) -> anyhow::Result<()> {
    if ics02_validation::is_tendermint_consensus_state(&msg.consensus_state) {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "MsgCreateClient: not a tendermint consensus state"
        ))
    }
}
