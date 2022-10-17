use anyhow::Result;
use decaf377::{Bls12_377, Fq, Fr};

use ark_ff::UniformRand;
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};
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
    /// Setup (will become the decentralized setup)
    pub fn setup<R: RngCore + CryptoRng>(
        rng: &mut R,
    ) -> Result<(ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>)> {
        let circuit = OutputCircuit {};
        Groth16::circuit_specific_setup(circuit, rng)
            .map_err(|_| anyhow::anyhow!("failure to perform setup"))
    }

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
    pub fn verify(self, vk: &VerifyingKey<Bls12_377>, public_input: &[Fq]) -> Result<bool> {
        let circuit_pvk = Groth16::process_vk(vk)
            .map_err(|_| anyhow::anyhow!("could not process verifying key"))?;
        Groth16::verify_with_processed_vk(&circuit_pvk, public_input, &self.groth16_proof)
            .map_err(|_| anyhow::anyhow!("boom"))
    }
}

// Test:
// Run decentralized_setup
// Use pk as input in new
