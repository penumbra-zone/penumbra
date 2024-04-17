use crate::auction::dutch::{DutchAuction, DutchAuctionDescription, DutchAuctionState};
use crate::auction::AuctionId;
use crate::component::AuctionStoreRead;
use crate::state_key;
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_num::Amount;
use penumbra_proto::core::component::auction::v1alpha1 as pb;
use penumbra_sct::component::clock::EpochRead;
use prost::{Message, Name};

#[async_trait]
pub(crate) trait DutchAuctionManager: StateWrite {
    /// Schedule an auction for the specified [`DutchAuctionDescritpion`], initializing
    /// its state, and registering it for execution by the component.
    async fn schedule_auction(&mut self, description: DutchAuctionDescription) {
        let auction_id = description.id();
        let DutchAuctionDescription {
            input: _,
            output_id: _,
            max_output: _,
            min_output: _,
            start_height,
            end_height,
            step_count,
            nonce: _,
        } = description;

        let next_trigger = Self::compute_next_trigger(TriggerData {
            start_height,
            end_height,
            step_count,
            current_height: self
                .get_block_height()
                .await
                .expect("block height is not missing"),
        })
        .expect("infaillible because of action validation")
        .expect("action validation guarantees the auction is not expired");

        let state = DutchAuctionState {
            sequence: 0,
            current_position: None,
            next_trigger,
            input_reserves: description.input.amount,
            output_reserves: Amount::zero(),
        };

        let dutch_auction = DutchAuction { description, state };

        // Set the triggger
        self.set_trigger_for_id(auction_id, next_trigger);
        // Write position to state
        self.write_dutch_auction_state(dutch_auction);
    }

    /// Terminate the Dutch auction associated with the specified [`AuctionId`].
    ///
    /// # Errors
    /// This method returns an error if the id is not found, or if the
    /// recorded entry is not of type `DutchAuction`.
    async fn close_auction_by_id(&mut self, id: AuctionId) -> Result<()> {
        let auction = self
            .get_dutch_auction_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("auction not found"))?;
        self.close_auction(auction)
    }

    /// Terminate and update the supplied auction state.
    fn close_auction(&mut self, auction_to_close: DutchAuction) -> Result<()> {
        let DutchAuctionState {
            sequence,
            current_position,
            next_trigger,
            input_reserves,
            output_reserves,
        } = auction_to_close.state;

        // Short-circuit to no-op if the auction is already closed.
        if sequence >= 1 {
            return Ok(());
        }

        let auction_id = auction_to_close.description.id();

        // We close and retire the DEX position owned by this auction state,
        // and return the respective amount of input and output we should credit
        // to the total tracked amount, so that it can be returned to its bearer.
        let (input_from_position, output_from_position) = if let Some(_position) = current_position
        {
            // Get position state
            // Withdraw position from the dex
            // Return reserves so that we can credit them to i/o rs.
            (Amount::zero(), Amount::zero())
        } else {
            (Amount::zero(), Amount::zero())
        };

        // If a `next_trigger` entry is set, we remove it.
        if next_trigger != 0 {
            self.unset_trigger_for_id(auction_id, next_trigger)
        }

        let total_input_reserves = input_reserves + input_from_position;
        let total_output_reserves = output_reserves + output_from_position;

        let closed_auction = DutchAuction {
            description: auction_to_close.description,
            state: DutchAuctionState {
                sequence: 1u64,
                current_position: None,
                next_trigger: 0,
                input_reserves: total_input_reserves,
                output_reserves: total_output_reserves,
            },
        };
        self.write_dutch_auction_state(closed_auction);
        Ok(())
    }

    /// Withdraw a dutch auction, zero-ing out its state, and increasing its sequence
    /// number.
    ///
    /// # Errors
    /// This method errors if the auction id is not found, or if the associated
    /// entry is not of type [`DutchAuction`].
    async fn withdraw_auction_by_id(&mut self, id: AuctionId) -> Result<()> {
        let auction = self
            .get_dutch_auction_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("auction not found"))?;
        self.withdraw_auction(auction);
        Ok(())
    }

    fn withdraw_auction(&mut self, mut auction: DutchAuction) {
        auction.state.sequence = auction.state.sequence.saturating_add(1);
        auction.state.current_position = None;
        auction.state.next_trigger = 0;
        auction.state.input_reserves = Amount::zero();
        auction.state.output_reserves = Amount::zero();
        self.write_dutch_auction_state(auction)
    }
}

