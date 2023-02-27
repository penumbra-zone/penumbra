use std::sync::Arc;

use anyhow::{ensure, Result};
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
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
    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
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

        // Check whether the end epoch is correct for the given validator (it
        // may already be unbonding).  This will only differ from the generic
        // unbonding time if the validator is already unbonding, so we only
        // require that the declared unbonding period is at least as long as the
        // expected one, to allow clients the option of skipping a query to
        // discover if the validator is already unbonding.
        let expected_end_epoch = state
            .current_unbonding_end_epoch_for(&u.validator_identity)
            .await?;
        if u.end_epoch_index < expected_end_epoch {
            return Err(anyhow::anyhow!(
                "undelegation end epoch must be at least {} but {} was specified",
                expected_end_epoch,
                u.end_epoch_index,
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
    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
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
