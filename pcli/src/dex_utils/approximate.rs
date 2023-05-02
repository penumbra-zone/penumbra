use penumbra_crypto::{dex::lp::position::Position, fixpoint::U128x128, Value};

pub fn approximate_xyk(_invariant_k: &Value, _current_price: U128x128) -> Vec<Position> {
    let a = vec![2.0, 3.0, 5.0, 7.0, 9.0, 11.0, 13.0, 15.0];
    let k = 2.0;
    let n = a.len();

    match solve(&a, k, n) {
        Ok(k_values) => println!("k_1, ..., k_n: {:?}", k_values),
        Err(err) => eprintln!("{}", err),
    }
    vec![]
}

use ndarray::Array;
use ndarray_linalg::Solve;

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
