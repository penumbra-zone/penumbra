//! This module is very similar to the one for phase1, so reading that one might be useful.
use anyhow::{Result, anyhow};
use ark_ec::Group;
use ark_ff::{fields::Field, UniformRand, Zero};
use ark_poly::domain::{EvaluationDomain, Radix2EvaluationDomain};
use ark_relations::r1cs::ConstraintMatrices;
use rand_core::{CryptoRngCore, OsRng};

use crate::log::{ContributionHash, Hashable, Phase};
use crate::phase1;
use crate::{
    dlog,
    group::{BatchedPairingChecker11, GroupHasher, F, G1, G2},
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
}

impl Hashable for RawCRSElements {
    /// Hash these elements, producing a succinct digest.
    fn hash(&self) -> ContributionHash {
        let mut hasher = GroupHasher::new(b"PC$:crs_elmnts2");
        hasher.eat_g1(&self.delta_1);
        hasher.eat_g2(&self.delta_2);

        hasher.eat_usize(self.inv_delta_p_1.len());
        for v in &self.inv_delta_p_1 {
            hasher.eat_g1(v);
        }

        hasher.eat_usize(self.inv_delta_t_1.len());
        for v in &self.inv_delta_t_1 {
            hasher.eat_g1(v);
        }

        ContributionHash(hasher.finalize_bytes())
    }
}

/// The CRS elements we produce in phase 2.
///
/// When combined with the elements of phase 1, the entire CRS will be present.
#[derive(Clone, Debug)]
pub struct CRSElements {
    pub(crate) raw: RawCRSElements,
}

impl Hashable for CRSElements {
    fn hash(&self) -> ContributionHash {
        self.raw.hash()
    }
}

