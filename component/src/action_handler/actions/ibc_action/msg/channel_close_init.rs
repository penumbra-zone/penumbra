use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
use ibc::core::ics24_host::identifier::PortId;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::Transaction;
use tracing::instrument;

use crate::action_handler::ActionHandler;
use crate::ibc::component::channel::execution::channel_close_init::ChannelCloseInitExecute;
use crate::ibc::component::channel::stateful::channel_close_init::ChannelCloseInitCheck;
use crate::ibc::ibc_handler::{AppHandlerCheck, AppHandlerExecute};
use crate::ibc::transfer::Ics20Transfer;

#[async_trait]
impl ActionHandler for MsgChannelCloseInit {
    #[instrument(name = "channel_close_init", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        // NOTE: no additional stateless validation is possible

        Ok(())
    }

    #[instrument(name = "channel_close_init", skip(self, state, _context))]
    async fn check_stateful(&self, state: Arc<State>, _context: Arc<Transaction>) -> Result<()> {
        state.validate(self).await?;
        let transfer = PortId::transfer();
        if self.port_id == transfer {
            Ics20Transfer::chan_close_init_check(state, self).await?;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
        }

        Ok(())
    }

    #[instrument(name = "channel_close_init", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        state.execute(self).await;
        let transfer = PortId::transfer();
        if self.port_id == transfer {
            Ics20Transfer::chan_close_init_execute(state, self).await;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
        }

        Ok(())
    }
}
