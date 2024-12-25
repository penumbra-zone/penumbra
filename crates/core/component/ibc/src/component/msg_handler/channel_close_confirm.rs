use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use ibc_types::core::{
    channel::{
        channel::State as ChannelState, events, msgs::MsgChannelCloseConfirm, ChannelEnd,
        Counterparty, PortId,
    },
    connection::State as ConnectionState,
};

use crate::component::{
    app_handler::{AppHandlerCheck, AppHandlerExecute},
    channel::{StateReadExt as _, StateWriteExt as _},
    connection::StateReadExt as _,
    proof_verification::ChannelProofVerifier,
    HostInterface, MsgHandler,
};

#[async_trait]
impl MsgHandler for MsgChannelCloseConfirm {
    async fn check_stateless<AH>(&self) -> Result<()> {
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
            .get_channel(&self.chan_id_on_b, &self.port_id_on_b)
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

        let expected_connection_hops = vec![connection
            .counterparty
            .connection_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no counterparty connection id provided"))?];

        let expected_counterparty =
            Counterparty::new(self.port_id_on_b.clone(), Some(self.chan_id_on_b.clone()));

        let expected_channel = ChannelEnd {
            state: ChannelState::Closed,
            ordering: channel.ordering,
            remote: expected_counterparty,
            connection_hops: expected_connection_hops,
            version: channel.version.clone(),
            ..ChannelEnd::default()
        };

        state
            .verify_channel_proof(
                &connection,
                &self.proof_chan_end_on_a,
                &self.proof_height_on_a,
                &channel
                    .remote
                    .channel_id
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("no channel id"))?,
                &channel.remote.port_id.clone(),
                &expected_channel,
            )
            .await?;

        let transfer = PortId::transfer();
        if self.port_id_on_b == transfer {
            AH::chan_close_confirm_check(&mut state, self).await?;
        } else {
            anyhow::bail!("invalid port id");
        }
        channel.set_state(ChannelState::Closed);
        state.put_channel(&self.chan_id_on_b, &self.port_id_on_b, channel.clone());

        state.record(
            events::channel::CloseConfirm {
                port_id: self.port_id_on_b.clone(),
                channel_id: self.chan_id_on_b.clone(),
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

        // TODO: should this be part of the handler?
        let transfer = PortId::transfer();
        if self.port_id_on_b == transfer {
            AH::chan_close_confirm_execute(state, self).await;
        } else {
            anyhow::bail!("invalid port id");
        }

        Ok(())
    }
}
