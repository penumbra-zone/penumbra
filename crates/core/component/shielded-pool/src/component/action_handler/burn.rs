use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_sdk_proto::{DomainType as _, StateWriteProto as _};

use crate::{event, ActionBurn};

#[async_trait]
impl ActionHandler for ActionBurn {
    type CheckStatelessContext = ();

    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // burns are always valid from a stateless perspective.
        // the balance check ensures the transaction has enough value to burn.
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // the actual burning is handled by the balance check in the transaction.
        // the burn action contributes a negative balance, which must be offset
        // by spends or other actions that provide positive balance.
        //
        // we just emit an event for observability.
        state.record_proto(
            event::EventBurn {
                value: self.value,
            }
            .to_proto(),
        );

        Ok(())
    }
}
