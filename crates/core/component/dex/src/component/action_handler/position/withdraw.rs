use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use penumbra_component::ActionHandler;
use penumbra_crypto::{Fr, Zero};
use penumbra_storage::{StateRead, StateWrite};

use crate::{
    component::{PositionManager, PositionRead},
    lp::{action::PositionWithdraw, position},
};

#[async_trait]
/// Debits a closed position NFT and credits a withdrawn position NFT and the final reserves.
impl ActionHandler for PositionWithdraw {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Nothing to do: the only validation is of the state change,
        // and that's done by the value balance mechanism.
        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        // Check that the committed reserves in the action match the state.
        let position = state
            .position_by_id(&self.position_id)
            .await?
            .ok_or_else(|| anyhow!("withdrew from unknown position {}", self.position_id))?;

        let expected_reserves_commitment = position
            .reserves
            .balance(&position.phi.pair)
            .commit(Fr::zero());

        if self.reserves_commitment != expected_reserves_commitment {
            return Err(anyhow!(
                "reserves commitment {:?} is incorrect, expected {:?}",
                self.reserves_commitment,
                expected_reserves_commitment
            ));
        }

        // We don't check that the position state is Closed here, because all
        // stateful checks run in parallel, and we don't want to prevent someone
        // from closing and withdrawing a position in one transaction (though
        // they'll only be able to do so if the reserves don't shift before
        // submission).

        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // See comment in check_stateful for why we check the position state here.
        let mut metadata = state
            .position_by_id(&self.position_id)
            .await?
            .ok_or_else(|| anyhow!("withdrew from unknown position {}", self.position_id))?;

        if metadata.state != position::State::Closed {
            return Err(anyhow::anyhow!(
                "attempted to withdraw position {} with state {}, expected Closed",
                self.position_id,
                metadata.state
            ));
        }

        metadata.state = position::State::Withdrawn;
        state.put_position(metadata).await?;

        Ok(())
    }
}
