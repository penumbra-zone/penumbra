use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc_types::core::ics03_connection::connection::State as ConnectionState;
use ibc_types::core::ics04_channel::channel::State as ChannelState;
use ibc_types::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
use ibc_types::core::ics24_host::identifier::PortId;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::Transaction;

use crate::action_handler::ActionHandler;
use crate::ibc::component::channel::{StateReadExt as _, StateWriteExt as _};
use crate::ibc::component::connection::StateReadExt as _;
use crate::ibc::event;
use crate::ibc::ibc_handler::{AppHandlerCheck, AppHandlerExecute};
use crate::ibc::transfer::Ics20Transfer;

#[async_trait]
impl ActionHandler for MsgChannelCloseInit {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
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
