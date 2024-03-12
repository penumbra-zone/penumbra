use anyhow::{ensure, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_proof_params::CONVERT_PROOF_VERIFICATION_KEY;
use penumbra_sct::component::clock::EpochRead;

use crate::component::validator_handler::ValidatorDataRead;
use crate::component::SlashingData;
use crate::undelegate_claim::UndelegateClaimProofPublic;
use crate::UndelegateClaim;
use crate::{component::action_handler::ActionHandler, UnbondingToken};

#[async_trait]
impl ActionHandler for UndelegateClaim {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        let unbonding_id = UnbondingToken::new(
            self.body.validator_identity,
            self.body.unbonding_start_height,
        )
        .id();

        self.proof.verify(
            &CONVERT_PROOF_VERIFICATION_KEY,
            UndelegateClaimProofPublic {
                balance_commitment: self.body.balance_commitment,
                unbonding_id,
                penalty: self.body.penalty,
            },
        )?;

        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, state: S) -> Result<()> {
        // These checks all formerly happened in the `check_historical` method,
        // if profiling shows that they cause a bottleneck we could (CAREFULLY)
        // move some of them back.

        // If the validator delegation pool is bonded, or unbonding, we must
        // check that the unbonding delay (measured in blocks) has elapsed.
        let current_height = state.get_block_height().await?;

        // Check if the unbonding height has been reached, it will always be the case if
        // the validator delegation pool is unbonded.
        let allowed_unbonding_height = state
            .compute_unbonding_height(
                &self.body.validator_identity,
                self.body.unbonding_start_height,
            )
            .await?;

        let wait_blocks = allowed_unbonding_height.saturating_sub(current_height);

        ensure!(
            current_height >= allowed_unbonding_height,
            "cannot claim unbonding tokens before height {} (currently at {}, wait {} blocks)",
            allowed_unbonding_height,
            current_height,
            wait_blocks
        );

        let unbonding_epoch_start = state
            .get_epoch_by_height(self.body.unbonding_start_height)
            .await?;
        let unbonding_epoch_end = state.get_epoch_by_height(allowed_unbonding_height).await?;

        // Compute the penalty for the epoch range [unbonding_epoch_start, unbonding_epoch_end], and check
        // that it matches the penalty in the claim.
        let expected_penalty = state
            .compounded_penalty_over_range(
                &self.body.validator_identity,
                unbonding_epoch_start.index,
                unbonding_epoch_end.index,
            )
            .await?;

        ensure!(
            self.body.penalty == expected_penalty,
            "penalty does not match expected penalty"
        );

        /* ---------- execution ----------- */
        // No state changes here - this action just converts one token to another

        Ok(())
    }
}
