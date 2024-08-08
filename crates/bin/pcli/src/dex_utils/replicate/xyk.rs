use crate::dex_utils::replicate::math_utils;
use anyhow::Context;
use ndarray::Array;
use penumbra_asset::Value;
use penumbra_dex::{
    lp::{position::Position, Reserves},
    DirectedUnitPair,
};
use penumbra_num::{fixpoint::U128x128, Amount};
use rand_core::OsRng;
use tracing::field;

/// The number of positions that is used to replicate the xyk CFMM.
pub(crate) const NUM_POOLS_PRECISION: usize = 30;

/// Maximum number of iteration that we allow GS to perform.
const GAUS_SEIDEL_MAX_ITERATION: usize = 10_000;

/// Sample a range of points around a given price
pub fn sample_prices(current_price: f64, num_points: usize) -> Vec<f64> {
    crate::dex_utils::replicate::math_utils::sample_to_upper(3.0 * current_price, num_points)
}

#[tracing::instrument(name = "replicate_xyk")]
pub fn replicate(
    pair: &DirectedUnitPair,
    raw_r1: &Value,
    current_price: U128x128,
    fee_bps: u32,
) -> anyhow::Result<Vec<Position>> {
    // First, we find the pool invariant using human display units. This means that we
    // only need to care about scaling into proper denom units right before posting the
    // positions. On the other hand, we have to unscale the values that we are given.
    let fp_raw_r1 = U128x128::from(raw_r1.amount.value());
    let r1_scaling_factor = U128x128::from(pair.start.unit_amount());

    let fp_r1 = (fp_raw_r1 / r1_scaling_factor).context("scaling factor can't be 0")?;
    let fp_r2 = (current_price * fp_r1).context("should not overflow when multiplying by price")?;

    tracing::debug!(
        %fp_r1,
        %fp_r2,
        "computed respective quantities"
    );

    let xyk_invariant = (fp_r1 * fp_r2).expect("no overflow when computing curve invariant!");

    let xyk_invariant: f64 = xyk_invariant.try_into()?;
    tracing::debug!(?xyk_invariant, "computed the total invariant for the PVF");

    let f64_current_price: f64 = current_price.try_into()?;

    let alphas = sample_prices(f64_current_price, NUM_POOLS_PRECISION);

    alphas
        .iter()
        .enumerate()
        .for_each(|(i, alpha)| tracing::debug!(i, alpha, "sampled tick"));

    // TODO(erwan): unused for now, but next refactor will rip out `solve` internals to
    // take this vector of solutions as an argument so that we can more easily recover from
    // working with non-singular matrices etc.
    let _b: Vec<f64> = alphas
        .iter()
        .map(|price: &f64| portfolio_value_function(xyk_invariant, *price))
        .collect();

    let position_ks = solve(&alphas, xyk_invariant, NUM_POOLS_PRECISION)?.to_vec();

    position_ks
        .iter()
        .enumerate()
        .for_each(|(i, pool_invariant)| tracing::debug!(i, pool_invariant, "found solution"));

    let unit_start = pair.start.unit_amount();
    let unit_end: U128x128 = pair.end.unit_amount().into();

    let positions: Vec<Position> = position_ks
        .iter()
        .enumerate()
        .zip(alphas)
        .map(|((i, k_i), alpha_i)| {
            tracing::debug!(i, f64_current_price, k_i, alpha_i, "constructing pool");

            // Case 1: \alpha_i < current_price
            // Populating ticks that are below the current price, the intuition
            // is that the positions accumulates the less valuable asset so as
            // the price trends to \alpha_i, we must provision inventories of
            // `asset_2`.
            // \phi(R) = alpha_i * (R_1 = 0)  + 1 * (R_2 = k_i * alpha_i) = k_i * alpha_i
            // Case 2: \alpha_i >= current_price
            // Tick is above the current price, therefore we want
            // to create a one-sided position with price `alpha_i`
            // that provisions `asset_1`.
            // \phi(R) = alpha_i * (R_1 = k_i) + 1 * (R_2 = 0) = alpha_i * k_i
            let approx_p: U128x128 = alpha_i
                .try_into()
                .expect("able to convert alpha_i to U128x128");
            let scaled_p = (approx_p * unit_end).expect("no overflow when scaling p");
            let p: Amount = scaled_p
                .round_down()
                .try_into()
                .expect("integral after truncating");

            let unscaled_q = Amount::from(1u64);
            let q = unscaled_q * unit_start;

            if alpha_i < f64_current_price {
                let r1: Amount = Amount::from(0u64);
                let approx_r2: U128x128 = (*k_i * pair.end.unit_amount().value() as f64 * alpha_i)
                    .try_into()
                    .expect("able to convert k_i * alpha_i to U128x128");
                let r2: Amount = approx_r2
                    .round_down()
                    .try_into()
                    .expect("integral after truncating");

                tracing::debug!(
                    i,
                    k_i,
                    alpha_i,
                    f64_current_price,
                    directed_pair = pair.to_string(),
                    r1 = field::display(r1),
                    r2 = field::display(r2),
                    ?p,
                    ?q,
                    "creating position with a tick below the current price"
                );

                Position::new(
                    OsRng,
                    pair.into_directed_trading_pair(),
                    fee_bps,
                    p,
                    q,
                    Reserves { r1, r2 },
                )
            } else {
                let approx_r1: U128x128 = (*k_i * pair.start.unit_amount().value() as f64)
                    .try_into()
                    .expect("able to convert k_i * alpha_i to U128x128");
                let r1: Amount = approx_r1
                    .round_down()
                    .try_into()
                    .expect("integral after truncating");
                let r2: Amount = Amount::from(0u64);

                tracing::debug!(
                    i,
                    k_i,
                    alpha_i,
                    f64_current_price,
                    directed_pair = pair.to_string(),
                    %r1,
                    %r2,
                    ?p,
                    ?q,
                    "creating position with a tick above the current price"
                );

                Position::new(
                    OsRng,
                    pair.into_directed_trading_pair(),
                    fee_bps,
                    p,
                    q,
                    Reserves { r1, r2 },
                )
            }
        })
        .collect();
    Ok(positions)
}

