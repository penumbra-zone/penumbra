use std::pin::Pin;

use crate::auction::dutch::{DutchAuction, DutchAuctionDescription, DutchAuctionState};
use crate::auction::AuctionId;
use crate::component::trigger_data::TriggerData;
use crate::component::AuctionStoreRead;
use crate::state_key;
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use futures::StreamExt;
use penumbra_asset::{Balance, Value};
use penumbra_dex::component::{PositionManager, PositionRead};
use penumbra_dex::lp::position::Position;
use penumbra_dex::lp::Reserves;
use penumbra_dex::DirectedTradingPair;
use penumbra_num::Amount;
use penumbra_proto::core::component::auction::v1alpha1 as pb;
use penumbra_proto::StateWriteProto;
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

        let auction_trigger = TriggerData {
            start_height,
            end_height,
            step_count,
        };

        let current_height = self
            .get_block_height()
            .await
            .expect("block height is not missing");

        let next_trigger = auction_trigger
            .compute_next_trigger_height(current_height)
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
        self.set_trigger_for_dutch_id(auction_id, next_trigger);
        // Write position to state
        self.write_dutch_auction_state(dutch_auction);
    }

    /// Execute the [`DutchAuction`] associated with [`AuctionId`], ticking its
    /// internal state using its immutable description.
    ///
    /// For a given auction, this translates into withdrawing a PCL liquidity position,
    /// credit and zero-out its reserves, and finally, examine the auction's termination
    /// condition.
    async fn execute_dutch_auction(
        &mut self,
        auction_id: AuctionId,
        trigger_height: u64,
    ) -> Result<()> {
        let old_dutch_auction = self
            .get_dutch_auction_by_id(auction_id)
            .await
            .expect("no deserialization errors")
            .expect("the auction exists");

        let DutchAuctionDescription {
            input,
            output_id,
            max_output: _,
            min_output: _,
            start_height,
            end_height,
            step_count,
            nonce: _,
        } = old_dutch_auction.description;

        let DutchAuctionState {
            sequence: _,
            current_position,
            next_trigger: _,
            input_reserves: _,
            output_reserves: _,
        } = old_dutch_auction.state;

        let auction_input_id = input.asset_id;
        let auction_output_id = output_id;

        let auction_trigger = TriggerData {
            start_height,
            end_height,
            step_count,
        };

        // Recover the LP's balances, if it exists.
        let lp_reserves = if let Some(auction_lp_id) = current_position {
            self.close_position_by_id(&auction_lp_id)
                .await
                .map_err(|e| {
                    tracing::error!(
                        ?e,
                        ?auction_lp_id,
                        ?auction_id,
                        "failed to close dutch auction LP"
                    )
                })
                .expect("position should exist and be opened or closed");
            self.withdraw_position(auction_lp_id, 0u64)
                .await
                .map_err(|e| {
                    tracing::error!(
                        ?e,
                        ?auction_lp_id,
                        ?auction_id,
                        "failed to close dutch auction LP"
                    )
                })
                .expect("no state incoherence")
        } else {
            Balance::zero()
        };

        // We remove the execution trigger that we are currently processing:
        self.unset_trigger_for_dutch_id(auction_id, trigger_height);

        // Prepare a new auction, based on the previous one.
        let mut new_dutch_auction = DutchAuction {
            description: old_dutch_auction.description,
            state: old_dutch_auction.state,
        };

        let balance_input_asset = lp_reserves
            .provided()
            .filter(|v| v.asset_id == input.asset_id)
            .map(|v| v.amount)
            .sum::<Amount>();
        let balance_output_asset = lp_reserves
            .provided()
            .filter(|v| v.asset_id == output_id)
            .map(|v| v.amount)
            .sum::<Amount>();

        new_dutch_auction.state.current_position = None;
        // We overwrite the input amount
        new_dutch_auction.state.input_reserves = balance_input_asset;
        new_dutch_auction.state.output_reserves += balance_output_asset;
        new_dutch_auction.state.next_trigger = 0;

        // Compute the current step index, between 0 and `step_count`.
        let step_index = auction_trigger
            .compute_step_index(trigger_height)
            .expect("trigger data is validated");

        if step_index >= step_count {
            // If the termination condition has been reached, we set the auction
            // sequence to 1 (Closed).
            new_dutch_auction.state.sequence = 1;
        } else {
            // Otherwise, we compute the next trigger height and generate a liquidity
            // position for the new auction round.
            let next_trigger = auction_trigger
                .compute_next_trigger_height(trigger_height)
                .expect("trigger data is validated")
                .expect("the step count has not been reached");

            // We compute the price parameters for the LP:
            let (p, q) = compute_pq_at_step(&new_dutch_auction.description, step_index);
            let pair = DirectedTradingPair::new(auction_input_id, auction_output_id);
            let nonce = new_dutch_auction.description.nonce.clone();
            let lp_reserves = Reserves {
                r1: new_dutch_auction.state.input_reserves,
                r2: Amount::zero(),
            };
            // Finally, we create a new position and set it to be auto-closing.
            let mut lp = Position::new_with_nonce(nonce, pair, 0u32, p, q, lp_reserves);
            lp.close_on_fill = true;
            let id = lp.id();
            new_dutch_auction.state.current_position = Some(id);

            // We open the position, and, on success, record an execution trigger:
            let _ = self.open_position(lp).await?;
            self.set_trigger_for_dutch_id(auction_id, next_trigger);
        };

        // We write-back the updated auction state, and complete execution.
        self.write_dutch_auction_state(new_dutch_auction);

        Ok(())
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
        self.close_auction(auction).await
    }

    /// Terminate and update the supplied auction state.
    async fn close_auction(&mut self, auction_to_close: DutchAuction) -> Result<()> {
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
        let (input_from_position, output_from_position) =
            if let Some(position_id) = current_position {
                use penumbra_dex::component::{PositionManager, PositionRead};

                let _ = self // TODO: redundant.
                    .position_by_id(&position_id)
                    .await
                    .expect("no deserialization error")
                    .expect("position MUST exist");

                let _ = self.close_position_by_id(&position_id).await?;
                let balance = self.withdraw_position(position_id, 0).await?;

                let input_id = auction_to_close.description.input.asset_id;
                let output_id = auction_to_close.description.output_id;

                let input_balance = balance
                    .provided()
                    .filter(|v| v.asset_id == input_id)
                    .map(|v| v.amount)
                    .sum::<Amount>();

                let output_balance = balance
                    .provided()
                    .filter(|v| v.asset_id == output_id)
                    .map(|v| v.amount)
                    .sum::<Amount>();

                (input_balance, output_balance)
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

    fn withdraw_auction(&mut self, mut auction: DutchAuction) -> Balance {
        let previous_input_reserves = Balance::from(Value {
            amount: auction.state.input_reserves,
            asset_id: auction.description.input.asset_id,
        });
        let previous_output_reserves = Balance::from(Value {
            amount: auction.state.output_reserves,
            asset_id: auction.description.output_id,
        });

        let withdraw_balance = previous_input_reserves + previous_output_reserves;

        auction.state.sequence = auction.state.sequence.saturating_add(1);
        auction.state.current_position = None;
        auction.state.next_trigger = 0;
        auction.state.input_reserves = Amount::zero();
        auction.state.output_reserves = Amount::zero();
        self.write_dutch_auction_state(auction);

        withdraw_balance
    }
}

impl<T: StateWrite + ?Sized> DutchAuctionManager for T {}

trait Inner: StateWrite {
    /// Serialize a `DutchAuction` as an `Any` into chain state.
    fn write_dutch_auction_state(&mut self, new_state: DutchAuction) {
        let id = new_state.description.id();
        let key = state_key::auction_store::by_id(id);
        let pb_state: pb::DutchAuction = new_state.into();
        let raw_auction = pb_state.encode_to_vec();

        let any_auction = prost_types::Any {
            type_url: pb::DutchAuction::type_url(),
            value: raw_auction,
        };

        let raw_any = any_auction.encode_to_vec();

        self.put_raw(key, raw_any);
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

/// TODO(erwan): prove that this works. More work needed to work out validation
/// rules that let us operate within the 112 bits constraint.
fn compute_pq_at_step(
    auction_description: &DutchAuctionDescription,
    step_index: u64,
) -> (Amount, Amount) {
    let step_index = Amount::from(step_index);
    let step_count = Amount::from(auction_description.step_count);

    let q = auction_description
        .input
        .amount
        .checked_mul(&step_count)
        .unwrap();

    let p = (step_count - step_index) * auction_description.max_output
        + step_index * auction_description.min_output;

    (p, q)
}
