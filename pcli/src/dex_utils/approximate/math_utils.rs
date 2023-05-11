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
