use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::{action::Delegate, Transaction};
use tracing::instrument;

use crate::{
    action_handler::ActionHandler,
    stake::{component::StateWriteExt as _, validator, StateReadExt as _},
};

#[async_trait]
impl ActionHandler for Delegate {
    #[instrument(name = "delegate", skip(self, context))]
    fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        // There are no stateless checks specific to this action.
        Ok(())
    }

    #[instrument(name = "delegate", skip(self, state))]
    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        let d = self;
        let next_rate_data = state
            .next_validator_rate(&d.validator_identity)
            .await?
            .ok_or_else(|| anyhow::anyhow!("unknown validator identity {}", d.validator_identity))?
            .clone();

        // Check whether the epoch is correct first, to give a more helpful
        // error message if it's wrong.
        if d.epoch_index != next_rate_data.epoch_index {
            return Err(anyhow::anyhow!(
                "delegation was prepared for epoch {} but the next epoch is {}",
                d.epoch_index,
                next_rate_data.epoch_index
            ));
        }

        // Check whether the delegation is allowed
        let validator = state
            .validator(&d.validator_identity)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing definition for validator"))?;
        let validator_state = state
            .validator_state(&d.validator_identity)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing state for validator"))?;

        use validator::State::*;
        if !validator.enabled {
            return Err(anyhow::anyhow!(
                "delegations are only allowed to enabled validators, but {} is disabled",
                d.validator_identity,
            ));
        }
        if !matches!(validator_state, Inactive | Active) {
            return Err(anyhow::anyhow!(
                    "delegations are only allowed to active or inactive validators, but {} is in state {:?}",
                    d.validator_identity,
                    validator_state,
                ));
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
        let expected_delegation_amount = next_rate_data.delegation_amount(d.unbonded_amount.into());

        if expected_delegation_amount != u64::from(d.delegation_amount) {
            return Err(anyhow::anyhow!(
                    "given {} unbonded stake, expected {} delegation tokens but description produces {}",
                    d.unbonded_amount,
                    expected_delegation_amount,
                    d.delegation_amount
                ));
        }

        Ok(())
    }

    #[instrument(name = "delegate", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        tracing::debug!(?self, "queuing delegation for next epoch");
        state.stub_push_delegation(self.clone());

        Ok(())
    }
}
