use crate::dex_utils::approximate::math_utils;
use ndarray::Array;
use penumbra_crypto::{
    dex::{
        lp::{position::Position, Reserves},
        DirectedUnitPair,
    },
    fixpoint::U128x128,
    Amount, Value,
};
use rand_core::OsRng;
use tracing::field;

/// The number of positions that is used to approximate the xyk CFMM.
pub(crate) const NUM_POOLS_PRECISION: usize = 10;

/// Experimental scaling factor for spot valuations
const PRICE_SCALING_FACTOR: u64 = 1_000_000;

/// Maximum number of iteration that we allow GS to perform.
const GAUS_SEIDEL_MAX_ITERATION: usize = 10_000;

pub(crate) fn sample_full_range(middle: f64, num_points: usize) -> Vec<f64> {
    let step = 3.0 * middle / (num_points as f64);

    (1..=num_points).map(|i| (i as f64) * step).collect()
}

/// A function that returns a vector of length `num_points` covering the range 0 to `upper`,
/// with equally spaced points.
pub fn sample_to_upper(upper: f64, num_points: usize) -> Vec<f64> {
    let step = upper / (num_points as f64);

    (1..=num_points).map(|i| (i as f64) * step).collect()
}

#[allow(dead_code)]
pub(crate) fn sample_points(middle: f64, num_points: usize) -> Vec<f64> {
    let step = middle / (num_points as f64 / 2.0);
    let start = middle - (num_points as f64 / 2.0 - 1.0) * step;

    (0..num_points).map(|i| start + (i as f64) * step).collect()
}

#[tracing::instrument(name = "approximate_xyk")]
pub fn approximate(
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

    let fp_r1 = (fp_raw_r1 / r1_scaling_factor).unwrap();
    let fp_r2 = (current_price * fp_r1).ok_or_else(|| {
        anyhow::anyhow!(
            "current_price: {} * fp_r1: {} caused overflow.",
            current_price,
            fp_r1
        )
    })?;

    tracing::debug!(
        r1 = field::display(fp_r1),
        r2 = field::display(fp_r2),
        "computed respective quantities"
    );

    let xyk_invariant = (fp_r1 * fp_r2).ok_or_else(|| {
        anyhow::anyhow!("overflow computing the curve invariant: {fp_r1} * {fp_r2}")
    })?;

    let xyk_invariant: f64 = xyk_invariant.try_into()?;
    tracing::debug!(?xyk_invariant, "computed the total invariant for the PVF");

    let alphas = sample_full_range(current_price.into(), NUM_POOLS_PRECISION);

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

    let f64_current_price: f64 = current_price.try_into()?;

    let price_scaling_factor = Amount::from(PRICE_SCALING_FACTOR);
    let fp_price_scaling_factor: U128x128 = price_scaling_factor.into();

    let positions: Vec<Position> = position_ks
        .iter()
        .enumerate()
        .zip(alphas)
        .map(|((i, k_i), alpha_i)| {
            tracing::debug!(i, f64_current_price, k_i, alpha_i, "constructing pool");

            // Populating ticks that are below the current price, the intuition
            // is that the positions accumulates the less valuable asset so as
            // the price trends to \alpha_i, we must provision inventories of
            // `asset_2`.
            if alpha_i < f64_current_price {
                let approx_p: U128x128 = alpha_i.try_into().unwrap();
                let scaled_p = (approx_p * fp_price_scaling_factor).unwrap();
                let p: Amount = scaled_p
                    .round_down()
                    .try_into()
                    .expect("integral after truncating");

                let unscaled_q = Amount::from(1u64);
                let q = unscaled_q * price_scaling_factor;

                let r1: Amount = Amount::from(0u64);
                let approx_r2: U128x128 = (*k_i * pair.end.unit_amount().value() as f64)
                    .try_into()
                    .unwrap();
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
                // Tick is above the current price, therefore we want
                // to create a one-sided position with price `alpha_i`
                // that provisions `asset_1`.
                let unscaled_p = Amount::from(1u64);
                let mut p = unscaled_p * price_scaling_factor;

                let approx_q: U128x128 = alpha_i.try_into().unwrap();
                let scaled_q = (approx_q * fp_price_scaling_factor).unwrap();
                let mut q: Amount = scaled_q
                    .round_down()
                    .try_into()
                    .expect("integral after truncating");

                std::mem::swap(&mut p, &mut q);

                let approx_r1: U128x128 = (*k_i * pair.start.unit_amount().value() as f64
                    / alpha_i)
                    .try_into()
                    .unwrap();
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
                    r1 = field::display(r1),
                    r2 = field::display(r2),
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
