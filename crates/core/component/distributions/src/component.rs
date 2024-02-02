pub mod state_key;
pub use view::{StateReadExt, StateWriteExt};

mod view;

use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::Component;
use penumbra_asset::STAKING_TOKEN_DENOM;
use penumbra_num::Amount;
use tendermint::v0_37::abci;
use tracing::instrument;

use crate::genesis;

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
        let new_issuance = state.compute_new_issuance().await?;
        tracing::debug!(?new_issuance, "computed new issuance for epoch");
        Ok(state.distribute(new_issuance).await)
    }
}

#[async_trait]
trait DistributionManager: StateWriteExt {
    /// Compute the total new issuance of staking tokens for this epoch.
    async fn compute_new_issuance(&self) -> Result<Amount> {
        use penumbra_sct::component::clock::EpochRead;

        let current_block_height = self.get_block_height().await?;
        let current_epoch = self.get_current_epoch().await?;
        let num_blocks = current_block_height
            .checked_sub(current_epoch.start_height)
            .unwrap_or_else(|| panic!("epoch start height is less than or equal to current block height (epoch_start={}, current_height={}", current_epoch.start_height, current_block_height));

        // TODO(erwan): Will make the distribution chain param an `Amount`
        // in a subsequent PR. Want to avoid conflicts with other in-flight changes.
        let staking_issuance_per_block = self
            .get_distributions_params()
            .await?
            .staking_issuance_per_block as u128;

        tracing::debug!(
            number_of_blocks_in_epoch = num_blocks,
            staking_issuance_per_block,
            "calculating issuance per epoch"
        );

        let new_issuance_for_epoch = staking_issuance_per_block
            .checked_mul(num_blocks as u128) /* Safe to cast a `u64` to `u128` */
            .expect("infaillible unless issuance is pathological");

        tracing::debug!(
            ?new_issuance_for_epoch,
            "computed new issuance for epoch (pre-scaled)"
        );

        let new_issuance_for_epoch = STAKING_TOKEN_DENOM
            .default_unit()
            .value(new_issuance_for_epoch.into())
            .amount;

        tracing::debug!(
            ?new_issuance_for_epoch,
            "computed new issuance for epoch (scaled)"
        );
        Ok(Amount::from(new_issuance_for_epoch))
    }

    /// Update the object store with the new issuance of staking tokens for this epoch.
    async fn distribute(&mut self, new_issuance: Amount) {
        self.set_staking_token_issuance_for_epoch(new_issuance)
    }
}

impl<T: StateWrite + ?Sized> DistributionManager for T {}