#[tracing::instrument(name = "replicate_clipped_xyk")]
/// Similar to [`ConstantProduct::replicate`], but "clips" the tail positions
/// creating a replicated portfolio with more uniform liquidity.
pub fn replicate_clipped(
    pair: &DirectedUnitPair,
    raw_r1: &Value,
    current_price: U128x128,
    fee_bps: u32,
) -> anyhow::Result<Vec<Position>> {
    // Generate the naive, "full-range", replicating portfolio of the constant-product curve.
    let mut positions = replicate(&pair, raw_r1, current_price, fee_bps)?;

    // Note on approach:
    // We have generated `NUM_PRECISION_POOLS` positions. The naive approximation
    // covers the full price range, which means that tail positions will emebed a
    // lot of inventory that will likely stay inert.
    // To improve the usefulness of the approximation, we can clip the tail LPs that
    // account for [0, \alpha_1] and [\alpha_n, +inf) and redistribute their reserves
    // uniformly (weighting can be tuned).

    // First, we pick the tail LPs:
    let lower_inf = positions.remove(0);
    let upper_inf = positions.pop().unwrap();

    // We want to track how many LPs provide for each asset respectively
    // so that we can distribute reserves transparently (user only provides
    // R1 of asset 1, and `R1*p` of asset2.
    let mut num_lp_r1 = 0usize;
    let mut num_lp_r2 = 0usize;

    for lp in positions.iter() {
        if lp.reserves_1().amount != Amount::zero() {
            num_lp_r1 += 1;
        } else {
            num_lp_r2 += 1;
        }
    }

    // We have the number of LPs for each asset, now we compute the amount
    // of bonus reserves remaining positions will get to cover for the clipped LPs:
    let chunk_r1 = if lower_inf.reserves_1().amount != Amount::zero() {
        lower_inf.reserves_1().amount.value() / num_lp_r1 as u128
    } else {
        upper_inf.reserves_1().amount.value() / num_lp_r1 as u128
    };

    let chunk_r2 = if upper_inf.reserves_2().amount != Amount::zero() {
        upper_inf.reserves_2().amount.value() / num_lp_r2 as u128
    } else {
        lower_inf.reserves_2().amount.value() / num_lp_r2 as u128
    };

    // Finally, we add the redistributed reserve to each LP:
    let positions = positions
        .iter_mut()
        .map(|lp| {
            if lp.reserves_1().amount != Amount::zero() {
                lp.reserves.r1 += chunk_r1.into();
            } else {
                lp.reserves.r2 += chunk_r2.into();
            }
            lp.to_owned()
        })
        .collect::<Vec<_>>();
    Ok(positions)
}

pub fn solve(
    alpha: &[f64],
    k: f64,
    n: usize,
) -> anyhow::Result<Array<f64, ndarray::Dim<[usize; 1]>>> {
    let mut A = Array::zeros((n, n));
    let mut b = Array::zeros(n);

    for j in 0..n {
        b[j] = portfolio_value_function(k, alpha[j]);

        for i in 0..j {
            A[[j, i]] = alpha[i];
        }
        for i in j..n {
            A[[j, i]] = alpha[j];
        }
    }

    math_utils::gauss_seidel(
        A,
        b,
        GAUS_SEIDEL_MAX_ITERATION,
        super::APPROXIMATION_TOLERANCE,
    )
}

pub fn portfolio_value_function(invariant_k: f64, price: f64) -> f64 {
    2.0 * f64::sqrt(invariant_k * price)
}
