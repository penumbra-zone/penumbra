use crate::fixpoint::U128x128;

/// Compute an exact integer allocation of `total_allocation` units, proportioned according to the
/// given `weights`, minimizing the average error ratio per key.
///
/// The maximum error in the allocation for any key is 1 unit; that is, a perfect fractional
/// allocation would differ for each key by no more than 1.
///
/// This function is guaranteed to return an allocation unless it is impossible to do so: the only
/// corner case where this is impossible is when the total allocation is non-zero and the sum of the
/// weights is zero. Any time outside this condition, an allocation is returned, and it is
/// guaranteed to be exact: the sum of the allocations returned will be exactly the requested total
/// allocation.
///
/// The allocation returned is guaranteed to be deterministic regardless of the initial order of the
/// weights: ties are broken by preferring keys with an earlier ordering. If multiple equal keys are
/// present, they are treated as separate and their weights are not combined, with preference being
/// given to the key which appears earliest in the list of weights.
///
/// The ordering of the keys in the returned allocation will be lexicographic by (weight, key), even
/// if the allocations given to a particular two keys end up being equal. This means that you
/// *cannot* rely on the ordering placing earlier-ordered keys first, even if their allocations are
/// the same. If you want to ensure a particular ordering, you must sort the keys yourself afterwards.
pub fn allocate<K: Ord>(
    mut total_allocation: u128,
    mut weights: Vec<(K, u128)>,
) -> Option<Vec<(K, u128)>> {
    // If the total allocation is zero, then every key will be allocated zero, so we can forget all
    // the weights without doing any more processing of them.
    if total_allocation == 0 {
        weights.clear();
    }

    // Scale the weights down to fit in a `u128`.
    let mut total_weight = scale_to_u128(&mut weights);

    // If the total weight is zero, then we can't allocate anything, which would violate the
    // guarantee to allocate exactly if we returned any result.
    if total_weight == 0 && total_allocation != 0 {
        return None;
    }

    // Sorting the keys by ascending weight minimizes the *percentage* error in the allocations,
    // because as the total remaining allocation decreases, the amount of error in a rounding
    // division increases, but since we are ascending the weights, we're pushing higher error
    // (which, notably, is capped at 1 unit of allocation, maximum!) to the most-weighted keys,
    // which means that the average *percentage* error is minimized, since the total allocation to
    // the most-weighted keys is the highest, and therefore the absolute error matters least to
    // them.
    weights.sort_by(|(key_a, weight_a), (key_b, weight_b)| {
        // Sort by weight, then by key to break ties.
        weight_a.cmp(weight_b).then(key_a.cmp(key_b))
    });

    // For each key in the weights, calculate the allocation for that key, sequentially iterating
    // from least-weighted to most-weighted.
    //
    // If two keys are equally-weighted, then it could happen that one key gets 1 unit of allocation
    // more than the other: this is deterministic based on comparing the keys, since above we sort
    // the weights in ascending lexicographic order of (weight, key).
    //
    // This computation is done in-place on the weights vector, so that we can avoid allocating a
    // new vector at any point during the whole procedure.
    //
    // This approach is loosely based off https://stackoverflow.com/a/38905829.
    weights.retain_mut(move |(_, value)| {
        // The weight is the current value at the key; we're going to replace it in-place with the
        // allocation.
        let weight = *value;
        // The allocation for this key is the total allocation times the fraction of the
        // total weight that this key has, rounded to the nearest integer.
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
        // Replace the value with its allocation.
        *value = integral_allocation;
        // Only keep this key if it was allocated something.
        integral_allocation != 0
    });
    Some(weights)
}

