#![allow(non_snake_case)]
/// The acceptable amount of difference between a value and its approximation.
const APPROXIMATION_TOLERANCE: f64 = 1e-8;

pub mod xyk {
    use crate::dex_utils::approximate::utils;
    use ndarray::Array;
    use penumbra_crypto::{
        dex::{
            lp::{position::Position, Reserves},
            Market,
        },
        fixpoint::U128x128,
        Amount, Value,
    };
    use rand_core::OsRng;

    /// The number of positions that is used to approximate the xyk CFMM.
    const NUM_POOLS_PRECISION: usize = 100;

   pub(crate) fn sample_points(middle: f64, num_points: usize) -> Vec<f64> {
        let start = middle - ((num_points as f64 - 1.0) / 2.0);
        let step = 2.0 * ((num_points as f64 - 1.0) / 2.0) / (num_points - 1) as f64;
        (0..num_points)
            .map(|i| start + i as f64 * step)
            .collect()
    }
    
    pub fn approximate(
        market: &Market,
        r1: &Value,
        current_price: U128x128,
    ) -> anyhow::Result<Vec<Position>> {
        let fp_r1 = U128x128::from(r1.amount.value());
        let fp_r2 = (current_price * fp_r1).ok_or_else(|| {
            anyhow::anyhow!(
                "current_price: {} * fp_r1: {} caused overflow.",
                current_price,
                fp_r1
            )
        })?;

        let xyk_invariant = (fp_r1 * fp_r2).ok_or_else(|| {
            anyhow::anyhow!("overflow computing the curve invariant: {fp_r1} * {fp_r2}")
        })?;

        let xyk_invariant: f64 = xyk_invariant.try_into()?;

        let alphas = sample_points(current_price.into(), NUM_POOLS_PRECISION);

        // TODO(erwan): unused for now, but next refactor will rip out `solve` internals to
        // take this vector of solutions as an argument so that we can more easily recover from
        // working with non-singular matrices etc.
        let _b: Vec<f64> = alphas
            .iter()
            .map(|price: &f64| portfolio_value_function(xyk_invariant, *price))
            .collect();

        let position_ks = solve(&alphas, xyk_invariant, NUM_POOLS_PRECISION)?.to_vec();

        // TODO(erwan): we need an argument that will output structured position data
        // that we can pipe into a julia notebook and check if the curve makes sense.
        position_ks
            .iter()
            .enumerate()
            .for_each(|(i, k)| println!("k_{i} = {k}"));

        let f64_current_price: f64 = current_price.try_into()?;

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
                    let approx_p: U128x128 = (alpha_i * market.end.unit_amount().value() as f64)
                        .try_into()
                        .unwrap();
                    let p: Amount = approx_p
                        .round_down()
                        .try_into()
                        .expect("integral after truncating");
                    let q = Amount::from(1u64) * market.start.unit_amount();

                    let r1: Amount = Amount::from(0u64);
                    let approx_r2: U128x128 = (*k_i * market.start.unit_amount().value() as f64)
                        .try_into()
                        .unwrap();
                    let r2: Amount = approx_r2
                        .round_down()
                        .try_into()
                        .expect("integral after truncating");

                    Position::new(
                        OsRng,
                        market.into_directed_trading_pair(),
                        0u32,
                        p,
                        q,
                        Reserves { r1, r2 },
                    )
                } else {
                    // Tick is above the current price, therefore we want
                    // to create a one-sided position with price `alpha_i`
                    // that provisions `asset_1`.
                    let p = Amount::from(1u64) * market.end.unit_amount();
                    let approx_q: U128x128 = (alpha_i * market.start.unit_amount().value() as f64)
                        .try_into()
                        .unwrap();
                    let q: Amount = approx_q
                        .round_down()
                        .try_into()
                        .expect("integral after truncating");

                    let approx_r1: U128x128 = (*k_i * market.start.unit_amount().value() as f64)
                        .try_into()
                        .unwrap();
                    let r1: Amount = approx_r1
                        .round_down()
                        .try_into()
                        .expect("integral after truncating");
                    let r2: Amount = Amount::from(0u64);

                    Position::new(
                        OsRng,
                        market.into_directed_trading_pair(),
                        0u32,
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
        let L = lower_triangular(&A);
        let D = &A - &L;

        let mut k = Array::zeros(n);
        for _ in 0..max_iterations {
            let k_old = k.clone();

            for i in 0..n {
                // This looks more gnarly than it actually is, TODO(erwan): link to the spec.
                // The goal here is to take advantage of the fact that L is lower triangular,
                // it's essentially doing forward subsitution. TODO(erwan): write performance argument.
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
