use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_sdk_compliance::RegulatedAssetCheck;

use crate::{component::StateWriteExt as _, CommunityPoolDeposit};

#[async_trait]
impl ActionHandler for CommunityPoolDeposit {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Any deposit into the Community Pool is valid (stateless check).
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // Block regulated assets from being deposited into the community pool
        state
            .ensure_not_regulated(self.value.asset_id, "CommunityPoolDeposit")
            .await?;

        Ok(state.community_pool_deposit(self.value).await)
    }
}