/// Given an association of keys to weights, sum the weights, scaling down the weights uniformly to
/// make sure that the total weight fits in a `u128`, returning the total and modifying the weights
/// in-place.
///
/// This is used internally in [`exact_allocation`].
pub fn scale_to_u128<K>(weights: &mut Vec<(K, u128)>) -> u128 {
    // Calculate the total weight, tracking overflows so we can compute a scaling factor in the case
    // when the total of weights exceeds `u128::MAX`. This is computing the sum as a `u256` in two
    // limbs of `u128`: hi and lo.
    let mut lo: u128 = 0;
    let mut hi: u128 = 0;
    for (_, weight) in weights.iter() {
        if let Some(new_lo) = lo.checked_add(*weight) {
            lo = new_lo;
        } else {
            // If lo overflows, track the overflow in hi.
            hi += 1;
            // Explicitly wrapping-add the weight to lo, so that we can continue without losing the remainder.
            lo = lo.wrapping_add(*weight);
        };
    }

    // Compute a scaling factor such that the total weight is scaled down to fit in a `u128` if this
    // scaling factor is applied to each weight.
    let scaling_factor = if hi == 0 {
        // If there were no overflows, then the scaling factor is 1. This special case is desirable
        // so that we get *zero* precision loss for weights that fit in a `u128`, rather than
        // round-down loss from dividing by a computed scaling factor.
        U128x128::from(1u8)
    } else {
        // If there were overflows, then the scaling factor is (hi . lo) as a U128x128. This is done
        // so that if the total weight exceeds `u128::MAX`, we scale down the weights to fit within
        // that bound: i.e., the hi limb of the total weight is the integral part of the scaling
        // factor, since it represents by how many times we have exceeded `u128::MAX`.
        U128x128::from_parts(hi, lo)
    };

    // Compute a new set of weights and total weight by applying the scaling factor to the weights.
    // Even if there was overflow, the new total weight may be less than `u128::MAX`, since loss of
    // precision when dividing individual weights may have reduced the total weight. This is done
    // in-place using `Vec::retain` to avoid allocating a new vector.
    let mut total_scaled_weight: u128 = 0;
    weights.retain_mut(|(_, weight)| {
        // Scale each weight down by dividing it by the scaling factor and rounding down.
        *weight = (U128x128::from(*weight) / scaling_factor)
            .expect("scaling factor is never zero")
            .round_down() // must round *down* to avoid total exceeding `u128::MAX` in all situations
            .try_into()
            .expect("rounded amount is always integral");
        // Track the total scaled weight, so we can return it.
        total_scaled_weight += *weight;
        // Only output the scaled weight if it is greater than zero, since we don't want to do extra
        // work for weights that are dropped by scaling.
        *weight != 0
    });

    total_scaled_weight
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn total_and_scale_to_u128_is_exact(
            mut weights in proptest::collection::vec((0..u8::MAX, 0..u128::MAX), 0..(u8::MAX as usize))
        ) {
            let total_weight = scale_to_u128(&mut weights);

            // The total weight is the sum of the scaled weights (implicit in this is that the sum
            // of scaled weights doesn't overflow, which will panic the test).
            let actual_total_weight: u128 = weights.iter().map(|(_, weight)| *weight).sum();
            prop_assert_eq!(total_weight, actual_total_weight, "total weight is not exact");
        }

        #[test]
        fn exact_allocation_is_exact(
            total_allocation in 0u128..u128::MAX,
            weights in proptest::collection::vec((0..u8::MAX, 0..u128::MAX), 0..(u8::MAX as usize))
        ) {
            let total_weight_is_zero = weights.iter().all(|(_, weight)| *weight == 0);
            let allocation = allocate(total_allocation, weights.clone());

            // If an allocation was returned, it is exact, and the total weight must have been
            // nonzero; otherwise, the total weight must have been zero.
            if let Some(allocation) = allocation {
                // If an allocation was returned, then the total allocation is exactly the requested amount.
                let actual_total_allocation: u128 = allocation.iter().map(|(_, allocation)| allocation).sum();
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
        fn alloc<const N: usize>(n: u128, ws: [(&str, u128); N]) -> Option<Vec<(&str, u128)>> {
            allocate(n, ws.to_vec())
        }

        assert_eq!(None, alloc(1, []), "can't allocate something to nobody");
        assert_eq!(
            None,
            alloc(1, [("a", 0)]),
            "can't allocate something to zero weights"
        );
        assert_eq!(Some(vec![]), alloc(0, []), "can allocate nothing to nobody");
        assert_eq!(
            Some(vec![]),
            alloc(0, [("a", 1)]),
            "can allocate nothing to somebody"
        );
        assert_eq!(
            Some(vec![]),
            alloc(0, [("a", 0)]),
            "can allocate nothing to zero weights"
        );
        assert_eq!(
            Some(vec![("a", 1)]),
            alloc(1, [("a", 1)]),
            "can allocate the whole pot to one person"
        );
        assert_eq!(
            Some(vec![("a", 1)]),
            alloc(1, [("a", 2)]),
            "doubling the weight doesn't change the allocation"
        );
        assert_eq!(
            Some(vec![("a", 1), ("b", 1)]),
            alloc(2, [("a", 1), ("b", 1)]),
            "can allocate the whole pot to two people exactly evenly"
        );
        assert_eq!(
            Some(vec![("a", 1), ("b", 1)]),
            alloc(2, [("a", 2), ("b", 2)]),
            "doubling the weight doesn't change the allocation for two people"
        );
        assert_eq!(
            Some(vec![("a", 1), ("b", 1)]),
            alloc(2, [("a", 1), ("b", 2)]),
            "allocating two units to two people with different weights"
        );
        assert_eq!(
            Some(vec![("b", 1), ("a", 1)]),
            alloc(2, [("a", 2), ("b", 1)]),
            "allocating two units to two people with different weights, reverse order"
        );
        assert_eq!(
            Some(vec![("a", 1), ("b", 1)]),
            alloc(2, [("a", 1), ("b", 1), ("c", 1)]),
            "can't allocate 2 units 3 people exactly evenly, so pick the first two"
        );
        assert_eq!(
            Some(vec![("a", 1),/*       */("c", 1)]),
            alloc(2, [("a", 1), ("b", 1), ("c", 2)]),
            "can't allocate 2 units 3 people exactly evenly, so pick the first low-weight and the first high-weight"
        );
        assert_eq!(
            Some(vec![/*     */ ("c", 1), ("b", 2)]),
            alloc(3, [("a", 1), ("b", 3), ("c", 2)]),
            "allocating 3 units to 3 people with different weights"
        );
        assert_eq!(
            Some(vec![("b", 1), /*   */ ("a", 2)]),
            alloc(3, [("a", u128::MAX), ("b", u128::MAX / 2)]),
            "can allocate exactly even when the total weight is greater than u128::MAX"
        );
    }
}
