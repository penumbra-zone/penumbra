pub mod state_key;

mod view;

use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use anyhow::Result;
use async_trait::async_trait;
use penumbra_asset::{asset, STAKING_TOKEN_ASSET_ID};
use penumbra_chain::{component::StateReadExt as _, genesis};
use penumbra_component::Component;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};
use tracing::instrument;
pub use view::{StateReadExt, StateWriteExt};

#[allow(unused_imports)]
use penumbra_dex::{component::StateReadExt as _, component::StateWriteExt as _};
use penumbra_stake::{component::StateWriteExt as _, StateReadExt as _};

pub struct Distributions {}

#[async_trait]
impl Component for Distributions {
    type AppState = ();

    #[instrument(name = "distributions", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            None => {}
            Some(app_state) => {
                // Tally up the total issuance of the staking token from the genesis allocations, so that we
                // can accurately track the total amount issued in the future.
                let genesis_issuance = app_state
                    .allocations
                    .iter()
                    .filter(|alloc| {
                        // Filter only for allocations of the staking token
                        asset::REGISTRY.parse_denom(&alloc.denom).map(|d| d.id())
                            == Some(*STAKING_TOKEN_ASSET_ID)
                    })
                    .fold(0u64, |sum, alloc| {
                        // Total the allocations
                        sum.checked_add(
                            u128::from(alloc.amount)
                                .try_into()
                                .expect("genesis issuance does not overflow `u64`"),
                        )
                        .expect("genesis issuance does not overflow `u64`")
                    });
                tracing::info!(
                    "total genesis issuance of staking token: {}",
                    genesis_issuance
                );
                state.set_total_issued(genesis_issuance);
            }
        }
    }

    #[instrument(name = "distributions", skip(state))]
    async fn end_epoch<S: StateWrite + 'static>(state: &mut Arc<S>) -> Result<()> {
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

        tracing::info!(
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

        tracing::info!(new_issuance = ?issuance, new_remainder = ?remainder);

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
                tracing::info!(%component, ?issuance, "issuing tokens to component"
                );
                match component {
                    Staking => state.set_staking_issuance(issuance),
                    Dex => todo!("set dex issuance"),
                }
            }
        }

        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum ComponentName {
    Staking,
    Dex,
}

impl Display for ComponentName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ComponentName::Staking => write!(f, "staking"),
            ComponentName::Dex => write!(f, "dex"),
        }
    }
}

#[async_trait]
trait DistributionsImpl
where
    Self: StateRead + StateWrite,
{
    // Compute the total issuance for this epoch, and the remainder that will be carried over to
    // the next epoch, given the remainder that was carried forward from the preceding epoch.
    async fn total_issuance_and_remainder(&self, remainder: u64) -> Result<(u64, u64)> {
        // This currently computes the new issuance by multiplying the total staking token ever
        // issued by the base reward rate. This is a stand-in for a more accurate and good model of
        // issuance, which will be implemented later. For now, this inflates the total issuance of
        // staking tokens by a fixed ratio per epoch.
        let base_reward_rate = self.get_chain_params().await?.base_reward_rate;
        let total_issued = self.total_issued().await?;
        const BPS_SQUARED: u64 = 1_0000_0000; // reward rate is measured in basis points squared
        let new_issuance = total_issued * base_reward_rate / BPS_SQUARED;
        let issuance = new_issuance + remainder;
        Ok((issuance, 0))
    }

    // Determine in each epoch what the relative weight of issuance per component should be. The
    // returned list of weights is used to allocate the total issuance between the different
    // components, and does not need to sum to any particular total; it will be rescaled to the
    // total issuance determined by `total_issuance_and_remainder`.
    async fn issuance_weights(&self) -> Result<Vec<(ComponentName, u128)>> {
        // Currently, only issue staking rewards:
        Ok(vec![(ComponentName::Staking, 1)])
    }

    // Get the remainder of the issuance that couldn't be distributed in the previous epoch.
    async fn remainder(&self) -> Result<u64> {
        self.get_proto(state_key::remainder())
            .await
            .map(Option::unwrap_or_default)
    }

    // Set the remainder of the issuance that will be carried forward to the next epoch.
    fn set_remainder(&mut self, remainder: u64) {
        self.put_proto(state_key::remainder().to_string(), remainder)
    }

    // Get the total issuance of staking tokens for all time.
    async fn total_issued(&self) -> Result<u64> {
        self.get_proto(state_key::total_issued())
            .await
            .map(Option::unwrap_or_default)
    }

    // Set the total issuance of staking tokens for all time.
    fn set_total_issued(&mut self, total_issued: u64) {
        self.put_proto(state_key::total_issued().to_string(), total_issued)
    }
}

impl<S: StateRead + StateWrite> DistributionsImpl for S {}
