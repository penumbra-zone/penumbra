use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;

use crate::{component::StateWriteExt as _, CommunityPoolDeposit};

#[async_trait]
impl ActionHandler for CommunityPoolDeposit {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Any deposit into the Community Pool is valid.
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        Ok(state.community_pool_deposit(self.value).await)
    }
}
