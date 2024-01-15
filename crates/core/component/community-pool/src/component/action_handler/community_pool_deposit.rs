use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::ActionHandler;

use crate::{component::StateWriteExt as _, CommunityPoolDeposit};

#[async_trait]
impl ActionHandler for CommunityPoolDeposit {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Any deposit into the Community Pool is valid.
        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        // Any deposit into the Community Pool is valid.
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        state.community_pool_deposit(self.value).await
    }
}
