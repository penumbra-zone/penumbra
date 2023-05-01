use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_crypto::dex::lp::position;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::action::PositionClose;

use crate::action_handler::ActionHandler;
use crate::dex::{PositionManager, PositionRead};

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
        let mut metadata = state
            .position_by_id(&self.position_id)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("could not find position with id {}", self.position_id)
            })?;

        if metadata.state != position::State::Opened {
            return Err(anyhow::anyhow!(
                "attempted to close position {} with state {}, expected Opened",
                self.position_id,
                metadata.state
            ));
        }

        metadata.state = position::State::Closed;
        state.put_position(metadata);

        Ok(())
    }
}
