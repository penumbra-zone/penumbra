use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_component::ActionHandler;
use penumbra_storage::{StateRead, StateWrite};

use crate::component::PositionManager;
use crate::lp::action::PositionClose;

#[async_trait]
/// Debits an opened position NFT and credits a closed position NFT.
impl ActionHandler for PositionClose {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Nothing to do: the only validation is of the state change,
        // and that's done by the value balance mechanism.
        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // We don't want to actually close the position here, because otherwise
        // the economic effects could depend on intra-block ordering, and we'd
        // lose the ability to do block-scoped JIT liquidity, where a single
        // transaction opens and closes a position, keeping liquidity live only
        // during that block's batch swap execution.
        state.queue_close_position(self.position_id);

        Ok(())
    }
}
