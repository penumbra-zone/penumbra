use std::sync::Arc;

use anyhow::{ensure, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::ActionHandler;
use penumbra_num::Amount;

use crate::{
    component::{
        validator_handler::{ValidatorDataRead, ValidatorManager},
        StateWriteExt as _,
    },
    event,
    validator::State::*,
    Delegate, StateReadExt as _,
};

#[async_trait]
impl ActionHandler for Delegate {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // There are no stateless checks specific to this action.
        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        let d = self;
        let next_rate_data = state
            .get_validator_rate(&d.validator_identity)
            .await?
            .ok_or_else(|| anyhow::anyhow!("unknown validator identity {}", d.validator_identity))?
            .clone();

        // Check whether the epoch is correct first, to give a more helpful
        // error message if it's wrong.
        if d.epoch_index != next_rate_data.epoch_index {
            anyhow::bail!(
                "delegation was prepared for epoch {} but the next epoch is {}",
                d.epoch_index,
                next_rate_data.epoch_index
            );
        }

        // Check whether the delegation is allowed
        // The delegation is allowed if:
        // - the validator definition is "enabled" by the operator
        // - the validator is not jailed or tombstoned
        let validator = state
            .get_validator_definition(&d.validator_identity)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing definition for validator"))?;
        let validator_state = state
            .get_validator_state(&d.validator_identity)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing state for validator"))?;

        if !validator.enabled {
            anyhow::bail!(
                "delegations are only allowed to enabled validators, but {} is disabled",
                d.validator_identity,
            );
        }
        if !matches!(validator_state, Defined | Inactive | Active) {
            anyhow::bail!(
                "delegations are only allowed to active or inactive validators, but {} is in state {:?}",
                d.validator_identity,
                validator_state,
            );
        }

        // For delegations, we enforce correct computation (with rounding)
        // of the *delegation amount based on the unbonded amount*, because
        // users (should be) starting with the amount of unbonded stake they
        // wish to delegate, and computing the amount of delegation tokens
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
        let expected_delegation_amount = next_rate_data.delegation_amount(d.unbonded_amount);

        if expected_delegation_amount != d.delegation_amount {
            anyhow::bail!(
                "given {} unbonded stake, expected {} delegation tokens but description produces {}",
                d.unbonded_amount,
                expected_delegation_amount,
                d.delegation_amount
            );
        }

        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let validator = self.validator_identity;
        let unbonded_delegation = self.unbonded_amount;
        // This action is executed in two phases:
        // 1. We check if the self-delegation requirement is met.
        // 2. We queue the delegation for the next epoch.

        let validator_state = state
            .get_validator_state(&self.validator_identity)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing state for validator"))?;

        // When a validator definition is published, it starts in a `Defined` state
        // where it is unindexed by the staking module. We transition validator with
        // too little stake to the `Defined` state as well. See #2921 for more details.
        if validator_state == Defined {
            let min_stake = state.get_stake_params().await?.min_validator_stake;
            // With #3853, we impose a minimum self-delegation requirement to simplify
            // end-epoch handling. The first delegation" to a `Defined` validator must
            // be at least `min_validator_stake`.
            //
            // Note: Validators can be demoted to `Defined` if they have too little stake,
            // if we don't check that the pool is empty, we could trap delegations.
            let validator_pool_size = state
                .get_validator_pool_size(&validator)
                .await
                .unwrap_or_else(Amount::zero);

            if validator_pool_size == Amount::zero() {
                ensure!(
                unbonded_delegation >= min_stake,
                "first delegation to a `Defined` validator must be at least min_validator_stake"
            );
                tracing::debug!(%validator, %unbonded_delegation, "first delegation to validator recorded");
            }
        }

        // We queue the delegation so it can be processed at the epoch boundary.
        tracing::debug!(?self, "queuing delegation for next epoch");
        state.push_delegation(self.clone());
        state.record(event::delegate(self));
        Ok(())
    }
}
