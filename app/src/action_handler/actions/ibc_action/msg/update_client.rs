use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::msgs::update_client::MsgUpdateClient;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::Transaction;

use crate::action_handler::ActionHandler;
use crate::ibc::component::client::stateless::update_client::header_is_tendermint;
use crate::ibc::component::client::{
    stateful::update_client::UpdateClientCheck, Ics2ClientExt as _,
};

#[async_trait]
impl ActionHandler for MsgUpdateClient {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        header_is_tendermint(self)?;

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        // No-op: IBC actions merge check_stateful and execute.
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // Optimization: no-op if the update is already committed.  We no-op
        // to Ok(()) rather than erroring to avoid having two "racing" relay
        // transactions fail just because they both contain the same client
        // update.
        if !state.update_is_already_committed(&self).await? {
            tracing::debug!(msg = ?self);
            state.validate(self).await?;
            state.execute_update_client(self).await?;
        } else {
            tracing::debug!("skipping duplicate update");
        }

        Ok(())
    }
}
