use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc_types::core::{
    ics03_connection::connection::State as ConnectionState,
    ics04_channel::{
        channel::{ChannelEnd, Counterparty, State as ChannelState},
        msgs::chan_close_confirm::MsgChannelCloseConfirm,
    },
    ics24_host::identifier::PortId,
};
use penumbra_storage::{StateRead, StateWrite};

use crate::{
    component::{
        app_handler::{AppHandlerCheck, AppHandlerExecute},
        channel::{
            stateful::proof_verification::ChannelProofVerifier, StateReadExt as _,
            StateWriteExt as _,
        },
        connection::StateReadExt as _,
        transfer::Ics20Transfer,
        ActionHandler,
    },
    event,
};

#[async_trait]
impl ActionHandler for MsgChannelCloseConfirm {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // NOTE: no additional stateless validation is possible

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        // No-op: IBC actions merge check_stateful and execute.
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
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
            return Err(anyhow::anyhow!("channel is already closed"));
        }

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
            state: ChannelState::Closed,
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
            Ics20Transfer::chan_close_confirm_check(&mut state, self).await?;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
        }
        channel.set_state(ChannelState::Closed);
        state.put_channel(&self.chan_id_on_b, &self.port_id_on_b, channel.clone());

        state.record(event::channel_close_confirm(
            &self.port_id_on_b,
            &self.chan_id_on_b,
            &channel,
        ));

        let transfer = PortId::transfer();
        if self.port_id_on_b == transfer {
            Ics20Transfer::chan_close_confirm_execute(state, self).await;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
        }

        Ok(())
    }
}
