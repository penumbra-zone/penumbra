//! Action handler for ActionBurn.

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_sdk_proto::StateWriteProto as _;

use crate::{component::event, ActionBurn};

#[async_trait]
impl ActionHandler for ActionBurn {
    type CheckStatelessContext = ();

    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Stateless validation is done in the constructor:
        // - Amount is non-zero
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // Burning is handled entirely by the value balance system.
        // The action has a negative balance (consuming the burned value),
        // which is checked by the transaction's overall balance verification.

        // Emit event for indexers
        state.record_proto(event::burn(self));

        Ok(())
    }
}
