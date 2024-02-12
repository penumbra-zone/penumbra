use std::sync::Arc;

use anyhow::{ensure, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_proof_params::CONVERT_PROOF_VERIFICATION_KEY;
use penumbra_proto::StateWriteProto as _;
use penumbra_sct::component::clock::EpochRead;

use crate::component::validator_handler::ValidatorDataRead;
use crate::component::SlashingData;
use crate::undelegate_claim::UndelegateClaimProofPublic;
use crate::{component::action_handler::ActionHandler, UnbondingToken};
use crate::{event, UndelegateClaim};

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

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        // If the validator delegation pool is bonded, or unbonding, check that enough epochs
        // have elapsed to claim the unbonding tokens:
        let current_epoch = state.get_current_epoch().await?;
        let unbonding_epoch = state
            .compute_unbonding_epoch_for_validator(&self.body.validator_identity)
            .await?;

        ensure!(
            unbonding_epoch > current_epoch.index,
            "cannot claim unbonding tokens before the end epoch (current epoch: {}, end epoch: {})",
            current_epoch.index,
            unbonding_epoch
        );

        // Compute the penalty for the epoch range [start_epoch_index, unbonding_epoch], and check
        // that it matches the penalty in the claim.
        let expected_penalty = state
            .compounded_penalty_over_range(
                &self.body.validator_identity,
                self.body.start_epoch_index,
                unbonding_epoch,
            )
            .await?;

        ensure!(
            self.body.penalty == expected_penalty,
            "penalty does not match expected penalty"
        );
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, state: S) -> Result<()> {
        // TODO: where should we be tracking token supply changes?
        state.record_proto(event::undelegate_claim(self));
        Ok(())
    }
}
