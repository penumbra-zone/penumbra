use anyhow::Result;
use ark_ff::Zero;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use decaf377::Fr;

use crate::{component::PositionManager, lp::action::PositionWithdraw};

#[async_trait]
/// Debits a closed position NFT and credits a withdrawn position NFT and the final reserves.
impl ActionHandler for PositionWithdraw {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Nothing to do: the only validation is of the state change,
        // and that's done by the value balance mechanism.
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // See comment in check_stateful for why we check the position state here:
        // we need to ensure that we're checking the reserves at the moment we execute
        // the withdrawal, to prevent any possibility of TOCTOU attacks.

        // Execute the withdrawal, extracting the reserves from the position.
        let actual_reserves = state
            .withdraw_position(self.position_id, self.sequence)
            .await?;

        // Next, and CRITICALLY, check that the commitment to the amount the user is
        // withdrawing is correct.
        //
        // Unlike other actions, where a balance commitment is used for
        // shielding a value, this commitment is used for compression, giving a
        // single commitment rather than a list of token amounts.
        //
        // Note: since this is forming a commitment only to the reserves,
        // we are implicitly setting the reward amount to 0. However, we can
        // add support for rewards in the future without client changes.
        let expected_reserves_commitment = actual_reserves.commit(Fr::zero());

        if self.reserves_commitment != expected_reserves_commitment {
            anyhow::bail!(
                "reserves commitment {:?} is incorrect, expected {:?}",
                self.reserves_commitment,
                expected_reserves_commitment
            );
        }

        Ok(())
    }
}