impl CRSElements {
    pub fn transition(
        phase1: phase1::CRSElements,
        circuit: &ConstraintMatrices<F>,
    ) -> Result<Self> {
        // To understand this function, it can be good to recap the relationship between
        // R1CS constraints and QAP constraints.
        //
        // While we call the constaints a "circuit", they're really an R1CS system.
        // The circuit contains matrices A, B, C. Each of these matrices has the same size.
        // The number of columns is the number of variables in our circuit,
        // along with an additional column for each constraint, representing an
        // "internal variable".
        // The number of rows is simply the number of constraints in our circuit.
        //
        // To transform the circuit into a QAP, each column of a matrix becomes a polynomial.
        // For the matrix A, we have uᵢ(X), for B, vᵢ(X), and C, wᵢ(X).
        // We also have a domain, a list of field elements, such that evaluting each polynomial
        // on the domain produces the entries of the corresponding matrix column.
        //
        // Furthermore, we also pad the matrices before this transformation.
        // The bottom of A is filled with:
        //   1 0 0 ...
        //   0 1 0 ...
        //   0 0 1 ...
        //   .........
        // based on the number of instance variables. This is to avoid
        // potential malleability issues. See: https://geometry.xyz/notebook/groth16-malleability.
        // B and C are simply padded with 0 to match this size.
        //
        // Finally, the domain might be larger than the size of these matrices,
        // and so we simply add rows filled with 0 to match the size of the domain.
        //
        // Now, we don't need to calculate these polynomials directly, instead what we care about
        // is the polynomials pᵢ(X) = α uᵢ(X) + β vᵢ(X) + wᵢ(X), evaluated at [x],
        // and the evaluations [t(x)xⁱ], where t is a polynomial that's 0 everywhere over the
        // domain.
        //
        // Our strategy here is to make use of lagrange polynomials Lⱼ(X), which
        // are 0 everywhere over the domain except for the jth entry.
        // Specifically, if we have the evaluations Lⱼ([x]), we can then calculate
        // each pᵢ([x]) by a linear combination over these evaluations,
        // using the coefficients in that matrix column.
        // Note that the matrices are very sparse, so there are only a linear
        // number of such operations to do, instead of a quadratic number.
        //
        // For calculating the lagrange polynomials, we want to avoid a quadratic
        // solution. To do so, we need to exploit the structure of the domain
        // we use: specifically, the fact that we can do fast fourier transforms (FFTs)
        // in it. (See section 3 of https://eprint.iacr.org/2017/602 for the origin
        // of most of this exposition).
        //
        // Our domain consists of successive powers of a root of unity: 1, ω, ω², ...,
        // such that ωᵈ = 1. The number of entries in the domain is d, in total.
        // This is useful, because we can quickly convert a polynomial p(X), represented
        // as a table of coefficients, into an evaluation table p(ωⁱ) (i ∈ [d]).
        //
        // We can also use this for evaluating the lagrange polynomials.
        // First, note the following equality:
        //  Lⱼ(X) = d⁻¹∑ᵢ (Xω⁻ʲ)ⁱ
        // (because the sum over all roots of unity is 0).
        // Defining:
        //  Pₓ(Y) := d⁻¹∑ᵢ xⁱ Yⁱ
        // we can write Lⱼ(x) = Lⱼ(x) = Pₓ(ω⁻ʲ).
        //
        // What we want is [Lⱼ(x)] for each j. With this equality, we see that we
        // can get this by doing an FFT over Pₓ(Y). This will also work even if
        // the coefficients are group elements, and not scalars.
        // (We also need to reverse the order of the resulting FFT).
        //
        // The structure of the domain also helps us with the vanishing polynomial, t.
        // The polynomial (Xᵈ − 1) is 0 everywhere over the domain, and give us
        // a simple expression for t(X). The evaluations [t(x)xⁱ] can then be obtained
        // by using simple indexing.
        //
        // Ok, that was the explanation, now onto the code.
        let circuit_size = circuit.num_constraints + circuit.num_instance_variables;
        let domain: Radix2EvaluationDomain<F> =
            Radix2EvaluationDomain::new(circuit_size).ok_or(anyhow!(
                "Failed to create evaluation domain size (at least) {}",
                circuit_size
            ))?;
        let domain_size = domain.size();
        // 0. Check that the phase1 degree is large enough.
        if phase1.degree < domain_size {
            return Err(anyhow!(
                "Phase1 elements not large enough: expected >= {}, found {}",
                domain_size,
                phase1.degree
            ));
        }
        let d_inv = domain.size_inv();

        // 1. Get the lagrange coefficients over [x].
        // 1.1. Setup a polynomial that's 1/d * x^i at each coefficient.
        let mut extracting_poly: Vec<_> = phase1
            .raw
            .x_1
            .iter()
            .copied()
            .take(domain_size)
            .map(|x| x * d_inv)
            .collect();
        domain.fft_in_place(&mut extracting_poly);
        extracting_poly.reverse();
        let lagrange = extracting_poly;

        // 2. Do the same for [αx].
        let mut extracting_poly: Vec<_> = phase1
            .raw
            .alpha_x_1
            .iter()
            .copied()
            .take(domain_size)
            .map(|x| x * d_inv)
            .collect();
        domain.fft_in_place(&mut extracting_poly);
        extracting_poly.reverse();
        let alpha_lagrange = extracting_poly;

        // 3. Do the same for [βx].
        let mut extracting_poly: Vec<_> = phase1
            .raw
            .beta_x_1
            .iter()
            .copied()
            .take(domain_size)
            .map(|x| x * d_inv)
            .collect();
        domain.fft_in_place(&mut extracting_poly);
        extracting_poly.reverse();
        let beta_lagrange = extracting_poly;
        // 4. Accumulate the p_i polynomials evaluated over [x].
        // This indexing is copied from ark_groth16/r1cs_to_qap.rs.html#106.
        // (I spent a full massage chair cycle thinking about this and couldn't figure out
        // why exactly they do it this way, but mirroring the code we're trying to be
        // compatible with is a good idea).
        let qap_num_variables =
            (circuit.num_instance_variables - 1) + circuit.num_witness_variables;
        let mut p = vec![G1::zero(); qap_num_variables + 1];

        // This is where we add the identity matrix block at the end to avoid malleability
        // shenanigans.
        {
            let start = 0;
            let end = circuit.num_instance_variables;
            let num_constraints = circuit.num_constraints;
            // One deviation if you're reading the arkworks code is that we're modifying
            // the entire p polynomial, and not u (which they call 'a'), but this effectively does
            // the same thing, because the other polynomials are set to 0 at these points.
            p[start..end].copy_from_slice(
                &alpha_lagrange[(start + num_constraints)..(end + num_constraints)],
            );
        }

        // Could zip here, but this looks cleaner to me.
        for i in 0..circuit.num_constraints {
            for &(ref coeff, j) in &circuit.a[i] {
                p[j] += alpha_lagrange[i] * coeff;
            }
            for &(ref coeff, j) in &circuit.b[i] {
                p[j] += beta_lagrange[i] * coeff;
            }
            for &(ref coeff, j) in &circuit.c[i] {
                p[j] += lagrange[i] * coeff;
            }
        }

        // 5. Calculate the t polynomial, evaluated multiplied by successive powers.
        let t: Vec<_> = (0..(domain_size - 2))
            .map(|i| phase1.raw.x_1[i + domain_size] - phase1.raw.x_1[i])
            .collect();

        Ok(Self {
            raw: RawCRSElements {
                delta_1: G1::generator(),
                delta_2: G2::generator(),
                inv_delta_p_1: p,
                inv_delta_t_1: t,
            },
        })
    }
}

