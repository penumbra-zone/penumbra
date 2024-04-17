use crate::auction::dutch::ActionDutchAuctionWithdraw;
use crate::component::AuctionStoreRead;
use anyhow::{bail, ensure, Context, Result};
use ark_ff::Zero;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use decaf377::Fr;
use penumbra_asset::{Balance, Value};

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

    async fn check_and_execute<S: StateWrite>(&self, state: S) -> Result<()> {
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

        // Check that the reported balance commitment, match the recorded reserves.
        let auction_input_reserves = Value {
            amount: auction_state.state.input_reserves,
            asset_id: auction_state.description.input.asset_id,
        };

        let auction_output_reserves = Value {
            amount: auction_state.state.output_reserves,
            asset_id: auction_state.description.output_id,
        };

        let reserves_balance =
            Balance::from(auction_input_reserves) + Balance::from(auction_output_reserves);

        let expected_reserve_commitment = reserves_balance.commit(Fr::zero());

        ensure!(
            self.reserves_commitment == expected_reserve_commitment,
            "the reported reserve commitment is incorrect"
        );

        // When we execute, it will be critical write back an auction state with zeroed out
        // reserves.

        Ok(())
    }
}
