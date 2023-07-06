use anyhow::Result;
use async_trait::async_trait;
use ibc_types::core::{
    channel::channel::State as ChannelState, channel::msgs::MsgChannelCloseInit, channel::PortId,
    connection::State as ConnectionState,
};
use penumbra_storage::StateWrite;

use crate::{
    component::{
        app_handler::{AppHandlerCheck, AppHandlerExecute},
        channel::{StateReadExt as _, StateWriteExt as _},
        connection::StateReadExt as _,
        transfer::Ics20Transfer,
        MsgHandler,
    },
    event,
};

#[async_trait]
impl MsgHandler for MsgChannelCloseInit {
    async fn check_stateless(&self) -> Result<()> {
        // NOTE: no additional stateless validation is possible

        Ok(())
    }

    async fn try_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        tracing::debug!(msg = ?self);
        // TODO: capability authentication?
        //
        // we probably do need capability authentication here, or some other authorization
        // method, to prevent anyone from spuriously closing channels.
        //
        let mut channel = state
            .get_channel(&self.chan_id_on_a, &self.port_id_on_a)
            .await?
            .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
        if channel.state_matches(&ChannelState::Closed) {
            return Err(anyhow::anyhow!("channel is already closed"));
        }

        let connection = state
            .get_connection(&channel.connection_hops[0])
            .await?
            .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;
        if !connection.state_matches(&ConnectionState::Open) {
            return Err(anyhow::anyhow!("connection for channel is not open"));
        }
        let transfer = PortId::transfer();
        if self.port_id_on_a == transfer {
            Ics20Transfer::chan_close_init_check(&mut state, self).await?;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
        }

        channel.set_state(ChannelState::Closed);
        state.put_channel(&self.chan_id_on_a, &self.port_id_on_a, channel.clone());

        state.record(event::channel_close_init(
            &self.port_id_on_a,
            &self.chan_id_on_a,
            &channel,
        ));

        let transfer = PortId::transfer();
        if self.port_id_on_a == transfer {
            Ics20Transfer::chan_close_init_execute(state, self).await;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
        }

        Ok(())
    }
}
