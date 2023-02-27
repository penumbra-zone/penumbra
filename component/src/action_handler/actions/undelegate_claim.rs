use std::sync::Arc;

use anyhow::{ensure, Result};
use async_trait::async_trait;
use penumbra_chain::StateReadExt;
use penumbra_crypto::stake::UnbondingToken;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::UndelegateClaim, Transaction};
use tracing::instrument;

use crate::{action_handler::ActionHandler, stake::StateReadExt as _};

#[async_trait]
impl ActionHandler for UndelegateClaim {
    #[instrument(name = "undelegate_claim", skip(self, _context))]
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        let unbonding_id = UnbondingToken::new(
            self.body.validator_identity,
            self.body.start_epoch_index,
            self.body.end_epoch_index,
        )
        .id();

        self.proof.verify(
            self.body.balance_commitment,
            unbonding_id,
            self.body.penalty,
        )?;

        Ok(())
    }

    #[instrument(name = "undelegate_claim", skip(self, state))]
    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        // We need to check two things:

        // 1. That we're past the specified unbonding end epoch.
        ensure!(
            state.get_current_epoch().await?.index >= self.body.end_epoch_index,
            "cannot claim unbonding tokens before the end epoch"
        );

        // 2. That the penalty is correct.
        let expected_penalty = state
            .compounded_penalty_over_range(
                &self.body.validator_identity,
                self.body.start_epoch_index,
                self.body.end_epoch_index,
            )
            .await?;
        ensure!(
            self.body.penalty == expected_penalty,
            "penalty does not match expected penalty"
        );
        Ok(())
    }

    #[instrument(name = "undelegate_claim", skip(self, _state))]
    async fn execute<S: StateWrite>(&self, _state: S) -> Result<()> {
        // TODO: where should we be tracking token supply changes?
        Ok(())
    }
}
