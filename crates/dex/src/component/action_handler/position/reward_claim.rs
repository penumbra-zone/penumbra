use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_component::ActionHandler;
use penumbra_storage::{StateRead, StateWrite};

use crate::lp::action::PositionRewardClaim;

#[async_trait]
/// Debits a withdrawn position NFT and credits a claimed position NFT and any liquidity incentives.
impl ActionHandler for PositionRewardClaim {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // It's important to reject all LP actions for now, to prevent
        // inflation / minting bugs until we implement all required checks
        // (e.g., minting tokens by withdrawing reserves we don't check)
        Err(anyhow::anyhow!("lp rewards not supported yet"))
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        // It's important to reject all LP actions for now, to prevent
        // inflation / minting bugs until we implement all required checks
        // (e.g., minting tokens by withdrawing reserves we don't check)
        Err(anyhow::anyhow!("lp rewards not supported yet"))
    }

    async fn execute<S: StateWrite>(&self, _state: S) -> Result<()> {
        // It's important to reject all LP actions for now, to prevent
        // inflation / minting bugs until we implement all required checks
        // (e.g., minting tokens by withdrawing reserves we don't check)
        Err(anyhow::anyhow!("lp rewards not supported yet"))
    }
}
