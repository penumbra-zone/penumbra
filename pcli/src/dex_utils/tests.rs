mod tests {
    use crate::dex_utils::approximate::xyk;
    const PRECISION_BOUND: f64 = 0.0001;

    fn approx_eq(a: f64, b: f64) -> bool {
        let ab_abs_diff = f64::abs(a - b);
        ab_abs_diff <= PRECISION_BOUND
    }

    #[test]
    /// Tests that the approximation work with a known solution obtained using LAPACK.
    fn test_xyk_solver() -> anyhow::Result<()> {
        let alphas = vec![2.0, 3.0, 5.0, 7.0, 9.0, 11.0, 13.0, 15.0];
        let k = 2.0;
        let n = alphas.len();

        let solution_vector = xyk::solve(&alphas, k, n)?;
        let known_solution = [
            1.1010205144336442,
            0.18619156818115412,
            0.13340819077963983,
            0.07839642626021837,
            0.05320822764119848,
            0.0391713189347902,
            0.030397692310478064,
            0.3782060614588767,
        ];

        assert_eq!(solution_vector.len(), known_solution.len());
        solution_vector
            .iter()
            .zip(known_solution)
            .for_each(|(solution_found, solution_known)| {
                assert!(approx_eq(*solution_found, solution_known))
            });

        Ok(())
    }
}
