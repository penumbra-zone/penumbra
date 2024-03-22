use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use ibc_types::core::{
    channel::channel::State as ChannelState, channel::events, channel::msgs::MsgChannelCloseInit,
    channel::PortId, connection::State as ConnectionState,
};

use crate::component::{
    app_handler::{AppHandlerCheck, AppHandlerExecute},
    channel::{StateReadExt as _, StateWriteExt as _},
    connection::StateReadExt as _,
    HostInterface, MsgHandler,
};

#[async_trait]
impl MsgHandler for MsgChannelCloseInit {
    async fn check_stateless<H: AppHandlerCheck>(&self) -> Result<()> {
        // NOTE: no additional stateless validation is possible

        Ok(())
    }

    async fn try_execute<
        S: StateWrite,
        AH: AppHandlerCheck + AppHandlerExecute,
        HI: HostInterface,
    >(
        &self,
        mut state: S,
    ) -> Result<()> {
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
            anyhow::bail!("channel is already closed");
        }

        let connection = state
            .get_connection(&channel.connection_hops[0])
            .await?
            .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;
        if !connection.state_matches(&ConnectionState::Open) {
            anyhow::bail!("connection for channel is not open");
        }
        let transfer = PortId::transfer();
        if self.port_id_on_a == transfer {
            AH::chan_close_init_check(&mut state, self).await?;
        } else {
            anyhow::bail!("invalid port id");
        }

        channel.set_state(ChannelState::Closed);
        state.put_channel(&self.chan_id_on_a, &self.port_id_on_a, channel.clone());

        state.record(
            events::channel::CloseInit {
                port_id: self.port_id_on_a.clone(),
                channel_id: self.chan_id_on_a.clone(),
                counterparty_port_id: channel.counterparty().port_id.clone(),
                counterparty_channel_id: channel
                    .counterparty()
                    .channel_id
                    .clone()
                    .unwrap_or_default(),
                connection_id: channel.connection_hops[0].clone(),
            }
            .into(),
        );

        let transfer = PortId::transfer();
        if self.port_id_on_a == transfer {
            AH::chan_close_init_execute(state, self).await;
        } else {
            anyhow::bail!("invalid port id");
        }

        Ok(())
    }
}
