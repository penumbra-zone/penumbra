use anyhow::Result;
use async_trait::async_trait;
use ibc_types::core::{
    ics03_connection::connection::State as ConnectionState,
    ics04_channel::{
        channel::{ChannelEnd, Counterparty, State as ChannelState},
        msgs::chan_open_confirm::MsgChannelOpenConfirm,
    },
    ics24_host::identifier::PortId,
};
use penumbra_storage::StateWrite;

use crate::{
    component::{
        app_handler::{AppHandlerCheck, AppHandlerExecute},
        channel::{
            stateful::proof_verification::ChannelProofVerifier, StateReadExt as _,
            StateWriteExt as _,
        },
        connection::StateReadExt as _,
        transfer::Ics20Transfer,
        MsgHandler,
    },
    event,
};

#[async_trait]
impl MsgHandler for MsgChannelOpenConfirm {
    async fn check_stateless(&self) -> Result<()> {
        // NOTE: no additional stateless validation is possible

        Ok(())
    }

    async fn try_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        tracing::debug!(msg = ?self);
        let mut channel = state
            .get_channel(&self.chan_id_on_b, &self.port_id_on_b)
            .await?
            .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
        if !channel.state_matches(&ChannelState::TryOpen) {
            return Err(anyhow::anyhow!("channel is not in the correct state"));
        }

        // TODO: capability authentication?

        let connection = state
            .get_connection(&channel.connection_hops[0])
            .await?
            .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;
        if !connection.state_matches(&ConnectionState::Open) {
            return Err(anyhow::anyhow!("connection for channel is not open"));
        }

        let expected_connection_hops = vec![connection
            .counterparty()
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
            Ics20Transfer::chan_open_confirm_check(&mut state, self).await?;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
        }

        channel.set_state(ChannelState::Open);
        state.put_channel(&self.chan_id_on_b, &self.port_id_on_b, channel.clone());

        state.record(event::channel_open_confirm(
            &self.port_id_on_b,
            &self.chan_id_on_b,
            &channel,
        ));

        let transfer = PortId::transfer();
        if self.port_id_on_b == transfer {
            Ics20Transfer::chan_open_confirm_execute(state, self).await;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
        }

        Ok(())
    }
}
