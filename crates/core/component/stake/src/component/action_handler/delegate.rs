use anyhow::{ensure, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{DomainType, StateWriteProto};
use penumbra_sdk_sct::component::clock::EpochRead;

use crate::{
    component::validator_handler::ValidatorDataRead, event, validator::State::*, Delegate,
    StateReadExt as _, StateWriteExt as _,
};

#[async_trait]
impl ActionHandler for Delegate {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // There are no stateless checks specific to this action.
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // These checks all formerly happened in the `check_historical` method,
        // if profiling shows that they cause a bottleneck we could (CAREFULLY)
        // move some of them back.

        let d = self;

        // We check if the rate data is for the current epoch to provide a helpful
        // error message if there is a mismatch.
        let current_epoch = state.get_current_epoch().await?;
        ensure!(
            d.epoch_index == current_epoch.index,
            "delegation was prepared for epoch {} but the current epoch is {}",
            d.epoch_index,
            current_epoch.index
        );

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
        let validator_rate = state
            .get_validator_rate(&d.validator_identity)
            .await?
            .ok_or_else(|| anyhow::anyhow!("unknown validator identity {}", d.validator_identity))?
            .clone();

        let expected_delegation_amount = validator_rate.delegation_amount(d.unbonded_amount);

        ensure!(
            expected_delegation_amount == d.delegation_amount,
            "given {} unbonded stake, expected {} delegation tokens but description produces {}",
            d.unbonded_amount,
            expected_delegation_amount,
            d.delegation_amount,
        );

        // The delegation is only allowed if both conditions are met:
        // - the validator definition is `enabled` by the operator
        // - the validator is not jailed or tombstoned
        let validator = state
            .get_validator_definition(&d.validator_identity)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing definition for validator"))?;
        let validator_state = state
            .get_validator_state(&d.validator_identity)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing state for validator"))?;

        ensure!(
            validator.enabled,
            "delegations are only allowed to enabled validators, but {} is disabled",
            d.validator_identity,
        );

        ensure!(
            matches!(validator_state, Defined | Inactive | Active),
            "delegations are only allowed to active or inactive validators, but {} is in state {:?}",
            d.validator_identity,
            validator_state,
        );

        // (end of former check_historical checks)

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
                    "first delegation to a `Defined` validator must be at least {min_stake}"
                );
                tracing::debug!(%validator, %unbonded_delegation, "first delegation to validator recorded");
            }
        }

        // We queue the delegation so it can be processed at the epoch boundary.
        tracing::debug!(?self, "queuing delegation for next epoch");
        state.push_delegation(self.clone());
        state.record_proto(event::EventDelegate::from(self).to_proto());
        Ok(())
    }
}
