use ark_bls12_377::{Config, Fq, Fr, G1Projective, G2Projective};
use ark_bw6_761::BW6_761;
use ark_ec::Group;
use ark_ff::UniformRand;
use ark_groth16::{r1cs_to_qap::LibsnarkReduction, Groth16};
use ark_r1cs_std::groups::bls12::{G1Var, G2Var};
use ark_r1cs_std::pairing::bls12::PairingVar;
use ark_r1cs_std::pairing::PairingVar as PairingVarTrait;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, SynthesisError,
};
use ark_snark::SNARK;
use criterion::{criterion_group, criterion_main, Criterion};
use rand_core::OsRng;

/// BLS threshold signature verification circuit over BLS12-377.
///
/// Verifies the pairing equation: e(sig, g2) == e(H(m), pubkey)
///
/// In a threshold setting, the aggregated signature and public key
/// are computed outside the circuit. The circuit only verifies
/// the final pairing check.
#[derive(Clone)]
struct BlsThresholdCircuit {
    /// Aggregated signature (point in G1)
    aggregate_sig: G1Projective,
    /// Aggregated public key (point in G2)
    aggregate_pubkey: G2Projective,
    /// Message hash (already hashed to G1 curve point)
    message_hash: G1Projective,
}

impl ConstraintSynthesizer<Fq> for BlsThresholdCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> Result<(), SynthesisError> {
        // Allocate signature as witness (private input)
        let sig_var = G1Var::<Config>::new_witness(cs.clone(), || Ok(self.aggregate_sig))?;

        // Allocate public key as witness
        let pubkey_var = G2Var::<Config>::new_witness(cs.clone(), || Ok(self.aggregate_pubkey))?;

        // Allocate message hash as witness
        let msg_var = G1Var::<Config>::new_witness(cs.clone(), || Ok(self.message_hash))?;

        // G2 generator as constant
        let g2_gen = G2Var::<Config>::new_constant(cs.clone(), G2Projective::generator())?;

        // Prepare points for pairing computation
        let prepared_sig = PairingVar::<Config>::prepare_g1(&sig_var)?;
        let prepared_g2 = PairingVar::<Config>::prepare_g2(&g2_gen)?;
        let prepared_msg = PairingVar::<Config>::prepare_g1(&msg_var)?;
        let prepared_pubkey = PairingVar::<Config>::prepare_g2(&pubkey_var)?;

        // Compute pairings: e(sig, g2) and e(msg, pubkey)
        let lhs = PairingVar::<Config>::pairing(prepared_sig, prepared_g2)?;
        let rhs = PairingVar::<Config>::pairing(prepared_msg, prepared_pubkey)?;

        // Enforce equality: e(sig, g2) == e(msg, pubkey)
        lhs.enforce_equal(&rhs)?;

        Ok(())
    }
}

impl BlsThresholdCircuit {
    /// Generate a valid BLS signature for testing.
    fn generate_valid() -> Self {
        let mut rng = ark_std::test_rng();

        // Generate random secret key
        let sk = Fr::rand(&mut rng);

        // Public key = sk * G2
        let pubkey = G2Projective::generator() * sk;

        // Random message point (simulating hash-to-curve output)
        let msg = G1Projective::rand(&mut rng);

        // Signature = sk * H(m)
        let sig = msg * sk;

        Self {
            aggregate_sig: sig,
            aggregate_pubkey: pubkey,
            message_hash: msg,
        }
    }
}

fn bench_bls_proving(c: &mut Criterion) {
    let mut rng = OsRng;

    // First, print constraint count
    let circuit = BlsThresholdCircuit::generate_valid();
    let cs = ConstraintSystem::<Fq>::new_ref();
    circuit
        .generate_constraints(cs.clone())
        .expect("constraint generation should succeed");

    println!("BLS threshold verify constraints: {}", cs.num_constraints());
    assert!(
        cs.is_satisfied().expect("should check satisfaction"),
        "circuit should be satisfied with valid signature"
    );

    // Generate proving and verifying keys using BW6-761
    // BW6-761's scalar field = BLS12-377's base field (Fq), so it can prove circuits over Fq
    println!("Generating BLS threshold proving keys with BW6-761 (this may take a while)...");
    let circuit_template = BlsThresholdCircuit::generate_valid();
    let (pk, _vk) =
        Groth16::<BW6_761, LibsnarkReduction>::circuit_specific_setup(circuit_template, &mut rng)
            .expect("can perform circuit setup");
    println!("Proving keys generated.");

    // Benchmark actual Groth16 proving time
    c.bench_function("bls12-377 threshold verify (proving)", |b| {
        b.iter(|| {
            let circuit = BlsThresholdCircuit::generate_valid();
            let _proof = Groth16::<BW6_761, LibsnarkReduction>::prove(&pk, circuit, &mut rng)
                .expect("can create proof");
        })
    });
}

criterion_group!(benches, bench_bls_proving);
criterion_main!(benches);
