//! This module contains definition for a single circuit setup.
#![deny(clippy::unwrap_used)]
// Todo: prune public interface once we know exactly what's needed.
mod dlog;
pub(crate) mod group;
pub mod log;
mod phase1;
mod phase2;

use ark_ec::{CurveGroup, Group};
use ark_ff::Zero;
use ark_groth16::data_structures::ProvingKey;
use ark_groth16::VerifyingKey;
use ark_poly::EvaluationDomain;
use ark_poly::Radix2EvaluationDomain;
use ark_relations::r1cs::ConstraintMatrices;

use ark_serialize::CanonicalDeserialize;
use ark_serialize::CanonicalSerialize;
use decaf377::Bls12_377;

pub use dlog::Proof as DLogProof;

pub use phase1::CRSElements as Phase1CRSElements;
pub use phase1::Contribution as Phase1Contribution;
pub(crate) use phase1::LinkingProof;
pub use phase1::RawCRSElements as Phase1RawCRSElements;
pub use phase1::RawContribution as Phase1RawContribution;

pub use phase2::CRSElements as Phase2CRSElements;
pub use phase2::Contribution as Phase2Contribution;
pub use phase2::RawCRSElements as Phase2RawCRSElements;
pub use phase2::RawContribution as Phase2RawContribution;

use group::{F, G1, G2};

use anyhow::{anyhow, Result};

#[derive(Clone, Debug, CanonicalSerialize, CanonicalDeserialize)]
pub struct ExtraTransitionInformation {
    /// The u polynomials evaluated at [x].
    u_1: Vec<G1>,
    /// The v polynomials evaluted at [x].
    v_1: Vec<G1>,
    /// The v polynomials evaluted at [x]_2.
    v_2: Vec<G2>,
    /// The p polynomials evaluated at [x]
    p_1: Vec<G1>,
}

// Compute the degree associated with a given circuit.
//
// This degree can then be used for both phases.
pub fn circuit_degree(circuit: &ConstraintMatrices<F>) -> Result<usize> {
    let circuit_size = circuit.num_constraints + circuit.num_instance_variables;
    Radix2EvaluationDomain::<group::F>::compute_size_of_domain(circuit_size)
        .ok_or_else(|| anyhow!("Circuit of size {} is too large", circuit_size))
}

