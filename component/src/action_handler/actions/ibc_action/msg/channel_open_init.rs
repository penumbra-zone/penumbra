use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use ibc::core::ics24_host::identifier::PortId;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::Transaction;

use crate::action_handler::ActionHandler;
use crate::ibc::component::channel::execution::channel_open_init::ChannelOpenInitExecute;
use crate::ibc::component::channel::stateful::channel_open_init::ChannelOpenInitCheck;
use crate::ibc::component::channel::stateless::channel_open_init::connection_hops_eq_1;
use crate::ibc::ibc_handler::{AppHandlerCheck, AppHandlerExecute};
use crate::ibc::transfer::Ics20Transfer;

#[async_trait]
impl ActionHandler for MsgChannelOpenInit {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        connection_hops_eq_1(self)?;

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        // No-op: IBC actions merge check_stateful and execute.
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        state.validate(self).await?;
        let transfer = PortId::transfer();
        if self.port_id_on_a == transfer {
            Ics20Transfer::chan_open_init_check(&mut state, self).await?;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
        }

        state.execute(self).await;
        let transfer = PortId::transfer();
        if self.port_id_on_a == transfer {
            Ics20Transfer::chan_open_init_execute(state, self).await;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
        }

        Ok(())
    }
}
