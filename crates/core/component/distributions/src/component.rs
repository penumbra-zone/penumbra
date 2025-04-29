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
use penumbra_sdk_proto::{DomainType as _, StateWriteProto};
use penumbra_sdk_sct::component::clock::EpochRead;
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
        let epoch_index = current_epoch.index;

        let params = state
            .get_distributions_params()
            .await
            .expect("distribution parameters should be available");
        let increase = Amount::from(params.liquidity_tournament_incentive_per_block);
        let current_block = u64::try_from(end_block.height).unwrap_or_default();
        // If the end_block is defined, and the current block is at least past there,
        // then we want to make sure to not issue any rewards.
        if let Some(end_block) = params
            .liquidity_tournament_end_block
            .filter(|b| current_block >= b.get())
        {
            // Doing this unconditionally for robustness. This should, in theory,
            // only need to be an edge trigger though, like the event.
            tracing::debug!(epoch_index, "zeroing out LQT reward issuance");
            state.set_lqt_reward_issuance_for_epoch(epoch_index, 0u64.into());
            if current_block == end_block.get() {
                state.record_proto(
                    event::EventLqtPoolSizeIncrease {
                        epoch_index,
                        increase: 0u64.into(),
                        new_total: 0u64.into(),
                    }
                    .to_proto(),
                )
            }
            return;
        }

        let new_total = state.increment_lqt_issuance(epoch_index, increase).await;

        // Emit an event for LQT pool size increase at the end of the block.
        state.record_proto(
            event::EventLqtPoolSizeIncrease {
                epoch_index,
                increase,
                new_total,
            }
            .to_proto(),
        )
    }

    #[instrument(name = "distributions", skip(state))]
    async fn end_epoch<S: StateWrite + 'static>(state: &mut Arc<S>) -> Result<()> {
        let state = Arc::get_mut(state).context("state should be unique")?;

        // Define staking budget.
        state.define_staking_budget().await?;

        // The lqt issuance budget is adjusted every block instead.

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

    async fn increment_lqt_issuance(&mut self, epoch_index: u64, by: Amount) -> Amount {
        let current = self.get_lqt_reward_issuance_for_epoch(epoch_index).await;
        let new = current
            .unwrap_or_default()
            .checked_add(&by)
            .expect("LQT issuance should never exceed an Amount");
        tracing::debug!(
            ?by,
            ?current,
            ?new,
            epoch_index,
            "incrementing lqt issuance"
        );
        self.set_lqt_reward_issuance_for_epoch(epoch_index, new);
        new
    }
}

impl<T: StateWrite + ?Sized> DistributionManager for T {}