/// Transition between phase1 and phase2, using the circuit.
///
/// This will also produce extra elements needed later when combining the elements
/// into the actual proving key.
pub fn transition(
    phase1: &phase1::CRSElements,
    circuit: &ConstraintMatrices<F>,
) -> Result<(ExtraTransitionInformation, phase2::CRSElements)> {
    // To understand this function, it can be good to recap the relationship between
    // R1CS constraints and QAP constraints.
    //
    // While we call the constraints a "circuit", they're really an R1CS system.
    // The circuit contains matrices A, B, C. Each of these matrices has the same size.
    // The number of columns is the number of variables in our circuit,
    // along with an additional column for each constraint, representing an
    // "internal variable".
    // The number of rows is simply the number of constraints in our circuit.
    //
    // To transform the circuit into a QAP, each column of a matrix becomes a polynomial.
    // For the matrix A, we have uᵢ(X), for B, vᵢ(X), and C, wᵢ(X).
    // We also have a domain, a list of field elements, such that evaluating each polynomial
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
    // is the polynomials pᵢ(X) = β uᵢ(X) + α vᵢ(X) + wᵢ(X), evaluated at [x],
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
    // This is also equivalent to doing an inverse FFT, and not including
    // the inverse of d in the polynomial.
    //
    // The structure of the domain also helps us with the vanishing polynomial, t.
    // The polynomial (Xᵈ − 1) is 0 everywhere over the domain, and give us
    // a simple expression for t(X). The evaluations [t(x)xⁱ] can then be obtained
    // by using simple indexing.
    //
    // Ok, that was the explanation, now onto the code.
    let circuit_size = circuit.num_constraints + circuit.num_instance_variables;
    let domain: Radix2EvaluationDomain<F> =
        Radix2EvaluationDomain::new(circuit_size).ok_or_else(|| {
            anyhow!(
                "Failed to create evaluation domain size (at least) {}",
                circuit_size
            )
        })?;
    let domain_size = domain.size();
    // 0. Check that the phase1 degree is large enough.
    if phase1.degree < domain_size {
        return Err(anyhow!(
            "Phase1 elements not large enough: expected >= {}, found {}",
            domain_size,
            phase1.degree
        ));
    }

    // 1. Get the lagrange coefficients over [x].
    // 1.1. Setup a polynomial that's x^i at each coefficient.
    let mut extracting_poly: Vec<_> = phase1.raw.x_1.iter().copied().take(domain_size).collect();
    domain.ifft_in_place(&mut extracting_poly);
    let lagrange_1 = extracting_poly;

    // 2. Do the same for [x]_2.
    let mut extracting_poly: Vec<_> = phase1.raw.x_2.iter().copied().take(domain_size).collect();
    domain.ifft_in_place(&mut extracting_poly);
    let lagrange_2 = extracting_poly;

    // 3. Do the same for [αx].
    let mut extracting_poly: Vec<_> = phase1
        .raw
        .alpha_x_1
        .iter()
        .copied()
        .take(domain_size)
        .collect();
    domain.ifft_in_place(&mut extracting_poly);
    let alpha_lagrange_1 = extracting_poly;

    // 4. Do the same for [βx].
    let mut extracting_poly: Vec<_> = phase1
        .raw
        .beta_x_1
        .iter()
        .copied()
        .take(domain_size)
        .collect();
    domain.ifft_in_place(&mut extracting_poly);
    let beta_lagrange_1 = extracting_poly;

    // 5. Accumulate the p_i polynomials evaluated over [x].
    // This indexing is copied from ark_groth16/r1cs_to_qap.rs.html#106.
    // (I spent a full massage chair cycle thinking about this and couldn't figure out
    // why exactly they do it this way, but mirroring the code we're trying to be
    // compatible with is a good idea).
    let qap_num_variables = (circuit.num_instance_variables - 1) + circuit.num_witness_variables;
    let mut p_1 = vec![G1::zero(); qap_num_variables + 1];
    // Also take this opportunity to accumulate the raw u_1, v_1, and v_2 polynomials
    let mut u_1 = vec![G1::zero(); qap_num_variables + 1];
    let mut v_1 = vec![G1::zero(); qap_num_variables + 1];
    let mut v_2 = vec![G2::zero(); qap_num_variables + 1];

    // This is where we add the identity matrix block at the end to avoid malleability
    // shenanigans.
    {
        let start = 0;
        let end = circuit.num_instance_variables;
        let num_constraints = circuit.num_constraints;
        // One deviation if you're reading the arkworks code is that we're modifying
        // the entire p polynomial, and not u (which they call 'a'), but this effectively does
        // the same thing, because the other polynomials are set to 0 at these points.
        p_1[start..end]
            .copy_from_slice(&beta_lagrange_1[(start + num_constraints)..(end + num_constraints)]);
        // We also modify u in the same way
        u_1[start..end]
            .copy_from_slice(&lagrange_1[(start + num_constraints)..(end + num_constraints)])
    }

    // Could zip here, but this looks cleaner to me.
    for i in 0..circuit.num_constraints {
        for &(ref coeff, j) in &circuit.a[i] {
            p_1[j] += beta_lagrange_1[i] * coeff;
            u_1[j] += lagrange_1[i] * coeff;
        }
        for &(ref coeff, j) in &circuit.b[i] {
            p_1[j] += alpha_lagrange_1[i] * coeff;
            v_1[j] += lagrange_1[i] * coeff;
            v_2[j] += lagrange_2[i] * coeff;
        }
        for &(ref coeff, j) in &circuit.c[i] {
            p_1[j] += lagrange_1[i] * coeff;
        }
    }

    // 5. Calculate the t polynomial, evaluated multiplied by successive powers.
    let t: Vec<_> = (0..(domain_size - 1))
        .map(|i| phase1.raw.x_1[i + domain_size] - phase1.raw.x_1[i])
        .collect();

    Ok((
        ExtraTransitionInformation {
            u_1,
            v_1,
            v_2,
            p_1: p_1.clone(),
        },
        phase2::CRSElements {
            raw: phase2::RawCRSElements {
                delta_1: G1::generator(),
                delta_2: G2::generator(),
                inv_delta_p_1: p_1,
                inv_delta_t_1: t,
            },
        },
    ))
}

