use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_sdk_compliance::RegulatedAssetCheck;

use crate::{
    component::{PositionManager, PositionRead},
    lp::action::PositionClose,
};

#[async_trait]
/// Debits an opened position NFT and credits a closed position NFT.
impl ActionHandler for PositionClose {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Nothing to do: the only validation is of the state change,
        // and that's done by the value balance mechanism.
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // Lookup position to check if assets are regulated
        let position = state
            .position_by_id(&self.position_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("position not found"))?;
        state
            .ensure_assets_not_regulated(
                &[position.phi.pair.asset_1(), position.phi.pair.asset_2()],
                "PositionClose",
            )
            .await?;

        // We don't want to actually close the position here, because otherwise
        // the economic effects could depend on intra-block ordering, and we'd
        // lose the ability to do block-scoped JIT liquidity, where a single
        // transaction opens and closes a position, keeping liquidity live only
        // during that block's batch swap execution.
        state.queue_close_position(self.position_id).await?;

        Ok(())
    }
}
