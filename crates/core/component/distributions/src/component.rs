pub mod state_key;

mod view;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_component::Component;
// use penumbra_dex::{component::StateReadExt as _, component::StateWriteExt as _};
// use penumbra_stake::{component::StateWriteExt as _, StateReadExt as _};
use penumbra_asset::STAKING_TOKEN_ASSET_ID;
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
        /*
                let state = Arc::get_mut(state).expect("state `Arc` is unique");

                // Get the remainders of issuances that couldn't be distributed last epoch, due to precision
                // loss or lack of activity.
                let staking_remainder: u64 = state.staking_issuance().await?;
                let dex_remainder: u64 = 0; // TODO: get this from the dex once LP rewards are implemented

                // Sum all the per-component remainders together, including any remainder in the
                // distribution component itself left over undistributed in the previous epoch
                let last_epoch_remainder =
                    staking_remainder
                        .checked_add(dex_remainder)
                        .ok_or_else(|| {
                            anyhow::anyhow!("staking and dex remainders overflowed when added together")
                        })?;

                // The remainder from the previous epoch could not be issued, so subtract it from the total
                // issuance for all time.
                let total_issued = state
                    .total_issued()
                    .await?
                    .checked_sub(last_epoch_remainder)
                    .expect(
                        "total issuance is greater than or equal to the remainder from the previous epoch",
                    );
                state.set_total_issued(total_issued);

                // Add the remainder from the previous epoch to the remainder carried over from before then.
                let remainder = last_epoch_remainder
                    .checked_add(state.remainder().await?)
                    .expect("remainder does not overflow `u64`");

                tracing::debug!(
                    ?remainder,
                    ?last_epoch_remainder,
                    ?staking_remainder,
                    ?dex_remainder,
                );

                // Clear out the remaining issuances, so that if we don't issue anything to one of them, we
                // don't leave the remainder there.
                state.set_staking_issuance(0);
                // TODO: clear dex issuance

                // Get the total issuance and new remainder for this epoch
                let (issuance, remainder) = state.total_issuance_and_remainder(remainder).await?;

                tracing::debug!(new_issuance = ?issuance, new_remainder = ?remainder);

                // Set the remainder to be carried over to the next epoch
                state.set_remainder(remainder);

                // Set the cumulative total issuance (pending receipt of remainders, which may decrease it
                // next epoch)
                state.set_total_issued(total_issued + issuance);

                // Determine the allocation of the issuance between the different components: this returns a
                // set of weights, which we'll use to scale the total issuance
                let weights = state.issuance_weights().await?;

                // Allocate the issuance according to the weights
                if let Some(allocation) = penumbra_num::allocate(issuance.into(), weights) {
                    for (component, issuance) in allocation {
                        use ComponentName::*;
                        let issuance: u64 = issuance.try_into().expect("total issuance is within `u64`");
                        tracing::debug!(%component, ?issuance, "issuing tokens to component"
                        );
                        match component {
                            Staking => state.set_staking_issuance(issuance),
                            Dex => todo!("set dex issuance"),
                        }
                    }
                }
        */
        Ok(())
    }
}

#[async_trait]
trait DistributionManager: StateWriteExt {
    /// Compute the total new issuance of staking tokens for this epoch.
    /// TODO(erwan): this is a stub implementation.
    async fn compute_new_issuance(&self) -> Result<u64> {
        let base_reward_rate: u64 = 0;
        let total_issued = self
            .total_issued()
            .await?
            .expect("total issuance has been initialized");
        const BPS_SQUARED: u64 = 1_0000_0000; // reward rate is measured in basis points squared
        let new_issuance = total_issued * base_reward_rate / BPS_SQUARED;
        Ok(new_issuance)
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
