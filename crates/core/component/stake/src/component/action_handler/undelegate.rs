use anyhow::{bail, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sct::component::clock::EpochRead;
use penumbra_shielded_pool::component::AssetRegistry;
use tracing::error;

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
        /* ----- checks ------ */
        // NB: These checks all formerly happened in the `check_historical` method, if profiling
        // shows that they cause a bottleneck we could (CAREFULLY) move some of them back.

        // # Invariant: the undelegation must have been prepared for the current epoch.
        self.check_that_undelegate_was_prepared_for_current_epoch(&state)
            .await?;

        // # Invariant: The unbonded amount must be correctly computed.
        self.check_that_unbonded_amount_is_correctly_computed(&state)
            .await?;

        /* ----- execution ------ */

        // Register the undelegation's denom, so clients can look it up later.
        state.register_denom(&self.unbonding_token().denom()).await;

        tracing::debug!(?self, "queuing undelegation for next epoch");
        state.push_undelegation(self.clone());

        state.record(event::undelegate(self));

        Ok(())
    }
}

/// Interfaces for enforcing [`ActionHandler`] invariants.
impl Undelegate {
    /// Returns `Ok(())` if this delegation is from the current epoch.
    ///
    /// This ensures that the unbonding delay is enforced correctly. Additionally, it helps us
    /// provide a more helpful error message if an epoch boundary was crossed.
    async fn check_that_undelegate_was_prepared_for_current_epoch(
        &self,
        state: &impl StateWrite,
    ) -> Result<()> {
        let u = self;
        let current_epoch = state.get_current_epoch().await?;

        // Check that this delegation is from the current epoch.
        if u.from_epoch != current_epoch {
            error!(
                ?u.from_epoch,
                ?current_epoch,
                "undelegation was prepared for a different epoch"
            );
            bail!(
                "undelegation was prepared for epoch {} but the current epoch is {}",
                u.from_epoch.index,
                current_epoch.index
            );
        }

        Ok(())
    }

    /// Returns `Ok(())` if this delegation has a correctly computed unbonding amount.
    ///
    /// For undelegations, we enforce correct computation (with rounding) of the *unbonded
    /// amount based on the delegation amount*, because users (should be) starting with the
    /// amount of delegation tokens they wish to undelegate, and computing the amount of
    /// unbonded stake they receive.
    ///
    /// The direction of the computation matters because the computation involves rounding, so
    /// while both...
    ///
    /// ```
    /// (unbonded amount, rates) -> delegation amount
    /// (delegation amount, rates) -> unbonded amount
    /// ```
    ///
    /// ...should give approximately the same results, they may not give *exactly* the same
    /// results.
    async fn check_that_unbonded_amount_is_correctly_computed(
        &self,
        state: &impl StateWrite,
    ) -> Result<()> {
        let u = self;

        // Compute the expected unbonded amount for the validator.
        let rate_data = state
            .get_validator_rate(&u.validator_identity)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("unknown validator identity {}", u.validator_identity)
            })?;
        let expected_unbonded_amount = rate_data.unbonded_amount(u.delegation_amount);

        // Check that this undelegation matches the expected amount.
        if u.unbonded_amount != expected_unbonded_amount {
            tracing::error!(
                actual = %u.unbonded_amount,
                expected = %expected_unbonded_amount,
                "undelegation amount does not match expected amount",
            );
            anyhow::bail!(
                "undelegation amount {} does not match expected amount {}",
                u.unbonded_amount,
                expected_unbonded_amount,
            );
        }

        Ok(())
    }
}
