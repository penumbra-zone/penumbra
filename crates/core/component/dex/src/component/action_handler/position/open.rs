use std::sync::Arc;

use anyhow::{ensure, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::ActionHandler;

use crate::{
    component::{PositionManager, StateReadExt},
    lp::{action::PositionOpen, position},
};

#[async_trait]
/// Debits the initial reserves and credits an opened position NFT.
impl ActionHandler for PositionOpen {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Check:
        //  + reserves are at most 52 bits wide,
        //  + the trading function coefficients are at most 80 bits wide.
        //  + at least some assets are provisioned.
        //  + the trading function coefficients are non-zero,
        //  + the trading function doesn't specify a cyclic pair,
        //  + the fee is <=50%.
        self.position.check_stateless()?;
        if self.position.state != position::State::Opened {
            anyhow::bail!("attempted to open a position with a state besides `Opened`");
        }
        Ok(())
    }

    async fn check_historical<S: StateRead + 'static>(
        &self,
        historical_state: Arc<S>,
    ) -> Result<()> {
        // Safety: This is safe to do in a historical check because the chain state
        // it inspects cannot be modified by other actions in the same transaction.
        ensure!(
            historical_state.get_dex_params().await?.is_enabled,
            "Dex MUST be enabled to open LPs"
        );
        Ok(())
    }
    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        state.open_position(self.position.clone()).await?;
        Ok(())
    }
}