/// Represents a raw, unvalidatedontribution.
#[derive(Clone, Debug)]
pub struct RawContribution {
    pub parent: ContributionHash,
    pub new_elements: RawCRSElements,
    linking_proof: dlog::Proof,
}

impl RawContribution {
    /// Check the internal integrity of this contribution, potentially producing
    /// a valid one.
    fn validate<R: CryptoRngCore>(self, rng: &mut R, root: &CRSElements) -> Option<Contribution> {
        self.new_elements
            .validate(rng, root)
            .map(|new_elements| Contribution {
                parent: self.parent,
                new_elements,
                linking_proof: self.linking_proof,
            })
    }
}

impl Hashable for RawContribution {
    fn hash(&self) -> ContributionHash {
        let mut hasher = GroupHasher::new(b"PC$:contrbution2");
        hasher.eat_bytes(self.parent.as_ref());
        hasher.eat_bytes(self.new_elements.hash().as_ref());
        hasher.eat_bytes(self.linking_proof.hash().as_ref());

        ContributionHash(hasher.finalize_bytes())
    }
}

impl From<Contribution> for RawContribution {
    fn from(value: Contribution) -> Self {
        Self {
            parent: value.parent,
            new_elements: value.new_elements.raw,
            linking_proof: value.linking_proof,
        }
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

impl Hashable for Contribution {
    fn hash(&self) -> ContributionHash {
        RawContribution::from(self.to_owned()).hash()
    }
}

impl Contribution {
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

/// A dummy struct to implement the phase trait.
#[derive(Clone, Debug, Default)]
struct Phase2;

impl Phase for Phase2 {
    type CRSElements = CRSElements;

    type RawContribution = RawContribution;

    type Contribution = Contribution;

    fn parent_hash(contribution: &Self::RawContribution) -> ContributionHash {
        contribution.parent
    }

    fn elements(contribution: &Self::Contribution) -> &Self::CRSElements {
        &contribution.new_elements
    }

    fn validate(
        root: &Self::CRSElements,
        contribution: &Self::RawContribution,
    ) -> Option<Self::Contribution> {
        contribution.to_owned().validate(&mut OsRng, root)
    }

    fn is_linked_to(contribution: &Self::Contribution, elements: &Self::CRSElements) -> bool {
        contribution.is_linked_to(elements)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::log::CONTRIBUTION_HASH_SIZE;

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
