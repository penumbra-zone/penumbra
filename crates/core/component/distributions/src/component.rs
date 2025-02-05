pub mod state_key;
pub use view::{StateReadExt, StateWriteExt};
pub mod rpc;

mod view;

use std::sync::Arc;

use crate::event;
use crate::genesis;
use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::Component;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::StateWriteProto;
use penumbra_sdk_sct::{component::clock::EpochRead, epoch::Epoch};
use tendermint::v0_37::abci;
use tracing::instrument;

pub struct Distributions {}

#[async_trait]
impl Component for Distributions {
    type AppState = genesis::Content;

    #[instrument(name = "distributions", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            None => { /* Checkpoint -- no-op */ }
            Some(genesis) => {
                state.put_distributions_params(genesis.distributions_params.clone());
            }
        };
    }

    #[instrument(name = "distributions", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    #[instrument(name = "distributions", skip(state))]
    async fn end_block<S: StateWrite + 'static>(
        state: &mut Arc<S>,
        end_block: &abci::request::EndBlock,
    ) {
        let state = Arc::get_mut(state).expect("the state should not be shared");

        let current_epoch = state
            .get_current_epoch()
            .await
            .expect("failed to retrieve current epoch from state");
        let current_block_height = end_block
            .height
            .try_into()
            .expect("block height should not be negative");

        let new_issuance = state
            .get_distributions_params()
            .await
            .expect("distribution parameters should be available")
            .staking_issuance_per_block as u128;

        let total_new_issuance = state
            .compute_lqt_issuance_from_blocks(current_epoch, current_block_height)
            .await
            .expect("should be able to compute LQT issuance from block");

        // Emit an event for LQT pool size increase at the end of the block.
        state.record_proto(event::event_lqt_pool_size_increase(
            current_epoch.index,
            new_issuance.into(),
            total_new_issuance,
        ))
    }

    #[instrument(name = "distributions", skip(state))]
    async fn end_epoch<S: StateWrite + 'static>(state: &mut Arc<S>) -> Result<()> {
        let state = Arc::get_mut(state).context("state should be unique")?;

        // Define staking budget.
        state.define_staking_budget().await?;

        // Define LQT budget.
        state.define_lqt_budget().await?;

        Ok(())
    }
}

#[async_trait]
trait DistributionManager: StateWriteExt {
    /// Compute the total new issuance of staking tokens for this epoch.
    async fn compute_new_staking_issuance(&self) -> Result<Amount> {
        use penumbra_sdk_sct::component::clock::EpochRead;

        let current_block_height = self.get_block_height().await?;
        let current_epoch = self.get_current_epoch().await?;
        let num_blocks = current_block_height
            .checked_sub(current_epoch.start_height)
            .unwrap_or_else(|| panic!("epoch start height is greater than current block height (epoch_start={}, current_height={}", current_epoch.start_height, current_block_height));

        // TODO(erwan): Will make the distribution chain param an `Amount`
        // in a subsequent PR. Want to avoid conflicts with other in-flight changes.
        let staking_issuance_per_block = self
            .get_distributions_params()
            .await?
            .staking_issuance_per_block as u128;

        tracing::debug!(
            number_of_blocks_in_epoch = num_blocks,
            staking_issuance_per_block,
            "calculating staking issuance per epoch"
        );

        let new_staking_issuance_for_epoch = staking_issuance_per_block
            .checked_mul(num_blocks as u128) /* Safe to cast a `u64` to `u128` */
            .expect("infallible unless issuance is pathological");

        tracing::debug!(
            ?new_staking_issuance_for_epoch,
            "computed new staking issuance for epoch"
        );

        Ok(Amount::from(new_staking_issuance_for_epoch))
    }

    /// Update the object store with the new issuance of staking tokens for this epoch.
    async fn define_staking_budget(&mut self) -> Result<()> {
        let new_issuance = self.compute_new_staking_issuance().await?;
        tracing::debug!(
            ?new_issuance,
            "computed new staking issuance for current epoch"
        );
        Ok(self.set_staking_token_issuance_for_epoch(new_issuance))
    }

    /// Helper function that computes the LQT issuance for the current block height in the epoch.
    async fn compute_lqt_issuance_from_blocks(
        &self,
        current_epoch: Epoch,
        current_block_height: u64,
    ) -> Result<Amount> {
        let epoch_length = current_block_height
            .checked_sub(current_epoch.start_height)
            .unwrap_or_else(|| panic!("epoch start height is greater than current block height (epoch_start={}, current_height={}", current_epoch.start_height, current_block_height));

        let lqt_block_reward_rate = self
            .get_distributions_params()
            .await?
            .liquidity_tournament_incentive_per_block as u64;

        tracing::debug!(
            number_of_blocks_in_epoch = epoch_length,
            lqt_block_reward_rate,
            "calculating lqt reward issuance per epoch"
        );

        let total_pool_size_for_epoch = lqt_block_reward_rate
            .checked_mul(epoch_length as u64)
            .expect("infallible unless issuance is pathological");

        tracing::debug!(
            ?total_pool_size_for_epoch,
            "computed new reward lqt issuance for epoch"
        );

        Ok(Amount::from(total_pool_size_for_epoch))
    }

    /// Computes total LQT reward issuance for the epoch.
    async fn compute_new_lqt_issuance(&self, current_epoch: Epoch) -> Result<Amount> {
        let current_block_height = self.get_block_height().await?;

        Ok(self
            .compute_lqt_issuance_from_blocks(current_epoch, current_block_height)
            .await?)
    }

    /// Update the nonverifiable storage with the newly issued LQT rewards for the current epoch.
    async fn define_lqt_budget(&mut self) -> Result<()> {
        // Grab the ambient epoch index.
        let current_epoch = self.get_current_epoch().await?;

        // New issuance for the current epoch.
        let new_issuance = self.compute_new_lqt_issuance(current_epoch).await?;
        tracing::debug!(
            ?new_issuance,
            "computed new lqt reward issuance for current epoch {}",
            current_epoch.index
        );

        Ok(self.set_lqt_reward_issuance_for_epoch(current_epoch.index, new_issuance))
    }
}

impl<T: StateWrite + ?Sized> DistributionManager for T {}
