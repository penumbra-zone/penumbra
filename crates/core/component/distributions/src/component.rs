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

    #[instrument(name = "distributions", skip(_state, _end_block))]
    async fn end_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
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

    /// Computes total LQT reward issuance for the epoch.
    async fn compute_new_lqt_issuance(&self, current_epoch: Epoch) -> Result<Amount> {
        let current_block_height = self.get_block_height().await?;
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

    /// Update the nonverifiable storage with the newly issued LQT rewards for the current epoch.
    async fn define_lqt_budget(&mut self) -> Result<()> {
        // Grab the ambient epoch index.
        let current_epoch = self.get_current_epoch().await?;

        let new_issuance = self.compute_new_lqt_issuance(current_epoch).await?;
        tracing::debug!(
            ?new_issuance,
            "computed new lqt reward issuance for currentepoch {}",
            current_epoch.index
        );

        // Retrieve the previous cumulative LQT issuance total from NV storage.
        let previous_issuance = self
            .get_cummulative_lqt_reward_issuance(current_epoch.index - 1)
            .await
            .ok_or_else(|| {
                tonic::Status::not_found(format!(
                    "failed to retrieve cumulative LQT issuance for epoch {} from non-verifiable storage",
                    current_epoch.index,
                ))
            })?;

        // Compute total new LQT issuance.
        let total_new_issuance = previous_issuance + new_issuance;

        // Emit an event for LQT pool size increase.
        self.record_proto(event::event_lqt_pool_size_increase(
            current_epoch.index,
            new_issuance,
            total_new_issuance,
        ));

        self.set_lqt_reward_issuance_for_epoch(current_epoch.index, new_issuance);
        self.set_cumulative_lqt_reward_issuance(current_epoch.index, total_new_issuance);

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> DistributionManager for T {}
