#![allow(non_snake_case)]
use ndarray::s;
use ndarray::Array;
use ndarray::Array2;

pub mod xyk {
    use ndarray::Array;
    use penumbra_crypto::{
        dex::{lp::position::Position, Market},
        fixpoint::U128x128,
        Value,
    };

    /// The number of positions that is used to approximate the xyk CFMM.
    const NUM_POOLS_PRECISION: usize = 100;

    pub fn approximate(
        market: &Market,
        invariant_k: &Value,
        _current_price: U128x128,
    ) -> anyhow::Result<Vec<Position>> {
        let alphas = super::sample_points(NUM_POOLS_PRECISION);
        let global_invariant = invariant_k.amount.value() as f64;

        let b: Vec<f64> = alphas
            .iter()
            .map(|price: &f64| portfolio_value_function(global_invariant, *price))
            .collect();

        let k_invariants = solve(
            &alphas,
            invariant_k.amount.value() as f64,
            NUM_POOLS_PRECISION,
        )?
        .to_vec();

        println!("k_invariants: {k_invariants:?}");

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

        super::gauss_seidel(A, b, 1000, super::APPROXIMATION_TOLERANCE)
    }

    pub fn portfolio_value_function(invariant_k: f64, price: f64) -> f64 {
        2.0 * f64::sqrt(invariant_k * price)
    }
}

pub mod balancer {}

pub mod volatility {}

/// The acceptable amount of difference between a value and its approximation.
const APPROXIMATION_TOLERANCE: f64 = 1e-8;

/// Applies the Gaus-Seidel method to a square matrix A and returns
/// a vector of solutions.
fn gauss_seidel(
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
fn lower_triangular(matrix: &Array2<f64>) -> Array2<f64> {
    let (rows, cols) = matrix.dim();
    let mut result = Array2::zeros((rows, cols));

    for i in 0..rows {
        for j in 0..=i {
            result[[i, j]] = matrix[[i, j]];
        }
    }

    result
}

fn sample_points(n: usize) -> Vec<f64> {
    let mut points = vec![];
    for i in 1..=n {
        points.push(i as f64)
    }

    points
}
