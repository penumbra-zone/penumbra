use anyhow::Result;
use ark_r1cs_std::fields::fp::AllocatedFp;
use ark_r1cs_std::prelude::Boolean;
use decaf377::{Bls12_377, FieldExt, Fq};

use super::zk_gadgets;
use ark_ff::UniformRand;
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};
use ark_r1cs_std::prelude::AllocVar;
use ark_relations::ns;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef};
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
struct OutputCircuit {
    /// Diversified basepoint.
    g_d: decaf377::Element,
}

impl ConstraintSynthesizer<Fq> for OutputCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // First we generate constraints for all the witnesses included on the struct

        // TODO: Use decaf Element gadget for all points here instead of compressing to Fq
        //let g_d = cs.new_witness_variable(|| Ok(self.g_d.vartime_compress_to_field()))?;
        let g_d = AllocatedFp::new_witness(ns!(cs, "diversified_base"), || {
            Ok(self.g_d.vartime_compress_to_field())
        })?;

        let identity_fq: Fq = decaf377::Element::default().vartime_compress_to_field();
        //let identity = FpVar::<Fq>::new_constant(ns!(cs, "decaf_identity"), identity_fq)?;
        let identity = AllocatedFp::new_constant(ns!(cs, "decaf_identity"), identity_fq)?;

        identity.conditional_enforce_not_equal(&g_d, &Boolean::TRUE);

        // TODO: Figure out how to best factor this logic into gadgets

        Ok(())
    }
}

impl OutputCircuit {
    fn setup_random_circuit<R: RngCore + CryptoRng>(rng: &mut R) -> Result<OutputCircuit> {
        let random_fq = Fq::rand(rng);
        let g_d = decaf377::Encoding(random_fq.to_bytes()).vartime_decompress()?;
        Ok(OutputCircuit { g_d })
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
        let circuit = OutputCircuit::setup_random_circuit(rng)?;
        Groth16::circuit_specific_setup(circuit, rng)
            .map_err(|_| anyhow::anyhow!("failure to perform setup"))
    }

    /// Prover POV
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        pk: ProvingKey<Bls12_377>,
        g_d: decaf377::Element,
    ) -> Result<OutputProof> {
        let circuit = OutputCircuit { g_d };
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
