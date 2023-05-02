use ndarray::Array;
use ndarray::Array1;
use ndarray::Array2;
use ndarray_linalg::Solve;
use penumbra_crypto::{dex::lp::position::Position, fixpoint::U128x128, Value};

pub fn approximate_xyk(_invariant_k: &Value, _current_price: U128x128) -> Vec<Position> {
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

    match subst_fwd(a.clone(), k, n) {
        Ok(k_values) => println!("k_values: {k_values:?}"),
        Err(e) => panic!("{}", e),
    }
    vec![]
}

fn solve(
    alpha: &[f64],
    k: f64,
    n: usize,
) -> Result<Array<f64, ndarray::Dim<[usize; 1]>>, ndarray_linalg::error::LinalgError> {
    let mut A = Array::zeros((n, n));
    let mut b = Array::zeros(n);

    for j in 0..n {
        b[j] = 2.0 * f64::sqrt(k * alpha[j]);

        for i in 0..j {
            A[[j, i]] = alpha[i];
        }
        for i in j..n {
            A[[j, i]] = alpha[j];
        }
    }

    A.solve_into(b)
}

fn subst_fwd(alpha: Vec<f64>, k: f64, n: usize) -> anyhow::Result<Vec<f64>> {
    let n = alpha.len();

    let mut A = Array::zeros((n, n));
    let mut b = Array::zeros(n);

    for j in 0..n {
        b[j] = 2.0 * f64::sqrt(k * alpha[j]);

        for i in 0..j {
            A[[j, i]] = alpha[i];
        }
        for i in j..n {
            A[[j, i]] = alpha[j];
        }
    }

    let mut k = vec![0.0; n];

    for i in 0..n {
        let mut sum = 0.0;
        for j in 0..i {
            sum += A[[i, j]] * k[j];
        }
        k[i] = (b[i] - sum) / A[[i, i]];
    }

    Ok(k)
}
