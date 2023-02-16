use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::DelegatorVote, Transaction};

use crate::ActionHandler;

#[async_trait]
impl ActionHandler for DelegatorVote {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        Ok(())
    }

    async fn check_stateful<S: StateRead>(&self, _state: Arc<S>) -> Result<()> {
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, _state: S) -> Result<()> {
        Ok(())
    }
}
