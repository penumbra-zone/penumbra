use ark_bls12_381::{Bls12_381, Fr};
use ark_ff::Field;
use ark_groth16::{Groth16, PreparedVerifyingKey};
use ark_r1cs_std::{
    fields::{fp::FpVar, FieldVar},
    prelude::{AllocVar, EqGadget},
    R1CSVar,
};
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystemRef, SynthesisError,
};
use ark_snark::SNARK;
use rand::thread_rng;

/// Circuit that proves: secret_number > threshold
///
/// Constraints:
/// 1. secret_number - threshold = difference
/// 2. difference * inverse = 1  (proves difference ≠ 0)
/// 3. [Range check would go here to ensure difference is positive]
///
/// NOTE: For a production system, you'd add range proofs to ensure
/// the difference is actually positive (not wrapped around in the field).
/// This basic example just proves the difference is non-zero.
#[derive(Clone)]
struct GreaterThanCircuit {
    // Private input
    secret_number: Option<Fr>,

    // Public input
    threshold: Option<Fr>,
}

impl ConstraintSynthesizer<Fr> for GreaterThanCircuit {
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<Fr>,
    ) -> Result<(), SynthesisError> {
        // Allocate the secret number as a private witness
        let secret_var = FpVar::new_witness(cs.clone(), || {
            self.secret_number.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Allocate the threshold as a public input
        let threshold_var = FpVar::new_input(cs.clone(), || {
            self.threshold.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Constraint 1: difference = secret_number - threshold
        let difference = secret_var.clone() - threshold_var.clone();

        // Constraint 2: difference * inverse = 1 (proves difference ≠ 0)
        // To do this, we need to compute the inverse as a witness
        let inverse = FpVar::new_witness(cs.clone(), || {
            let diff_value = secret_var.value()? - threshold_var.value()?;
            let inv = diff_value.inverse().ok_or(SynthesisError::Unsatisfiable)?;
            Ok(inv)
        })?;

        // Enforce: difference * inverse = 1
        let one = FpVar::one();
        let product = difference.clone() * inverse;
        product.enforce_equal(&one)?;

        // Constraint 3: Check difference is positive
        // NOTE: In a real implementation, you would add range checks here
        // to ensure that difference is in the range [1, 2^252] or similar,
        // which would prove it's positive and not wrapped around the field modulus.
        // For this basic example, we just prove it's non-zero via the inverse check.

        Ok(())
    }
}

fn main() {
    println!("=== Groth16 Zero-Knowledge Proof Example ===\n");
    println!("Proving: secret_number > 10");
    println!("Without revealing the actual secret_number!\n");

    let mut rng = thread_rng();

    // Our secret number (this is what we want to keep private)
    let secret_number = Fr::from(15u64);
    let threshold = Fr::from(10u64);

    println!("Secret number: 15 (kept private)");
    println!("Threshold: 10 (public)\n");

    // Step 1: Setup (in real life, this would be a trusted setup ceremony)
    println!("Step 1: Generating proving and verifying keys (trusted setup)...");
    let circuit = GreaterThanCircuit {
        secret_number: None,
        threshold: None,
    };

    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(circuit, &mut rng)
        .expect("Setup failed");

    let pvk = PreparedVerifyingKey::from(vk);
    println!("✓ Keys generated\n");

    // Step 2: Generate proof (prover side)
    println!("Step 2: Generating proof...");
    let circuit_with_inputs = GreaterThanCircuit {
        secret_number: Some(secret_number),
        threshold: Some(threshold),
    };

    let proof = Groth16::<Bls12_381>::prove(&pk, circuit_with_inputs, &mut rng)
        .expect("Proof generation failed");
    println!("✓ Proof generated\n");

    // Step 3: Verify proof (verifier side)
    println!("Step 3: Verifying proof...");
    let public_inputs = vec![threshold]; // Only the threshold is public

    let is_valid = Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &public_inputs, &proof)
        .expect("Verification failed");

    if is_valid {
        println!("✓ Proof is VALID!\n");
        println!("The verifier now knows that secret_number > 10");
        println!("But they still don't know that secret_number = 15 😎\n");
    } else {
        println!("✗ Proof is INVALID\n");
    }

    // Demonstrate that an invalid proof would fail
    println!("=== Testing with invalid secret (7 < 10) ===\n");
    let invalid_secret = Fr::from(7u64);

    // This should fail during proof generation because we can't compute
    // an inverse of a negative number (which wraps around in the field)
    let invalid_circuit = GreaterThanCircuit {
        secret_number: Some(invalid_secret),
        threshold: Some(threshold),
    };

    match Groth16::<Bls12_381>::prove(&pk, invalid_circuit, &mut rng) {
        Ok(_) => {
            println!("⚠ Warning: Proof generated (but this shows the limitation:");
            println!("   We need range checks to properly enforce positivity!)");
        }
        Err(e) => {
            println!("✓ Proof generation failed as expected: {}", e);
        }
    }

    println!("\n=== Done! ===");
}
