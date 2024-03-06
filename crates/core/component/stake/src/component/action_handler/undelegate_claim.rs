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
        let unbonding_id =
            UnbondingToken::new(self.body.validator_identity, self.body.start_epoch_index).id();

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

        // If the validator delegation pool is bonded, or unbonding, check that enough epochs
        // have elapsed to claim the unbonding tokens:
        let current_epoch = state.get_current_epoch().await?;
        let allowed_unbonding_epoch = state
            .compute_unbonding_epoch(&self.body.validator_identity, self.body.start_epoch_index)
            .await?;

        ensure!(
            current_epoch.index >= allowed_unbonding_epoch,
            "cannot claim unbonding tokens before the end epoch (current epoch: {}, unbonding epoch: {})",
            current_epoch.index,
            allowed_unbonding_epoch
        );

        // Compute the penalty for the epoch range [start_epoch_index, unbonding_epoch], and check
        // that it matches the penalty in the claim.
        let expected_penalty = state
            .compounded_penalty_over_range(
                &self.body.validator_identity,
                self.body.start_epoch_index,
                allowed_unbonding_epoch,
            )
            .await?;

        ensure!(
            self.body.penalty == expected_penalty,
            "penalty does not match expected penalty"
        );

        // (end of former check_historical impl)

        // No state changes here - this action just converts one token to another

        // TODO: where should we be tracking token supply changes?
        Ok(())
    }
}