/// Combine the outputs from all phases into a proving key.
///
/// This proving key also contains a verifying key inherently.
pub fn combine(
    circuit: &ConstraintMatrices<F>,
    phase1out: &Phase1CRSElements,
    phase2out: &Phase2CRSElements,
    extra: &ExtraTransitionInformation,
) -> ProvingKey<Bls12_377> {
    let vk = VerifyingKey {
        alpha_g1: phase1out.raw.alpha_1.into_affine(),
        beta_g2: phase1out.raw.beta_2.into_affine(),
        gamma_g2: G2::generator().into_affine(),
        delta_g2: phase2out.raw.delta_2.into_affine(),
        gamma_abc_g1: G1::normalize_batch(&extra.p_1[..circuit.num_instance_variables]),
    };
    ProvingKey {
        vk,
        beta_g1: phase1out.raw.beta_1.into_affine(),
        delta_g1: phase2out.raw.delta_1.into_affine(),
        a_query: G1::normalize_batch(&extra.u_1),
        b_g1_query: G1::normalize_batch(&extra.v_1),
        b_g2_query: G2::normalize_batch(&extra.v_2),
        h_query: G1::normalize_batch(&phase2out.raw.inv_delta_t_1),
        l_query: G1::normalize_batch(
            &phase2out.raw.inv_delta_p_1[circuit.num_instance_variables..],
        ),
    }
}

#[cfg(test)]
mod test {
    use ark_ff::One;
    use ark_groth16::{r1cs_to_qap::LibsnarkReduction, Groth16};
    use ark_r1cs_std::{
        eq::EqGadget,
        fields::fp::FpVar,
        prelude::{AllocVar, Boolean},
    };
    use ark_relations::r1cs::{self, ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef};
    use ark_snark::SNARK;
    use rand_core::{OsRng, SeedableRng};

    use crate::single::log::Hashable;

    use super::*;

    #[derive(Clone, Copy, Debug)]
    struct TestCircuit {
        x: F,
        y: F,
        pub x_plus_y: F,
    }

    impl ConstraintSynthesizer<F> for TestCircuit {
        fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> r1cs::Result<()> {
            // Witnesses
            let x_var = FpVar::new_witness(cs.clone(), || Ok(self.x))?;
            let y_var = FpVar::new_witness(cs.clone(), || Ok(self.y))?;

            // Public Inputs
            let x_plus_y_var = FpVar::new_input(cs, || Ok(self.x_plus_y))?;

            let add_output = x_var + y_var;

            x_plus_y_var.conditional_enforce_equal(&add_output, &Boolean::TRUE)
        }
    }

    #[test]
    fn test_can_generate_keys_through_ceremony() -> anyhow::Result<()> {
        let circuit = TestCircuit {
            x: F::one(),
            y: F::one(),
            x_plus_y: F::from(2u64),
        };
        let cs = ConstraintSystem::new_ref();
        cs.set_optimization_goal(r1cs::OptimizationGoal::Constraints);
        cs.set_mode(r1cs::SynthesisMode::Setup);
        circuit.generate_constraints(cs.clone())?;
        cs.finalize();

        let matrices = cs
            .to_matrices()
            .ok_or_else(|| anyhow!("Failed to generate constraint matrices."))?;

        let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1776);

        let degree = circuit_degree(&matrices)?;
        let phase1root = Phase1CRSElements::root(degree);

        // Doing two contributions to make sure there's not some weird bug there
        let phase1contribution = Phase1Contribution::make(&mut rng, phase1root.hash(), &phase1root);
        let phase1contribution = Phase1Contribution::make(
            &mut rng,
            phase1contribution.hash(),
            &phase1contribution.new_elements,
        );

        let (extra, phase2root) = transition(&phase1contribution.new_elements, &matrices)?;

        let phase2contribution = Phase2Contribution::make(&mut rng, phase2root.hash(), &phase2root);
        let phase2contribution = Phase2Contribution::make(
            &mut rng,
            phase2contribution.hash(),
            &phase2contribution.new_elements,
        );

        let pk = combine(
            &matrices,
            &phase1contribution.new_elements,
            &phase2contribution.new_elements,
            &extra,
        );

        let proof = Groth16::<Bls12_377, LibsnarkReduction>::prove(&pk, circuit, &mut OsRng)
            .map_err(|err| anyhow!(err))?;

        let public_inputs = vec![circuit.x_plus_y];
        let ok = Groth16::<Bls12_377, LibsnarkReduction>::verify(&pk.vk, &public_inputs, &proof)?;
        assert!(ok);

        Ok(())
    }
}
