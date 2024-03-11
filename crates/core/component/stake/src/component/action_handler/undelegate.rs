use anyhow::{ensure, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sct::component::clock::EpochRead;
use penumbra_shielded_pool::component::SupplyWrite;

use crate::{
    component::action_handler::ActionHandler,
    component::{validator_handler::ValidatorDataRead, StateWriteExt as _},
    event, Undelegate,
};

#[async_trait]
impl ActionHandler for Undelegate {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // These checks all formerly happened in the `check_stateful` method,
        // if profiling shows that they cause a bottleneck we could (CAREFULLY)
        // move some of them back.
        let u = self;

        // Check that the undelegation was prepared for the current epoch.
        // This allow us to provide a more helpful error message if an epoch
        // boundary was crossed. And more importantly, it will ensure that the
        // unbonding delay is enforced correctly.
        let current_epoch = state.get_current_epoch().await?;
        ensure!(
            u.from_epoch == current_epoch,
            "undelegation was prepared for epoch {} but the current epoch is {}",
            u.from_epoch.index,
            current_epoch.index
        );

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
        let rate_data = state
            .get_validator_rate(&u.validator_identity)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("unknown validator identity {}", u.validator_identity)
            })?;
        let expected_unbonded_amount = rate_data.unbonded_amount(u.delegation_amount);

        ensure!(
            u.unbonded_amount == expected_unbonded_amount,
            "undelegation amount {} does not match expected amount {}",
            u.unbonded_amount,
            expected_unbonded_amount,
        );

        /* ----- execution ------ */

        // Register the undelegation's denom, so clients can look it up later.
        state
            .register_denom(&self.unbonding_token().denom())
            .await?;

        tracing::debug!(?self, "queuing undelegation for next epoch");
        state.push_undelegation(self.clone());

        state.record(event::undelegate(self));

        Ok(())
    }
}
