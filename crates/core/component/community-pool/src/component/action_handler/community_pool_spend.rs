use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;

use crate::{component::StateWriteExt as _, CommunityPoolSpend};

#[async_trait]
impl ActionHandler for CommunityPoolSpend {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // We can't statelessly check that the Community Pool has enough funds to spend, because we don't know
        // what its state is here.
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // This will fail if we try to overdraw the Community Pool, so we can never spend more than we have.
        state.community_pool_withdraw(self.value).await
    }
}
