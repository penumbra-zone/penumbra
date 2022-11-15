use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::action::ValidatorVote;

use crate::action_handler::ActionHandler;

#[async_trait]
impl ActionHandler for ValidatorVote {
    fn check_tx_stateless(&self) -> anyhow::Result<()> {
        todo!()
    }

    async fn check_tx_stateful(&self, _state: Arc<State>) -> Result<()> {
        todo!()
    }

    async fn execute_tx(&self, _state: &mut StateTransaction) -> Result<()> {
        todo!()
    }
}
