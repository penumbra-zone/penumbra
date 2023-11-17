pub mod state_key;

mod view;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_component::Component;
// use penumbra_dex::{component::StateReadExt as _, component::StateWriteExt as _};
// use penumbra_stake::{component::StateWriteExt as _, StateReadExt as _};
use penumbra_asset::STAKING_TOKEN_ASSET_ID;
use penumbra_num::Amount;
use penumbra_shielded_pool::component::SupplyRead;
use penumbra_storage::StateWrite;
use tendermint::v0_37::abci;
use tracing::instrument;
pub use view::{StateReadExt, StateWriteExt};

pub struct Distributions {}

#[async_trait]
impl Component for Distributions {
    type AppState = ();

    #[instrument(name = "distributions", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            None => { /* Checkpoint -- no-op */ }
            Some(_) => {
                let genesis_issuance = state
                    .token_supply(&*STAKING_TOKEN_ASSET_ID)
                    .await
                    .expect("supply is valid")
                    .expect("shielded pool component has tallied genesis issuance");
                tracing::debug!(
                    "total genesis issuance of staking token: {}",
                    genesis_issuance
                );
                // TODO(erwan): it's not yet totally clear if it is necessary, or even desirable, for the
                // distributions component to track the total issuance. The shielded pool component
                // already does that. We do it anyway for now so that we can write the rest of the scaffolding.
                state.set_total_issued(genesis_issuance);
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

    #[instrument(name = "distributions", skip(_state))]
    async fn end_epoch<S: StateWrite + 'static>(_state: &mut Arc<S>) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
trait DistributionManager: StateWriteExt {
    /// Compute the total new issuance of staking tokens for this epoch.
    /// TODO(erwan): this is a stub implementation.
    async fn compute_new_issuance(&self) -> Result<Amount> {
        let base_reward_rate: u64 = 0;
        let total_issued = self
            .total_issued()
            .await?
            .expect("total issuance has been initialized");
        const BPS_SQUARED: u64 = 1_0000_0000; // reward rate is measured in basis points squared
        let new_issuance = total_issued * base_reward_rate / BPS_SQUARED;
        Ok(new_issuance.into())
    }

    /// Update the object store with the new issuance of staking tokens for this epoch.
    /// TODO(erwan): this is a stub implementation.
    async fn distribute(&mut self) -> Result<()> {
        let new_issuance = self.compute_new_issuance().await?;
        tracing::debug!(?new_issuance, "computed new issuance for epoch");
        self.set_total_issued(new_issuance);
        todo!()
    }
}
