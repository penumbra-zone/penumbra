use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::DelegatorVote, Transaction};

use crate::{
    governance::{check, execute},
    ActionHandler,
};

#[async_trait]
impl ActionHandler for DelegatorVote {
    async fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        check::stateless::delegator_vote(self, context)
    }

    async fn check_stateful<S: StateRead>(&self, state: Arc<S>) -> Result<()> {
        check::stateful::delegator_vote(&state, self).await
    }

    async fn execute<S: StateWrite>(&self, state: S) -> Result<()> {
        execute::delegator_vote(state, self).await
    }
}
