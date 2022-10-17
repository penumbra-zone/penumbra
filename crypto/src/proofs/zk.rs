use anyhow::Result;
use decaf377::{Bls12_377, Fq, Fr};

use ark_ec::{bls12::Bls12, PairingEngine, ProjectiveCurve};
use ark_ff::UniformRand;
use ark_groth16::{
    generate_parameters, prepare_verifying_key, verify_proof, Groth16, PreparedVerifyingKey, Proof,
    ProvingKey,
};
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, OptimizationGoal,
};
use ark_snark::SNARK;
use rand::{CryptoRng, RngCore};

// Public:
// * vcm (value commitment)
// * ncm (note commitment)
// * epk (point)
//
// Witnesses:
// * g_d (point)
// * pk_d (point)
// * v (u64 value plus asset ID (scalar))
// * vblind (Fr)
// * nblind (Fq)
// * esk (scalar)
//
// Output circuits check:
// 1. Diversified base is not identity (not implemented).
// 2. Ephemeral public key integrity (not implemented).
// 3. Value commitment integrity (not implemented).
// 4. Note commitment integrity (not implemented).
struct OutputCircuit {}

impl ConstraintSynthesizer<Fq> for OutputCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        todo!()
    }
}

struct OutputProof {
    groth16_proof: Proof<Bls12_377>,
}

impl OutputProof {
    /// Decentralized setup for the OutputProof
    // pub fn decentralized_setup<
    //     E: PairingEngine,
    //     R: RngCore + CryptoRng,
    //     C: ConstraintSynthesizer<Fq>,
    // >(
    //     circuit: C,
    //     rng: &mut R,
    // ) -> R1CSResult<ProvingKey<E>> {
    //     let alpha = Fq::rand(rng);
    //     let beta = Fq::rand(rng);
    //     let gamma = Fq::rand(rng);
    //     let delta = Fq::rand(rng);

    //     let g1_generator = E::G1Projective::prime_subgroup_generator();
    //     let g2_generator = E::G2Projective::prime_subgroup_generator();

    //     let pk = generate_parameters::<E, C, R>(
    //         circuit,
    //         alpha,
    //         beta,
    //         gamma,
    //         delta,
    //         g1_generator,
    //         g2_generator,
    //         rng,
    //     )?;

    //     Ok(pk)
    // }

    /// Prover POV
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        pk: ProvingKey<Bls12_377>,
    ) -> Result<OutputProof> {
        let circuit = OutputCircuit {};
        let groth16_proof = Groth16::prove(&pk, circuit, rng).expect("can prove");
        Ok(OutputProof { groth16_proof })
    }

    /// Verifier POV
    pub fn verify(
        self,
        circuit_pvk: &PreparedVerifyingKey<Bls12_377>,
        public_input: &[Fq],
    ) -> Result<bool> {
        Groth16::verify_with_processed_vk(circuit_pvk, public_input, &self.groth16_proof)
                .map_err(|_| anyhow::anyhow!("boom")),
    }
}

// Test:
// Run decentralized_setup
// Use pk as input in new
