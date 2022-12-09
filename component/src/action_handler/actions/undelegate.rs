use std::sync::Arc;

use anyhow::{ensure, Result};
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::{action::Undelegate, Transaction};
use tracing::instrument;

use crate::{
    action_handler::ActionHandler,
    shielded_pool::SupplyWrite,
    stake::{component::StateWriteExt as _, StateReadExt as _},
};

#[async_trait]
impl ActionHandler for Undelegate {
    #[instrument(name = "undelegate", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        Ok(())
    }

    #[instrument(name = "undelegate", skip(self, state))]
    async fn check_stateful(&self, state: Arc<State>) -> Result<()> {
        let u = self;
        let rate_data = state
            .next_validator_rate(&u.validator_identity)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("unknown validator identity {}", u.validator_identity)
            })?;

        // Check whether the start epoch is correct first, to give a more helpful
        // error message if it's wrong.
        if u.start_epoch_index != rate_data.epoch_index {
            return Err(anyhow::anyhow!(
                "undelegation was prepared for next epoch {} but the next epoch is {}",
                u.start_epoch_index,
                rate_data.epoch_index
            ));
        }
        // Check whether the end epoch is correct for the given validator (it may already be unbonding).
        let expected_end_epoch = state
            .current_unbonding_end_epoch_for(&u.validator_identity)
            .await?;
        if u.end_epoch_index != expected_end_epoch {
            return Err(anyhow::anyhow!(
                "undelegation was prepared for end epoch {} but the end epoch is {}",
                u.end_epoch_index,
                expected_end_epoch
            ));
        }

        // For undelegations, we enforce correct computation (with rounding)
        // of the *unbonded amount based on the delegation amount*, because
        // users (should be) starting with the amount of delegation tokens they
        // wish to undelegate, and computing the amount of unbonded stake
        // they receive.
        //
        // The direction of the computation matters because the computation
        // involves rounding, so while both
        //
        // (unbonded amount, rates) -> delegation amount
        // (delegation amount, rates) -> unbonded amount
        //
        // should give approximately the same results, they may not give
        // exactly the same results.
        let expected_unbonded_amount = rate_data.unbonded_amount(u.delegation_amount.into());

        ensure!(
            u64::from(u.unbonded_amount) == expected_unbonded_amount,
            "undelegation amount {} does not match expected amount {}",
            u.unbonded_amount,
            expected_unbonded_amount,
        );

        Ok(())
    }

    #[instrument(name = "undelegate", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        tracing::debug!(?self, "queuing undelegation for next epoch");
        state.stub_push_undelegation(self.clone());
        // Register the undelegation's denom, so we clients can look it up later.
        state
            .register_denom(&self.unbonding_token().denom())
            .await?;
        // TODO: should we be tracking changes to token supply here or in end_epoch?

        Ok(())
    }
}
