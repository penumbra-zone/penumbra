use ark_ec::Group;
use ark_ff::Zero;

use crate::dlog;
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
#[derive(Clone, Debug)]
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
        let mut i = 0;
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

/// A linking proof shows knowledge of the new secret elements linking two sets of CRS elements.
///
/// This pets two cats with one hand:
/// 1. We show that we're actually building off of the previous elements.
/// 2. We show that we know the secret elements we're using, avoiding rogue key chicanery.
#[derive(Clone, Copy, Debug)]
struct LinkingProof {
    alpha_proof: dlog::Proof,
    beta_proof: dlog::Proof,
    x_proof: dlog::Proof,
}

/// The max
pub const CONTRIBUTION_HASH_SIZE: usize = 32;

// Note: Don't need constant time equality because we're hashing public data: contributions.

/// The hash of a contribution, providing a unique string for each contribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContributionHash([u8; CONTRIBUTION_HASH_SIZE]);

/// Represents a contribution to phase1 of the ceremony.
///
/// This contribution is linked to a previous contribution, which it builds upon.
///
/// The contribution includes new elements for the CRS, along with a proof that these elements
/// build upon the claimed parent contribution.
#[derive(Clone, Debug)]
pub struct Contribution {
    pub parent: ContributionHash,
    pub new_elements: CRSElements,
    linking_proof: LinkingProof,
}

#[cfg(test)]
mod test {
    use super::*;

    use ark_ff::UniformRand;
    use rand_core::OsRng;

    use crate::group::F;

    /// The degree we use for tests.
    ///
    /// Keeping this small makes tests go faster.
    const D: usize = 2;

    fn make_crs(alpha: F, beta: F, x: F) -> CRSElements {
        CRSElements {
            alpha_1: G1::generator() * alpha,
            beta_1: G1::generator() * beta,
            beta_2: G2::generator() * beta,
            x_1: vec![
                G1::generator(),
                G1::generator() * x,
                G1::generator() * (x * x),
                G1::generator() * (x * x * x),
            ],
            x_2: vec![G2::generator(), G2::generator() * x],
            alpha_x_1: vec![G1::generator() * alpha, G1::generator() * (alpha * x)],
            beta_x_1: vec![G1::generator() * beta, G1::generator() * (beta * x)],
        }
    }

    fn non_trivial_crs() -> CRSElements {
        let alpha = F::rand(&mut OsRng);
        let beta = F::rand(&mut OsRng);
        let x = F::rand(&mut OsRng);

        make_crs(alpha, beta, x)
    }

    #[test]
    fn test_root_crs_is_valid() {
        let root = CRSElements::root(D);
        assert!(root.is_valid());
    }

    #[test]
    fn test_nontrivial_crs_is_valid() {
        let crs = non_trivial_crs();
        assert!(crs.is_valid());
    }

    #[test]
    fn test_changing_alpha_makes_crs_invalid() {
        let mut crs = non_trivial_crs();
        crs.alpha_1 = G1::generator();
        assert!(!crs.is_valid());
    }

    #[test]
    fn test_changing_beta_makes_crs_invalid() {
        let mut crs = non_trivial_crs();
        crs.beta_1 = G1::generator();
        assert!(!crs.is_valid());
    }

    #[test]
    fn test_setting_zero_elements_makes_crs_invalid() {
        let alpha = F::rand(&mut OsRng);
        let beta = F::rand(&mut OsRng);
        let x = F::rand(&mut OsRng);

        let crs0 = make_crs(F::zero(), beta, x);
        assert!(!crs0.is_valid());
        let crs1 = make_crs(alpha, F::zero(), x);
        assert!(!crs1.is_valid());
        let crs2 = make_crs(alpha, beta, F::zero());
        assert!(!crs2.is_valid());
    }

    #[test]
    fn test_bad_powers_of_x_makes_crs_invalid() {
        let alpha = F::rand(&mut OsRng);
        let beta = F::rand(&mut OsRng);
        let x = F::rand(&mut OsRng);
        let crs = CRSElements {
            alpha_1: G1::generator() * alpha,
            beta_1: G1::generator() * beta,
            beta_2: G2::generator() * beta,
            x_1: vec![
                G1::generator(),
                G1::generator() * x,
                G1::generator() * (x * x),
                // The important part
                G1::generator() * (x * x),
            ],
            x_2: vec![G2::generator(), G2::generator() * x],
            alpha_x_1: vec![G1::generator() * alpha, G1::generator() * (alpha * x)],
            beta_x_1: vec![G1::generator() * beta, G1::generator() * (beta * x)],
        };
        assert!(!crs.is_valid());
    }
}
