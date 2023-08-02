//! This module is very similar to the one for phase1, so reading that one might be useful.
use std::hash;

use ark_ec::Group;
use ark_ff::{fields::Field, UniformRand, Zero};
use rand_core::CryptoRngCore;

use crate::{
    dlog,
    group::{BatchedPairingChecker11, GroupHasher, Hash, F, G1, G2},
};

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

    /// Hash these elements, producing a succinct digest.
    fn hash(&self) -> Hash {
        let mut hasher = GroupHasher::new(b"PC$:crs_elmnts2");
        hasher.eat_g1(&self.delta_1);
        hasher.eat_g2(&self.delta_2);

        hasher.eat_usize(self.inv_delta_p_1.len());
        for v in &self.inv_delta_p_1 {
            hasher.eat_g1(&v);
        }

        hasher.eat_usize(self.inv_delta_t_1.len());
        for v in &self.inv_delta_t_1 {
            hasher.eat_g1(&v);
        }

        hasher.finalize_bytes()
    }
}

/// The CRS elements we produce in phase 2.
///
/// When combined with the elements of phase 1, the entire CRS will be present.
#[derive(Clone, Debug)]
pub struct CRSElements {
    raw: RawCRSElements,
}

impl CRSElements {
    fn hash(&self) -> Hash {
        self.raw.hash()
    }
}

pub const CONTRIBUTION_HASH_SIZE: usize = 32;

// Note: Don't need constant time equality because we're hashing public data: contributions.

/// The hash of a contribution, providing a unique string for each contribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, hash::Hash)]
pub struct ContributionHash(pub [u8; CONTRIBUTION_HASH_SIZE]);

impl AsRef<[u8]> for ContributionHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Represents a contribution to phase2 of the ceremony.
///
/// This contribution is linked to the previous contribution it builds upon.
///
/// The contribution contains new CRS elements, and a proof linking these elements
/// to those of the parent contribution.
#[derive(Clone, Debug)]
pub struct Contribution {
    pub parent: ContributionHash,
    pub new_elements: CRSElements,
    linking_proof: dlog::Proof,
}

impl Contribution {
    /// Hash this contribution, producing a unique digest.
    pub fn hash(&self) -> ContributionHash {
        let mut hasher = GroupHasher::new(b"PC$:contrbution2");
        hasher.eat_bytes(self.parent.as_ref());
        hasher.eat_bytes(self.new_elements.hash().as_ref());
        hasher.eat_bytes(self.linking_proof.hash().as_ref());

        ContributionHash(hasher.finalize_bytes())
    }

    /// Make a new contribution, over the previous CRS elements.
    ///
    /// We also need a hash of the parent contribution we're building on.
    pub fn make<R: CryptoRngCore>(
        rng: &mut R,
        parent: ContributionHash,
        old: &CRSElements,
    ) -> Self {
        let delta = F::rand(rng);
        // e.w. negligible probability this will not panic
        let delta_inv = delta.inverse().unwrap();

        let mut new = old.clone();
        new.raw.delta_1 *= delta;
        new.raw.delta_2 *= delta;
        for v in &mut new.raw.inv_delta_p_1 {
            *v *= delta_inv;
        }
        for v in &mut new.raw.inv_delta_t_1 {
            *v *= delta_inv;
        }

        let linking_proof = dlog::prove(
            rng,
            b"phase2 delta proof",
            dlog::Statement {
                result: new.raw.delta_1,
                base: old.raw.delta_1,
            },
            dlog::Witness { dlog: delta },
        );

        Contribution {
            parent,
            new_elements: new,
            linking_proof,
        }
    }

    /// Verify that this contribution is linked to a previous list of elements.
    #[must_use]
    pub fn is_linked_to(&self, parent: &CRSElements) -> bool {
        // 1. Check that the sizes match between the two elements.
        if self.new_elements.raw.inv_delta_p_1.len() != parent.raw.inv_delta_p_1.len()
            || self.new_elements.raw.inv_delta_t_1.len() != parent.raw.inv_delta_t_1.len()
        {
            return false;
        }
        // 2. Check that the linking proof verifies
        if !dlog::verify(
            b"phase2 delta proof",
            dlog::Statement {
                result: self.new_elements.raw.delta_1,
                base: parent.raw.delta_1,
            },
            &self.linking_proof,
        ) {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use rand_core::OsRng;

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

    #[test]
    fn test_contribution_produces_valid_crs() {
        let (root, start) = non_trivial_crs();
        let start = start.validate(&mut OsRng, &root).unwrap();
        let contribution = Contribution::make(
            &mut OsRng,
            ContributionHash([0u8; CONTRIBUTION_HASH_SIZE]),
            &start,
        );
        assert!(contribution
            .new_elements
            .raw
            .validate(&mut OsRng, &root)
            .is_some());
    }

    #[test]
    fn test_can_calculate_contribution_hash() {
        let (root, start) = non_trivial_crs();
        let start = start.validate(&mut OsRng, &root).unwrap();
        let contribution = Contribution::make(
            &mut OsRng,
            ContributionHash([0u8; CONTRIBUTION_HASH_SIZE]),
            &start,
        );
        assert_ne!(contribution.hash(), contribution.parent);
    }

    #[test]
    fn test_contribution_is_linked_to_parent() {
        let (root, start) = non_trivial_crs();
        let start = start.validate(&mut OsRng, &root).unwrap();
        let contribution = Contribution::make(
            &mut OsRng,
            ContributionHash([0u8; CONTRIBUTION_HASH_SIZE]),
            &start,
        );
        assert!(contribution.is_linked_to(&start));
    }

    #[test]
    fn test_contribution_is_not_linked_to_itself() {
        let (root, start) = non_trivial_crs();
        let start = start.validate(&mut OsRng, &root).unwrap();
        let contribution = Contribution::make(
            &mut OsRng,
            ContributionHash([0u8; CONTRIBUTION_HASH_SIZE]),
            &start,
        );
        assert!(!contribution.is_linked_to(&contribution.new_elements));
    }
}
