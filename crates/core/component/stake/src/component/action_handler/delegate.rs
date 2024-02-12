use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::ActionHandler;
use penumbra_proto::StateWriteProto as _;

use crate::{
    component::{
        validator_handler::{ValidatorDataRead, ValidatorManager},
        StateWriteExt as _,
    },
    event, validator, Delegate, StateReadExt as _,
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

        use validator::State::*;
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
        use crate::validator;

        tracing::debug!(?self, "queuing delegation for next epoch");
        state.push_delegation(self.clone());

        // When a validator definition is published, it starts in a `Defined` state
        // until it gathers enough stake to become `Inactive` and get indexed in the
        // validator list.
        //
        // Unlike other validator state transitions, this one is executed with the
        // delegation transaction and not at the end of the epoch. This is because we
        // want to avoid having to iterate over all defined validators at all.
        // See #2921 for more details.
        let validator_state = state
            .get_validator_state(&self.validator_identity)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing state for validator"))?;

        if matches!(validator_state, validator::State::Defined)
            && self.delegation_amount.value()
                >= state.get_stake_params().await?.min_validator_stake.value()
        {
            tracing::debug!(validator_identity = %self.validator_identity, delegation_amount = %self.delegation_amount, "validator has enough stake to transition out of defined state");
            state
                .set_validator_state(&self.validator_identity, validator::State::Inactive)
                .await?;
        }

        state.record_proto(event::delegate(self));
        Ok(())
    }
}
