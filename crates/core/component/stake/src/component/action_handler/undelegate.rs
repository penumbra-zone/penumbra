use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sdk_proto::{DomainType as _, StateWriteProto};
use penumbra_sdk_sct::component::clock::EpochRead;
use penumbra_sdk_shielded_pool::component::AssetRegistry;

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
        // These checks all formerly happened in the `check_historical` method,
        // if profiling shows that they cause a bottleneck we could (CAREFULLY)
        // move some of them back.
        let u = self;

        // Check that the undelegation was prepared for the current epoch.
        // This let us provide a more helpful error message if an epoch boundary was crossed.
        // And it ensures that the unbonding delay is enforced correctly.
        let current_epoch = state.get_current_epoch().await?;
        let prepared_for_current_epoch = u.from_epoch == current_epoch;
        if !prepared_for_current_epoch {
            tracing::error!(
                from = ?u.from_epoch,
                current = ?current_epoch,
                "undelegation was prepared for a different epoch",
            );
            anyhow::bail!(
                "undelegation was prepared for epoch {} but the current epoch is {}",
                u.from_epoch.index,
                current_epoch.index
            );
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
        let rate_data = state
            .get_validator_rate(&u.validator_identity)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("unknown validator identity {}", u.validator_identity)
            })?;
        let expected_unbonded_amount = rate_data.unbonded_amount(u.delegation_amount);

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

        /* ----- execution ------ */

        // Register the undelegation's denom, so clients can look it up later.
        state.register_denom(&self.unbonding_token().denom()).await;

        tracing::debug!(?self, "queuing undelegation for next epoch");
        state.push_undelegation(self.clone());

        state.record_proto(event::EventUndelegate::from(self).to_proto());

        Ok(())
    }
}
