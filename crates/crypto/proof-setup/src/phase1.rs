use ark_ec::Group;
use ark_ff::Zero;

use crate::group::{pairing, G1, G2};

/// Assert that a given degree is high enough.
///
/// (We use this enough times to warrant separating it out).
const fn assert_degree_large_enough(d: usize) {
    // We need to have at least the index 1 for our x_i values, so we need d >= 2.
    assert!(d >= 2)
}

/// The CRS elements we produce in phase 1.
///
/// Not all elements of the final CRS are present here.
#[derive(Clone)]
pub struct CRSElements {
    pub alpha_1: G1,
    pub beta_1: G1,
    pub beta_2: G2,
    pub x_1: Vec<G1>,
    pub x_2: Vec<G2>,
    pub alpha_x_1: Vec<G1>,
    pub beta_x_1: Vec<G1>,
}

impl CRSElements {
    /// Generate a "root" CRS, containing the value 1 for each secret element.
    ///
    /// This takes in the degree "d" associated with the circuit we need
    /// to do a setup for, as per the docs.
    ///
    /// Naturally, these elements shouldn't actually be used as-is, but this
    /// serves as a logical basis for the start of the phase.
    pub fn root(d: usize) -> Self {
        assert_degree_large_enough(d);

        Self {
            alpha_1: G1::generator(),
            beta_1: G1::generator(),
            beta_2: G2::generator(),
            x_1: vec![G1::generator(); (2 * d - 2) + 1],
            x_2: vec![G2::generator(); (d - 1) + 1],
            alpha_x_1: vec![G1::generator(); (d - 1) + 1],
            beta_x_1: vec![G1::generator(); (d - 1) + 1],
        }
    }

    /// Check whether or not these elements are internally consistent.
    ///
    /// This checks if the structure of the elements uses the secret scalars
    /// hidden behind the group elements correctly.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        // 1. Check that the elements committing to the secret values are not 0.
        if self.alpha_1.is_zero()
            || self.beta_1.is_zero()
            || self.beta_2.is_zero()
            || self.x_1[1].is_zero()
            || self.x_2[1].is_zero()
        {
            return false;
        }
        // 2. Check that the two beta commitments match.
        if pairing(self.beta_1, G2::generator()) != pairing(G1::generator(), self.beta_2) {
            return false;
        }
        // 3. Check that the x values match on both groups.
        // Todo: use a batched pairing check for this
        if !self
            .x_1
            .iter()
            .zip(self.x_2.iter())
            .all(|(l, r)| pairing(l, G2::generator()) == pairing(G1::generator(), r))
        {
            return false;
        }
        // 4. Check that alpha and x are connected in alpha_x.
        if !self
            .x_2
            .iter()
            .zip(self.alpha_x_1.iter())
            .all(|(x_i, alpha_x_i)| {
                pairing(self.alpha_1, x_i) == pairing(alpha_x_i, G2::generator())
            })
        {
            return false;
        }
        // 5. Check that beta and x are connected in beta_x.
        if !self
            .x_2
            .iter()
            .zip(self.beta_x_1.iter())
            .all(|(x_i, beta_x_i)| pairing(self.beta_1, x_i) == pairing(beta_x_i, G2::generator()))
        {
            return false;
        }
        // 6. Check that the x_i are the correct powers of x.
        if !self
            .x_1
            .iter()
            .zip(self.x_1.iter().skip(1))
            .all(|(x_i, x_i_plus_1)| {
                pairing(x_i, self.x_2[1]) == pairing(x_i_plus_1, G2::generator())
            })
        {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// The degree we use for tests.
    ///
    /// Keeping this small makes tests go faster.
    const D: usize = 2;

    #[test]
    fn test_phase1_root_crs_is_valid() {
        let root = CRSElements::root(D);
        assert!(root.is_valid());
    }
}
