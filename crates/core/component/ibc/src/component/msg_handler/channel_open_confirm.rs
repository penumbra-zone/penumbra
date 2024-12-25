use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use ibc_types::core::{
    channel::channel::State as ChannelState, channel::events, channel::msgs::MsgChannelOpenConfirm,
    channel::ChannelEnd, channel::Counterparty, channel::PortId,
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
impl MsgHandler for MsgChannelOpenConfirm {
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
        let mut channel = state
            .get_channel(&self.chan_id_on_b, &self.port_id_on_b)
            .await?
            .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
        if !channel.state_matches(&ChannelState::TryOpen) {
            anyhow::bail!("channel is not in the correct state");
        }

        // TODO: capability authentication?

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
            state: ChannelState::Open,
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
            AH::chan_open_confirm_check(&mut state, self).await?;
        } else {
            anyhow::bail!("invalid port id");
        }

        channel.set_state(ChannelState::Open);
        state.put_channel(&self.chan_id_on_b, &self.port_id_on_b, channel.clone());

        state.record(
            events::channel::OpenConfirm {
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

        let transfer = PortId::transfer();
        if self.port_id_on_b == transfer {
            AH::chan_open_confirm_execute(state, self).await;
        } else {
            anyhow::bail!("invalid port id");
        }

        Ok(())
    }
}
