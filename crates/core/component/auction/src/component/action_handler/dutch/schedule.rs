use crate::auction::dutch::DutchAuctionDescription;
use crate::component::AuctionStoreRead;
use anyhow::{ensure, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_sct::component::clock::EpochRead; // AuctionRead?

use crate::auction::dutch::ActionDutchAuctionSchedule;
use crate::component::DutchAuctionManager;

#[async_trait]
impl ActionHandler for ActionDutchAuctionSchedule {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        let DutchAuctionDescription {
            input,
            output_id,
            max_output,
            min_output,
            start_height,
            end_height,
            step_count,
            nonce: _,
        } = self.description;

        // Fail fast if the step count is zero
        ensure!(step_count > 0, "step count MUST be positive (got zero)");

        // Check that we disallow identical input/output ids.
        ensure!(
            input.asset_id != output_id,
            "input id MUST be different from output id"
        );

        // Check that the `max_output` is greater than the `min_output`
        ensure!(
            max_output > min_output,
            "max_output MUST be greater than min_output"
        );

        // Check that the max output is greater than zero.
        ensure!(max_output > 0u128.into(), "max output MUST be positive");

        // Check that the start and end height are valid.
        ensure!(
            start_height < end_height,
            "the start height MUST be strictly less than the end height (got: start={} >= end={})",
            start_height,
            end_height
        );

        // Check that the step count is positive.
        ensure!(
            step_count > 0,
            "step count MUST be greater than zero (got: {step_count})"
        );

        // Check that the step count is less than 1000.
        ensure!(
            step_count <= 1000,
            "the dutch auction step count MUST be less than 1000 (got: {step_count})",
        );

        // Check that height delta is a multiple of `step_count`.
        let block_window = end_height.checked_sub(start_height).ok_or_else(|| {
            anyhow::anyhow!(
                "underflow ({end_height} < {start_height}) - the validation rules are incoherent!"
            )
        })?;
        ensure!(
            (block_window % step_count) == 0,
            "the block window ({block_window}) MUST be a multiple of the step count ({step_count})"
        );

        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let schedule = self;

        // Check that `start_height` is in the future.
        let current_height = state.get_block_height().await?;
        let start_height = schedule.description.start_height;
        ensure!(
            start_height > current_height,
            "dutch auction MUST start in the future (start={}, current={})",
            start_height,
            current_height
        );

        // Check that the `auction_id` is unused.
        let id = schedule.description.id();
        ensure!(
            !state.auction_id_exists(id).await,
            "the supplied auction id is already known to the chain (id={id})"
        );

        state.schedule_auction(schedule.description.clone()).await;
        Ok(())
    }
}
