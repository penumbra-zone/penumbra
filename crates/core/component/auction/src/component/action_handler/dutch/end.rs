use anyhow::{ensure, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_sdk_proto::StateWriteProto;
use tracing::instrument;

use crate::auction::dutch::ActionDutchAuctionEnd;
use crate::component::AuctionStoreRead;
use crate::component::DutchAuctionManager;
use crate::event;

use anyhow::{bail, Context};

#[async_trait]
impl ActionHandler for ActionDutchAuctionEnd {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        Ok(())
    }

    #[instrument(name = "dutch_auction_end", skip(self, state))]
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
            matches!(auction.state.sequence, 0 | 1),
            "auction MUST have a sequence number set to opened (0) or closed (1) (got: {})",
            auction.state.sequence
        );

        // Keep a copy of the auction state for the event.
        let auction_state = auction.state.clone();

        // Terminate the auction
        state.end_auction(auction).await?;
        // Emit an event, tracing the reason for the auction ending.
        state.record_proto(event::dutch_auction_closed_by_user(
            auction_id,
            auction_state,
        ));

        Ok(())
    }
}
