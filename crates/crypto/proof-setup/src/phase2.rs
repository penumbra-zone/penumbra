use ark_ec::Group;
use ark_ff::Zero;
use rand_core::CryptoRngCore;

use crate::group::{BatchedPairingChecker11, G1, G2};

/// Raw CRS elements, not yet validated for consistency.
#[derive(Clone, Debug)]
pub struct RawCRSElements {
    pub delta_1: G1,
    pub delta_2: G2,
    pub inv_delta_p_1: Vec<G1>,
    pub inv_delta_t_1: Vec<G1>,
}

impl RawCRSElements {
    #[must_use]
    pub fn validate<R: CryptoRngCore>(
        self,
        rng: &mut R,
        root: &CRSElements,
    ) -> Option<CRSElements> {
        // 0. Check that the lengths match that of the root.
        if self.inv_delta_p_1.len() != root.raw.inv_delta_p_1.len()
            || self.inv_delta_t_1.len() != root.raw.inv_delta_t_1.len()
        {
            return None;
        }
        // 1. Check that the elements committing to secret values are not 0.
        if self.delta_1.is_zero() || self.delta_2.is_zero() {
            return None;
        }
        // 2. Check that the two delta commitments match.
        // 3. Check that 1/delta has multiplied the root polynomial p
        // 3. Check that 1/delta has multiplied the root polynomial t
        // We can use one batch check for all of these!
        let mut checker = BatchedPairingChecker11::new(self.delta_2, G2::generator());
        checker.add(G1::generator(), self.delta_1);
        for (&inv_delta_p_i, &p_i) in self.inv_delta_p_1.iter().zip(root.raw.inv_delta_p_1.iter()) {
            checker.add(inv_delta_p_i, p_i);
        }
        for (&inv_delta_t_i, &t_i) in self.inv_delta_t_1.iter().zip(root.raw.inv_delta_t_1.iter()) {
            checker.add(inv_delta_t_i, t_i);
        }
        if !checker.check(rng) {
            return None;
        }

        Some(CRSElements { raw: self })
    }
}

/// The CRS elements we produce in phase 2.
///
/// When combined with the elements of phase 1, the entire CRS will be present.
#[derive(Clone, Debug)]
pub struct CRSElements {
    raw: RawCRSElements,
}

#[cfg(test)]
mod test {
    use super::*;

    use ark_ff::{fields::Field, UniformRand};
    use rand_core::OsRng;

    use crate::group::F;

    fn make_crs(delta: F, delta_inv: F) -> (CRSElements, RawCRSElements) {
        let x = F::rand(&mut OsRng);

        let root = CRSElements {
            raw: RawCRSElements {
                delta_1: G1::generator(),
                delta_2: G2::generator(),
                inv_delta_p_1: vec![G1::generator() * x],
                inv_delta_t_1: vec![G1::generator() * (x * x)],
            },
        };

        let new = RawCRSElements {
            delta_1: root.raw.delta_1 * delta,
            delta_2: root.raw.delta_2 * delta,
            inv_delta_p_1: root
                .raw
                .inv_delta_p_1
                .iter()
                .map(|&x| x * delta_inv)
                .collect(),
            inv_delta_t_1: root
                .raw
                .inv_delta_t_1
                .iter()
                .map(|&x| x * delta_inv)
                .collect(),
        };

        (root, new)
    }

    fn non_trivial_crs() -> (CRSElements, RawCRSElements) {
        let delta = F::rand(&mut OsRng);
        // Won't panic e.w. negligible probability
        let delta_inv = delta.inverse().unwrap();

        make_crs(delta, delta_inv)
    }

    #[test]
    fn test_nontrivial_crs_is_valid() {
        let (root, crs) = non_trivial_crs();
        assert!(crs.validate(&mut OsRng, &root).is_some());
    }

    #[test]
    fn test_changing_delta_makes_crs_invalid() {
        let (root, mut crs) = non_trivial_crs();
        crs.delta_1 = G1::generator();
        crs.delta_2 = G2::generator();
        assert!(crs.validate(&mut OsRng, &root).is_none());
    }

    #[test]
    fn test_different_deltas_makes_crs_invalid() {
        let (root, mut crs) = non_trivial_crs();
        crs.delta_1 = G1::generator();
        assert!(crs.validate(&mut OsRng, &root).is_none());
    }

    #[test]
    fn test_different_length_from_root_is_invalid_crs() {
        let (root, mut crs) = non_trivial_crs();
        crs.inv_delta_p_1.clear();
        crs.inv_delta_t_1.clear();
        assert!(crs.validate(&mut OsRng, &root).is_none());
    }

    #[test]
    fn test_setting_zero_elements_makes_crs_invalid() {
        let (root, crs) = make_crs(F::zero(), F::zero());
        assert!(crs.validate(&mut OsRng, &root).is_none());
    }

    #[test]
    fn test_not_inverting_delta_makes_crs_invalid() {
        let delta = F::rand(&mut OsRng);
        let (root, crs) = make_crs(delta, delta);
        assert!(crs.validate(&mut OsRng, &root).is_none());
    }
}
