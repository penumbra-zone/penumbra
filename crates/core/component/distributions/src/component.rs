pub mod state_key;

mod view;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::{component::StateReadExt as _, genesis, params::ChainParameters};
use penumbra_component::Component;
use penumbra_num::fixpoint::U128x128;
use penumbra_storage::StateWrite;
use tendermint::abci;
pub use view::{StateReadExt, StateWriteExt};

use penumbra_dex::{component::StateReadExt as _, component::StateWriteExt as _};
use penumbra_stake::{component::StateWriteExt as _, StateReadExt as _};

pub struct Distributions {}

#[async_trait]
impl Component for Distributions {
    type AppState = genesis::AppState;

    async fn init_chain<S: StateWrite>(_state: S, _app_state: &Self::AppState) {}

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

/// Given an association of keys to weights, sum the weights, scaling down the weights uniformly to
/// make sure that the total weight fits in a `u128`, returning the total and the scaled weights.
fn total_and_scale_to_u128<K>(weights: Vec<(K, u128)>) -> (u128, Vec<(K, u128)>) {
    // Calculate the total weight, tracking overflows so we can compute a scaling factor in the case
    // when the total of weights exceeds `u128::MAX`.
    let mut total_weight_remainder: u128 = 0;
    let mut number_of_overflows: u128 = 0;
    for (_, weight) in &weights {
        if let Some(new) = total_weight_remainder.checked_add(*weight) {
            total_weight_remainder = new;
        } else {
            // If the total weight overflows, track the overflow and continue, so we accumulate
            // an exact accounting of the total weight, even if it overflows.
            number_of_overflows += 1;
            // The remaining total weight after we track the overflow is the weight that was going
            // to overflow things, minus the amount remaining prior to overflow (i.e. the remainder).
            total_weight_remainder = weight - (u128::MAX - total_weight_remainder);
        };
    }

    // Compute a scaling factor such that the total weight is scaled down to fit in a `u128` if this
    // scaling factor is applied to each weight.
    let scaling_factor = if number_of_overflows == 0 {
        // If there were no overflows, then the scaling factor is 1.
        U128x128::from(1u8)
    } else {
        // If there were overflows, then the scaling factor is (number_of_overflows . total_weight) as
        // a U128x128.
        U128x128::from_parts(number_of_overflows, total_weight_remainder)
    };

    // Compute a new set of weights and total weight by applying the scaling factor to the weights.
    // Even if there was overflow, the new total weight may be less than `u128::MAX`, since loss of
    // precision when dividing individual weights may have reduced the total weight.
    let mut total_scaled_weight: u128 = 0;
    let mut scaled_weights = Vec::with_capacity(weights.len());
    for (key, weight) in weights {
        // Scale each weight down by dividing it by the scaling factor and rounding down.
        let scaled_weight = (U128x128::from(weight) / scaling_factor)
            .expect("scaling factor is never zero")
            .round_down() // must round *down* to avoid total exceeding `u128::MAX` in all situations
            .try_into()
            .expect("rounded amount is always integral");
        // Track the total scaled weight, so we can return it.
        total_scaled_weight += scaled_weight;
        // Only output the scaled weight if it is greater than zero, since we don't want to do extra
        // work for weights that are dropped by scaling.
        if scaled_weight != 0 {
            scaled_weights.push((key, scaled_weight));
        }
    }

    (total_scaled_weight, scaled_weights)
}

/// Return an exact allocation of `total_allocation` units of allocation, proportioned according to
/// the given `weights`.
///
/// This method minimizes the average error ratio in the allocations, and error is bounded by at
/// most 1 unit of allocation per key. If the total of the weights exceeds `u128::MAX`, an empty
/// list of allocations is returned.
fn exact_allocation<K: Ord>(
    mut total_allocation: u128,
    weights: Vec<(K, u128)>,
) -> Option<Vec<(K, u128)>> {
    // If the total allocation is zero, then we can allocate nothing to any key, regardless of what
    // the weights assigned to each key are.
    if total_allocation == 0 {
        return Some(Vec::new());
    }

    // Scale the weights down to fit in a `u128`.
    let (mut total_weight, mut weights) = total_and_scale_to_u128(weights);

    // If the total weight is zero, then we can't allocate anything, which would violate the
    // guarantee to allocate exactly if we returned any result.
    if total_weight == 0 {
        return None;
    }

    // For each key in the weights, calculate the allocation for that key, sequentially iterating
    // from least-weighted to most-weighted. This minimizes the *percentage* error in the
    // allocations, because as the total allocation decreases, the amount of error in a rounding
    // division increases, but since we are ascending the weights, we're pushing higher error
    // (which, notably, is capped at 1 unit of allocation, maximum!) to the most-weighted keys,
    // which means that the average *percentage* error is minimized, since the total allocation
    // to the most-weighted keys is the highest, and therefore the absolute error matters least
    // to them. If two keys are equally-weighted, then it could happen that one key gets 1 unit
    // of allocation more than the other: this is deterministic based on comparing the keys,
    // since we sort the weights in ascending lexicographic order of (key, weight).
    weights.sort();
    weights
        .into_iter()
        .filter_map(|(key, weight)| {
            // The allocation for this key is the total allocation times the fraction of the total
            // weight that this key has, rounded to the nearest integer.
            let fraction_of_total_weight =
                U128x128::ratio(weight, total_weight).expect("total weight is not zero");
            let fractional_allocation = U128x128::from(total_allocation) * fraction_of_total_weight;
            let integral_allocation = fractional_allocation
                .expect("fraction of total weight is never greater than one")
                .round_nearest() // must round to *nearest* to minimize error
                .try_into()
                .expect("rounded amount is always integral");
            // We've assigned this weight, so subtract it from the remaining total.
            total_weight -= weight;
            // We've assigned this integral allocation, so subtract it from the remaining total.
            total_allocation -= integral_allocation;
            // Return the key and its allocation.
            if integral_allocation != 0 {
                Some((key, integral_allocation))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .into()
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn total_and_scale_to_u128_is_exact(
            weights in proptest::collection::vec((0..u8::MAX, 0..u128::MAX), 0..(u8::MAX as usize))
        ) {
            let (total_weight, scaled_weights) = total_and_scale_to_u128(weights.clone());

            // The total weight is the sum of the scaled weights (implicit in this is that the sum
            // of scaled weights doesn't overflow, which will panic the test).
            let actual_total_weight: u128 = scaled_weights.iter().map(|(_, weight)| *weight).sum();
            prop_assert_eq!(total_weight, actual_total_weight, "total weight is not exact");
        }

        #[test]
        fn exact_allocation_is_exact(
            total_allocation in 0u128..u128::MAX,
            weights in proptest::collection::vec((0..u8::MAX, 0..u128::MAX), 0..(u8::MAX as usize))
        ) {
            let total_weight_is_zero = weights.iter().all(|(_, weight)| *weight == 0);
            let allocation = exact_allocation(total_allocation, weights.clone());

            // If an allocation was returned, it is exact, and the total weight must have been
            // nonzero; otherwise, the total weight must have been zero.
            if let Some(allocation) = allocation {
                // If an allocation was returned, then the total allocation is exactly the requested amount.
                let actual_total_allocation: u128 = allocation.iter().map(|(_, allocation)| *allocation).sum();
                prop_assert_eq!(total_allocation, actual_total_allocation, "total allocation is not exact");
                // And the total weight was not zero.
                prop_assert!(!total_weight_is_zero, "total weight is zero when allocation returned");
            } else {
                // Otherwise, the total weight was zero.
                prop_assert!(total_weight_is_zero, "total weight is not zero when no allocation returned");
            }
        }
    }

    #[test]
    fn exact_allocation_simple() {
        let alloc = exact_allocation::<&str>;
        assert_eq!(None, alloc(1, vec![]), "can't allocate something to nobody");
        assert_eq!(
            None,
            alloc(1, vec![("a", 0)]),
            "can't allocate something to zero weights"
        );
        assert_eq!(
            Some(vec![]),
            alloc(0, vec![]),
            "can allocate nothing to nobody"
        );
        assert_eq!(
            Some(vec![]),
            alloc(0, vec![("a", 1)]),
            "can allocate nothing to somebody"
        );
        assert_eq!(
            Some(vec![]),
            alloc(0, vec![("a", 0)]),
            "can allocate nothing to zero weights"
        );
        assert_eq!(
            Some(vec![("a", 1)]),
            alloc(1, vec![("a", 1)]),
            "can allocate the whole pot to one person"
        );
        assert_eq!(
            Some(vec![("a", 1)]),
            alloc(1, vec![("a", 2)]),
            "doubling the weight doesn't change the allocation"
        );
        assert_eq!(
            Some(vec![("a", 1), ("b", 1)]),
            alloc(2, vec![("a", 1), ("b", 1)]),
            "can allocate the whole pot to two people exactly evenly"
        );
        assert_eq!(
            Some(vec![("a", 1), ("b", 1)]),
            alloc(2, vec![("a", 2), ("b", 2)]),
            "doubling the weight doesn't change the allocation for two people"
        );
        assert_eq!(
            Some(vec![("a", 1), ("b", 1)]),
            alloc(2, vec![("a", 1), ("b", 2)]),
            "allocating two units to two people with different weights"
        );
        assert_eq!(
            Some(vec![("a", 1), ("b", 1)]),
            alloc(2, vec![("a", 2), ("b", 1)]),
            "allocating two units to two people with different weights, reverse order"
        );
        assert_eq!(
            Some(vec![("a", 1), ("b", 1)]),
            alloc(2, vec![("a", 1), ("b", 1), ("c", 1)]),
            "can't allocate 2 units 3 people exactly evenly, so pick the first two"
        );
        assert_eq!(
            Some(vec![("a", 1), ("c", 1)]),
            alloc(2, vec![("a", 1), ("b", 1), ("c", 2)]),
            "can't allocate 2 units 3 people exactly evenly, so pick the first low-weight and the first high-weight"
        );
        assert_eq!(
            Some(vec![("b", 2), ("c", 1)]),
            alloc(3, vec![("a", 1), ("b", 3), ("c", 2)]),
            "allocating 3 units to 3 people with different weights"
        );
        assert_eq!(
            Some(vec![("a", 2), ("b", 1)]),
            alloc(3, vec![("a", u128::MAX), ("b", u128::MAX / 2)]),
            "can allocate exactly even when the total weight is greater than u128::MAX"
        );
    }
}
