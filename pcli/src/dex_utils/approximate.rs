#![allow(non_snake_case)]
/// The acceptable amount of difference between a value and its approximation.
const APPROXIMATION_TOLERANCE: f64 = 1e-8;

pub mod xyk {
    use crate::dex_utils::approximate::utils;
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
    const MYSTERIOUS_SCALING_FACTOR: u64 = 1_000_000;

    pub(crate) fn sample_points(middle: f64, num_points: usize) -> Vec<f64> {
        let step = middle / (num_points as f64 / 2.0);
        let start = middle - (num_points as f64 / 2.0 - 1.0) * step;

        (0..num_points).map(|i| start + (i as f64) * step).collect()
    }

    pub fn approximate(
        pair: &DirectedUnitPair,
        r1: &Value,
        current_price: U128x128,
        fee_bps: u32,
    ) -> anyhow::Result<Vec<Position>> {
        // Henry: it could be interactive by default, --accept, --YES! that would
        // skip interactivity.

        // Note: we solve over the human-friendly display price and proceed with denom scaling
        // and cross-multiplication right before posting the positions. So here we need to rescale
        // the `r1` quantity.
        let fp_r1 = U128x128::from(r1.amount.value());
        let r1_scaling = U128x128::from(pair.start.unit_amount());
        let fp_r1 = U128x128::ratio(fp_r1, r1_scaling).unwrap();
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

        let alphas = sample_points(current_price.into(), NUM_POOLS_PRECISION);

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

        let scaling_factor = Amount::from(MYSTERIOUS_SCALING_FACTOR);
        let fp_scaling_factor: U128x128 = scaling_factor.into();

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
                    let scaled_p = (approx_p * fp_scaling_factor).unwrap();
                    let p: Amount = scaled_p
                        .round_down()
                        .try_into()
                        .expect("integral after truncating");

                    let unscaled_q = Amount::from(1u64);
                    let q = unscaled_q * scaling_factor;

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
                        "create a position with a tick below the current price"
                    );
                    tracing::debug!(
                        directed_pair = pair.to_string(),
                        r1 = field::display(r1),
                        r2 = field::display(r2),
                        ?p,
                        ?q,
                        "creating position"
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
                    let p = unscaled_p * scaling_factor;

                    let approx_q: U128x128 = alpha_i.try_into().unwrap();
                    let scaled_q = (approx_q * fp_scaling_factor).unwrap();
                    let q: Amount = scaled_q
                        .round_down()
                        .try_into()
                        .expect("integral after truncating");

                    let approx_r1: U128x128 = (*k_i * pair.start.unit_amount().value() as f64)
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
                        "create a position with a tick above the current price"
                    );
                    tracing::debug!(
                        directed_pair = pair.to_string(),
                        r1 = field::display(r1),
                        r2 = field::display(r2),
                        ?p,
                        ?q,
                        "creating position"
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

        utils::gauss_seidel(A, b, 1000, super::APPROXIMATION_TOLERANCE)
    }

    pub fn portfolio_value_function(invariant_k: f64, price: f64) -> f64 {
        2.0 * f64::sqrt(invariant_k * price)
    }
}

pub mod balancer {}

pub mod volatility {}

pub mod utils {
    use ndarray::s;
    use ndarray::Array;
    use ndarray::Array2;
    use penumbra_crypto::dex::lp::position::Position;
    use serde::Serialize;

    /// Applies the Gaus-Seidel method to a square matrix A and returns
    /// a vector of solutions.
    pub(crate) fn gauss_seidel(
        A: Array<f64, ndarray::Dim<[usize; 2]>>,
        b: Array<f64, ndarray::Dim<[usize; 1]>>,
        max_iterations: usize,
        tolerance: f64,
    ) -> anyhow::Result<Array<f64, ndarray::Dim<[usize; 1]>>> {
        let n = A.shape()[0];

        // First, we decompose the matrix into a lower triangular (L),
        // and an off-diagonal upper triangular matrix (D) st. A = L + D
        let L = lower_triangular(&A);
        let D = &A - &L;

        let mut k = Array::zeros(n);
        for _ in 0..max_iterations {
            let k_old = k.clone();

            for i in 0..n {
                let partial_off_diagonal_solution = D.slice(s![i, ..]).dot(&k);
                let partial_lower_triangular_solution = L.slice(s![i, ..i]).dot(&k.slice(s![..i]));
                let sum_ld = partial_off_diagonal_solution + partial_lower_triangular_solution;
                k[i] = (b[i] - sum_ld) / L[[i, i]];
            }

            let delta_approximation = &k - &k_old;
            let l2_norm_delta = delta_approximation
                .iter()
                .map(|&x| x * x)
                .sum::<f64>()
                .sqrt();

            if l2_norm_delta < tolerance {
                break;
            }
        }

        Ok(k)
    }

    /// Converts a square matrix into a lower triangular matrix.
    pub(crate) fn lower_triangular(matrix: &Array2<f64>) -> Array2<f64> {
        let (rows, cols) = matrix.dim();
        let mut result = Array2::zeros((rows, cols));

        for i in 0..rows {
            for j in 0..=i {
                result[[i, j]] = matrix[[i, j]];
            }
        }

        result
    }

    #[derive(Serialize)]
    pub struct PayoffPositionEntry {
        pub payoff: PayoffPosition,
        pub current_price: f64,
        pub index: usize,
        pub canonical_pair: String,
        // for debugging.
        pub alpha: f64,
        pub total_k: f64,
    }

    /// For debugging purposes. We want to be able to serialize a position
    /// to JSON so that we can pipe it into a Julia notebook. The reason why
    /// this is a separate structure from [`position::Position`] is that we
    /// might want to do extra processing, rounding, etc. and we'd rather note
    /// clutter it with serializiation methods that are useful for narrow purposes.
    #[derive(Serialize)]
    pub struct PayoffPosition {
        pub p: u128,
        pub q: u128,
        pub k: u128,
        pub r1: u128,
        pub r2: u128,
    }

    impl From<Position> for PayoffPosition {
        fn from(value: Position) -> Self {
            let p = value.phi.component.p.value();
            let q = value.phi.component.q.value();
            let r1 = value.reserves.r1.value();
            let r2 = value.reserves.r2.value();
            let k = p * r1 + q * r2;
            Self { p, q, k, r1, r2 }
        }
    }
}
