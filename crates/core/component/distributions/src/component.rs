pub mod state_key;

mod view;

use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_component::Component;
use penumbra_num::Amount;
use penumbra_storage::StateWrite;
use tendermint::v0_37::abci;
use tracing::instrument;
pub use view::{StateReadExt, StateWriteExt};

pub struct Distributions {}

#[async_trait]
impl Component for Distributions {
    type AppState = ();

    #[instrument(name = "distributions", skip(_state, app_state))]
    async fn init_chain<S: StateWrite>(mut _state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            None => { /* Checkpoint -- no-op */ }
            Some(_) => { /* no-op, future check of genesis chain parameters? */ }
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
        use penumbra_chain::component::StateReadExt as _;
        let current_block_height = self.get_block_height().await?;
        let current_epoch = self.get_epoch_for_height(current_block_height).await?;
        let num_blocks = current_block_height
            .checked_sub(current_epoch.start_height)
            .expect("epoch start height is less than or equal to current block height");

        let staking_issuance_per_block = self
            .get_distributions_params()
            .await?
            .staking_issuance_per_block;

        tracing::debug!(
            number_of_blocks_in_epoch = num_blocks,
            staking_issuance_per_block,
            "calculating issuance per epoch"
        );

        let new_issuance_for_epoch = staking_issuance_per_block
            .checked_mul(num_blocks)
            .expect("infaillible unless issuance is pathological");

        Ok(Amount::from(new_issuance_for_epoch))
    }

    /// Update the object store with the new issuance of staking tokens for this epoch.
    async fn distribute(&mut self, new_issuance: Amount) {
        self.set_staking_token_issuance_for_epoch(new_issuance)
    }
}

impl<T: StateWrite + ?Sized> DistributionManager for T {}