impl<T: StateWrite + ?Sized> DutchAuctionManager for T {}

trait Inner: StateWrite {
    /// Serialize a `DutchAuction` as an `Any` into chain state.
    fn write_dutch_auction_state(&mut self, new_state: DutchAuction) {
        let id = new_state.description.id();
        let key = state_key::auction_store::by_id(id);
        let pb_state: pb::DutchAuction = new_state.into();
        let raw_auction = pb_state.encode_length_delimited_to_vec();

        let any_auction = prost_types::Any {
            type_url: pb::DutchAuction::full_name(),
            value: raw_auction,
        };

        let raw_any = any_auction.encode_length_delimited_to_vec();

        self.put_raw(key, raw_any);
    }

    /// Compute the next trigger height, return `None` if the step count
    /// has been reached and the auction should be retired.
    ///
    /// # Errors
    /// This method errors if the block interval is not a multiple of the
    /// specified `step_count`, or if it operates over an invalid block
    /// interval (which should NEVER happen unless validation is broken).
    ///
    // TODO(erwan): doing everything checked at least for now, will remove as
    // i fill the tests module.
    fn compute_next_trigger(trigger_data: TriggerData) -> Result<Option<u64>> {
        let block_interval = trigger_data
            .end_height
            .checked_sub(trigger_data.start_height)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "block interval calculation has underflowed (end={}, start={})",
                    trigger_data.end_height,
                    trigger_data.start_height
                )
            })?;

        // Compute the step size, based on the block interval and the number of
        // discrete steps the auction specifies.
        let step_size = block_interval
            .checked_div(trigger_data.step_count)
            .ok_or_else(|| anyhow::anyhow!("step count is zero"))?;

        // Compute the step index for the current height, this should work even if
        // the supplied height does not fall perfectly on a step boundary. First, we
        // "clamp it" to a previous step index, then we increment by 1 to compute the
        // next one, and finally we determine a concrete trigger height based off that.
        let prev_step_index = trigger_data
            .current_height
            .checked_div(step_size)
            .ok_or_else(|| anyhow::anyhow!("step size is zero"))?;

        if prev_step_index >= trigger_data.step_count {
            return Ok(None);
        }

        let next_step_index = prev_step_index
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("step index has overflowed"))?;

        let next_step_size_from_start =
            step_size.checked_mul(next_step_index).ok_or_else(|| {
                anyhow::anyhow!(
                    "next step size from start has overflowed (step_size={}, next_step_index={})",
                    step_size,
                    next_step_index
                )
            })?;

        Ok(trigger_data
            .start_height
            .checked_add(next_step_size_from_start))
    }

    /// Set a trigger for an auction.
    fn set_trigger_for_id(&mut self, auction_id: AuctionId, trigger_height: u64) {
        let trigger_path = state_key::dutch::trigger::auction_at_height(auction_id, trigger_height);
        self.put_raw(trigger_path, vec![]);
    }

    /// Delete a trigger for an auction.
    fn unset_trigger_for_id(&mut self, auction_id: AuctionId, trigger_height: u64) {
        let trigger_path = state_key::dutch::trigger::auction_at_height(auction_id, trigger_height);
        self.delete(trigger_path);
    }
}

impl<T: StateWrite + ?Sized> Inner for T {}

struct TriggerData {
    pub start_height: u64,
    pub end_height: u64,
    pub current_height: u64,
    pub step_count: u64,
}

#[cfg(tests)]
mod tests {}
