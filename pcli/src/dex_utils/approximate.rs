#![allow(non_snake_case)]
/// The acceptable amount of difference between a value and its approximation.
const APPROXIMATION_TOLERANCE: f64 = 1e-8;

pub mod xyk {
    use std::thread::current;

    use crate::dex_utils::approximate::utils;
    use ndarray::Array;
    use penumbra_crypto::{
        dex::{
            lp::{position::Position, Reserves},
            Market,
        },
        fixpoint::{self, U128x128},
        Amount, Value,
    };
    use rand_core::OsRng;

    /// The number of positions that is used to approximate the xyk CFMM.
    const NUM_POOLS_PRECISION: usize = 100;

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

        let xyk_invariant = fixpoint::to_f64_unsafe(&xyk_invariant);

        let alphas = utils::sample_points(NUM_POOLS_PRECISION);

        // TODO(erwan): unused for now, but next refactor will rip out `solve` internals to
        // take this vector of solutions as an argument so that we can more easily recover from
        // working with non-singular matrices etc.
        let _b: Vec<f64> = alphas
            .iter()
            .map(|price: &f64| portfolio_value_function(xyk_invariant, *price))
            .collect();

        let position_ks = solve(&alphas, xyk_invariant, NUM_POOLS_PRECISION)?.to_vec();

        // TODO(erwan): it would be nice to have an option to output structured input
        // so that a graph tool can pickup the solutions and they can be independently checked.
        position_ks
            .iter()
            .enumerate()
            .for_each(|(i, k)| println!("k_{i} = {k}"));

        let f64_current_price = fixpoint::to_f64_unsafe(&current_price);

        let _positions: Vec<Position> = position_ks
            .iter()
            .enumerate()
            .zip(alphas)
            .map(|((i, k_i), alpha_i)| {
                tracing::debug!(i, f64_current_price, k_i, alpha_i, "constructing pool");
                let mut p = fixpoint::from_f64_unsafe(alpha_i)
                    .round_down()
                    .try_into()
                    .expect("integral after truncating");
                let mut q = Amount::from(1u64) * market.start.unit_amount();

                let mut r1: Amount = fixpoint::from_f64_unsafe(k_i)
                    .round_down()
                    .try_into()
                    .expect("integral after truncating");

                let mut r2: Amount = Amount::from(0u64);

                if alpha_i < f64_current_price {
                    // Tick is below the current price, therefore we want
                    // to create a one-sided position with price `alpha_i`
                    // that provisions `asset_1`.
                } else {
                    // Tick is above the current price, so we want to create
                    // a one-sided position with price `alpha_i` that provisions
                    // `asset_2`.
                }
                Position::new(
                    OsRng,
                    market.into_directed_trading_pair(),
                    0u32,
                    p,
                    q,
                    Reserves { r1, r2 },
                )
            })
            .collect();
        Ok(vec![])
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

pub(crate) mod utils {
    use ndarray::s;
    use ndarray::Array;
    use ndarray::Array2;

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

    pub(crate) fn sample_points(n: usize) -> Vec<f64> {
        let mut points = vec![];
        for i in 1..=n {
            points.push(i as f64)
        }

        points
    }
}
