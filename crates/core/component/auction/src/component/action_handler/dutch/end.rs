use anyhow::{ensure, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;

use crate::auction::dutch::ActionDutchAuctionEnd;
use crate::component::AuctionStoreRead;
use crate::component::DutchAuctionManager;

use anyhow::{bail, Context};

#[async_trait]
impl ActionHandler for ActionDutchAuctionEnd {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let auction_id = self.auction_id;

        let auction_state = state
            .get_dutch_auction_by_id(auction_id)
            .await
            .context("the auction associated with this id is not a dutch auction")?;

        let Some(auction) = auction_state else {
            bail!("no auction found for id {auction_id}")
        };

        // Check that the sequence number for the auction state is 0 (opened) or 1 (closed).
        ensure!(
            auction.state.sequence <= 1,
            "auction MUST have a sequence number set to opened (0) or closed (1) (got: {})",
            auction.state.sequence
        );

        state.close_auction(auction).await?;
        Ok(())
    }
}
