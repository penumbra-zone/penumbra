use crate::auction::dutch::ActionDutchAuctionWithdraw;
use crate::component::AuctionStoreRead;
use crate::component::DutchAuctionManager;
use anyhow::{bail, ensure, Context, Result};
use ark_ff::Zero;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use decaf377::Fr;

#[async_trait]
impl ActionHandler for ActionDutchAuctionWithdraw {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        ensure!(
            self.seq >= 1,
            "the sequence number MUST be greater or equal to 1 (got: {})",
            self.seq
        );

        ensure!(
            self.seq < u64::MAX,
            "the sequence number maximum is `u64::MAX`"
        );

        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let auction_id = self.auction_id;

        // Check that the auction exists and is a Dutch auction.
        let auction_state = state
            .get_dutch_auction_by_id(auction_id)
            .await
            .context("the auction associated with this id is not a dutch auction")?;

        let Some(auction_state) = auction_state else {
            bail!("no auction found for id {auction_id}")
        };

        // Check that sequence number is incremented by one.
        ensure!(
            self.seq == auction_state.state.sequence.saturating_add(1),
            "the action sequence number MUST be incremented by one (previous: {}, action: {})",
            self.seq,
            auction_state.state.sequence
        );

        // Execute the withdrawal, zero-ing out the auction state
        // and increasing its sequence number.
        let withdrawn_balance = state.withdraw_auction(auction_state).await?;

        // Check that the reported balance commitment, match the recorded reserves.
        let expected_reserve_commitment = withdrawn_balance.commit(Fr::zero());

        ensure!(
            self.reserves_commitment == expected_reserve_commitment,
            "the reported reserve commitment is incorrect"
        );

        Ok(())
    }
}
