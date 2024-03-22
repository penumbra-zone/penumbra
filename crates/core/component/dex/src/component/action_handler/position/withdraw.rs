use anyhow::{anyhow, Result};
use ark_ff::Zero;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use decaf377::Fr;
use penumbra_proto::StateWriteProto;

use crate::{
    component::{PositionManager, PositionRead, ValueCircuitBreaker},
    event,
    lp::{action::PositionWithdraw, position, Reserves},
};

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

        let mut metadata = state
            .position_by_id(&self.position_id)
            .await?
            .ok_or_else(|| anyhow!("withdrew from unknown position {}", self.position_id))?;

        // First, check that the commitment to the amount the user is
        // withdrawing is correct.
        //
        // Unlike other actions, where a balance commitment is used for
        // shielding a value, this commitment is used for compression, giving a
        // single commitment rather than a list of token amounts.

        // Note: since this is forming a commitment only to the reserves,
        // we are implicitly setting the reward amount to 0. However, we can
        // add support for rewards in the future without client changes.
        let expected_reserves_commitment = metadata
            .reserves
            .balance(&metadata.phi.pair)
            .commit(Fr::zero());

        if self.reserves_commitment != expected_reserves_commitment {
            anyhow::bail!(
                "reserves commitment {:?} is incorrect, expected {:?}",
                self.reserves_commitment,
                expected_reserves_commitment
            );
        }

        // Next, check that the withdrawal is consistent with the position state.
        // This should be redundant with the value balance mechanism (clients should
        // only be able to get the required input LPNFTs if the state transitions are
        // consistent), but we check it here for defense in depth.
        if self.sequence == 0 {
            if metadata.state != position::State::Closed {
                anyhow::bail!(
                    "attempted to withdraw position {} with state {}, expected Closed",
                    self.position_id,
                    metadata.state
                );
            }
        } else {
            if let position::State::Withdrawn { sequence } = metadata.state {
                if sequence + 1 != self.sequence {
                    anyhow::bail!(
                        "attempted to withdraw position {} with sequence {}, expected {}",
                        self.position_id,
                        self.sequence,
                        sequence + 1
                    );
                }
            } else {
                anyhow::bail!(
                    "attempted to withdraw position {} with state {}, expected Withdrawn",
                    self.position_id,
                    metadata.state
                );
            }
        }

        // Record an event prior to updating the position state, so we have access to
        // the current reserves.
        state.record_proto(event::position_withdraw(self, &metadata));

        // Debit the DEX for the outflows from this position.
        // TODO: in a future PR, split current PositionManager to PositionManagerInner
        // and fold this into a position open method
        state.vcb_debit(metadata.reserves_1()).await?;
        state.vcb_debit(metadata.reserves_2()).await?;

        // Finally, update the position. This has two steps:
        // - update the state with the correct sequence number;
        // - zero out the reserves, to prevent double-withdrawals.
        metadata.state = position::State::Withdrawn {
            // We just checked that the supplied sequence number is incremented by 1 from prev.
            sequence: self.sequence,
        };
        metadata.reserves = Reserves::zero();

        state.put_position(metadata).await?;

        Ok(())
    }
}
