use ndarray::s;
use ndarray::Array;
use ndarray::Array2;

pub mod xyk {
    use ndarray::Array;
    use penumbra_crypto::{dex::lp::position::Position, fixpoint::U128x128, Value};
    pub fn approximate(_invariant_k: &Value, _current_price: U128x128) -> Vec<Position> {
        let a = vec![2.0, 3.0, 5.0, 7.0, 9.0, 11.0, 13.0, 15.0];
        let k = 2.0;
        let n = a.len();

        let b: Vec<f64> = a
            .iter()
            .map(|alpha: &f64| {
                let inner: f64 = k * *alpha;
                let inner = inner.sqrt();
                2.0 * inner
            })
            .collect();

        match solve(&a, k, n) {
            Ok(k_values) => println!("k_1, ..., k_n: {:?}", k_values),
            Err(err) => eprintln!("{}", err),
        }

        vec![]
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

        super::gauss_seidel(A, b, 1000, 1e-8)
    }

    pub fn portfolio_value_function(invariant_k: f64, price: f64) -> f64 {
        2.0 * f64::sqrt(invariant_k * price)
    }
}

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

fn gauss_seidel(
    A: Array<f64, ndarray::Dim<[usize; 2]>>,
    b: Array<f64, ndarray::Dim<[usize; 1]>>,
    max_iterations: usize,
    _tolerance: f64,
) -> anyhow::Result<Array<f64, ndarray::Dim<[usize; 1]>>> {
    let n = A.shape()[0];
    let L = lower_triangular(&A);
    let D = &A - &L;

    let mut k = Array::zeros(n);
    for _ in 0..max_iterations {
        // See TODO about implementing tolerance checks.
        // let k_old = k.clone();

        for i in 0..n {
            // This looks more gnarly than it actually is, TODO(erwan): link to the spec.
            // The goal here is to take advantage of the fact that L is lower triangular,
            // it's essentially doing forward subsitution. TODO(erwan): write performance argument.
            let partial_off_diagonal_solution = D.slice(s![i, ..]).dot(&k);
            let partial_lower_triangular_solution = L.slice(s![i, ..i]).dot(&k.slice(s![..i]));
            let sum_ld = partial_off_diagonal_solution + partial_lower_triangular_solution;
            k[i] = (b[i] - sum_ld) / L[[i, i]];
        }

        // TODO(erwan): unfortunately, we cannot use the L2 Norm helper
        // from `ndarray-linalg` without adding a dependency to LAPACK.
        // I'm not going to implement it manually, even though it can be
        // done because it's relatively lower priority. However, it would
        // be nice to stop G-S early if we get below a tolerance level.
        // Mostly for performance purposes.
        // if (&k - &k_old).norm_l2() < tol {
        //     break;
        // }
    }

    Ok(k)
}
