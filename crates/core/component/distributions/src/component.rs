pub mod state_key;

mod view;

use std::{
    iter::Sum,
    ops::{Add, Div, Mul, Shr, Sub},
    sync::Arc,
};

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::{component::StateReadExt as _, genesis, params::ChainParameters};
use penumbra_component::Component;
use penumbra_storage::StateWrite;
use tendermint::abci;
pub use view::{StateReadExt, StateWriteExt};

use penumbra_dex::{component::StateReadExt as _, component::StateWriteExt as _};
use penumbra_stake::{component::StateWriteExt as _, StateReadExt as _};

pub struct Distributions {}

#[async_trait]
impl Component for Distributions {
    type AppState = ();

    async fn init_chain<S: StateWrite>(_state: S, _app_state: Option<&()>) {}

    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    async fn end_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
    }

    async fn end_epoch<S: StateWrite + 'static>(state: &mut Arc<S>) -> Result<()> {
        let state = Arc::get_mut(state).expect("state `Arc` is unique");

        // // Get the remainders of issuances that couldn't be distributed last epoch, due to precision
        // // loss or lack of activity.
        // let staking_remainder: u64 = state.staking_issuance().await?;
        // let dex_remainder: u64 = 0; // TODO: get this from the dex once LP rewards are implemented
        // let remainder = staking_remainder
        //     .checked_add(dex_remainder)
        //     .ok_or_else(|| {
        //         anyhow::anyhow!("staking and dex remainders overflowed when added together")
        //     })?
        //     .checked_add(state.remainder().await?)
        //     .ok_or_else(|| {
        //         anyhow::anyhow!(
        //             "staking and dex remainders overflowed when added to the previous remainder"
        //         )
        //     })?;

        // // Clear out the remaining issuances, so that if we don't issue anything to one of them, we
        // // don't leave the remainder there.
        // state.set_staking_issuance(0);
        // // TODO: clear dex issuance

        // // Get the total issuance and new remainder for this epoch
        // let (issuance, remainder) = state.total_issuance(remainder).await?;

        // // Set the remainder to be carried over to the next epoch
        // state.set_remainder(remainder).await?;

        Ok(())
    }
}

fn exact_allocation<K: Ord>(
    mut total_allocation: u128,
    mut weights: Vec<(K, u128)>,
) -> Vec<(K, u128)> {
    // 1. Sort the weights in ascending order.
    weights.sort();

    // 2. Calculate the total weight.
    let mut total_weight: u128 = weights.iter().map(|(_, weight)| *weight).sum();

    // Unsigned rounding division algorithm: add half the divisor (truncating down) to the dividend,
    // then divide, which means that this is (dividend / divisor) + ((divisor / 2) / divisor), the
    // latter addend of which is 0 when the divisor is less than half the remainder, and 1
    // otherwise: exactly what we want for rounding division.
    fn rounding_div(dividend: u128, divisor: u128) -> u128 {
        (dividend + (divisor >> 1)) / divisor
    }

    // 3. For each key in the weights, calculate the allocation for that key, sequentially iterating
    //    from least-weighted to most-weighted. This minimizes the *percentage* error in the
    //    allocations, because as the total allocation decreases, the amount of error in a rounding
    //    division increases, but since we are ascending the weights, we're pushing higher error
    //    (which, notably, is capped at 1 unit of allocation, maximum!) to the most-weighted keys,
    //    which means that the average *percentage* error is minimized, since the total allocation
    //    to the most-weighted keys is the highest, and therefore the absolute error matters least
    //    to them. If two keys are equally-weighted, then it could happen that one key gets 1 unit
    //    of allocation more than the other: this is deterministic based on comparing the keys,
    //    since we sort the weights in ascending lexicographic order of (key, weight).
    weights
        .into_iter()
        .map(|(key, weight)| {
            // a. The allocation for this key is the total allocation times the fraction of the total
            // weight that this key has, rounded to the nearest integer.
            let allocation = rounding_div(total_allocation * weight, total_weight);
            // b. The remaining total weight to distribute should subtract this weight, because it was assigned.
            total_weight = total_weight - weight;
            // c. The remaining total allocation to distribute should subtract this allocation, because it was assigned.
            total_allocation = total_allocation - allocation;
            // d. Return the key and its allocation.
            (key, allocation)
        })
        .collect()
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn exact_allocation_is_exact(
            total_allocation in 0u128..u128::MAX,
            weights in proptest::collection::vec((0u8..u8::MAX, 1u128..u128::MAX), 1..(u8::MAX as usize))
        ) {
            let allocation = super::exact_allocation(total_allocation, weights.clone());
            let actual_total_allocation: u128 = allocation.iter().map(|(_, allocation)| *allocation).sum();

            prop_assert_eq!(total_allocation, actual_total_allocation, "total allocation is not exact");
            prop_assert_eq!(allocation.len(), weights.len(), "number of allocations is not exact");

            let mut initial_key_set = weights.iter().map(|(key, _)| key).collect::<Vec<_>>();
            initial_key_set.sort();
            let mut actual_key_set = allocation.iter().map(|(key, _)| key).collect::<Vec<_>>();
            actual_key_set.sort();
            prop_assert_eq!(initial_key_set, actual_key_set, "keys are not the same multiset of keys");
        }
    }
}
