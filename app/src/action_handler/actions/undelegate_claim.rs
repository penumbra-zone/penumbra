use std::sync::Arc;

use anyhow::{ensure, Result};
use async_trait::async_trait;
use penumbra_chain::StateReadExt;
use penumbra_crypto::stake::UnbondingToken;
use penumbra_proof_params::UNDELEGATECLAIM_PROOF_VERIFICATION_KEY;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::action::UndelegateClaim;

use crate::{action_handler::ActionHandler, stake::StateReadExt as _};

#[async_trait]
impl ActionHandler for UndelegateClaim {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        let unbonding_id =
            UnbondingToken::new(self.body.validator_identity, self.body.start_epoch_index).id();

        self.proof.verify(
            &UNDELEGATECLAIM_PROOF_VERIFICATION_KEY,
            self.body.balance_commitment,
            unbonding_id,
            self.body.penalty,
        )?;

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        // We need to check two things:

        // 1. That we're past the specified unbonding end epoch.

        let current_epoch = state.epoch().await?;
        let end_epoch_index = state
            .unbonding_end_epoch_for(&self.body.validator_identity, self.body.start_epoch_index)
            .await?;
        ensure!(
            current_epoch.index >= end_epoch_index,
            "cannot claim unbonding tokens before the end epoch"
        );

        // 2. That the penalty is correct.
        let expected_penalty = state
            .compounded_penalty_over_range(
                &self.body.validator_identity,
                self.body.start_epoch_index,
                end_epoch_index,
            )
            .await?;
        ensure!(
            self.body.penalty == expected_penalty,
            "penalty does not match expected penalty"
        );
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, _state: S) -> Result<()> {
        // TODO: where should we be tracking token supply changes?
        Ok(())
    }
}
