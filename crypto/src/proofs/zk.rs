use anyhow::Result;
use ark_r1cs_std::fields::fp::AllocatedFp;
use ark_r1cs_std::prelude::Boolean;
use ark_serialize::CanonicalDeserialize;
use decaf377::{Bls12_377, Fq};
use decaf377_ka as ka;

use super::zk_gadgets;
use ark_ff::UniformRand;
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey};
use ark_r1cs_std::{groups::curves::short_weierstrass::ProjectiveVar, prelude::AllocVar};
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
// 1. Diversified base is not identity (implemented).
// 2. Ephemeral public key integrity (not implemented).
// 3. Value commitment integrity (not implemented).
// 4. Note commitment integrity (not implemented).
struct OutputCircuit {
    // Witnesses
    /// Diversified basepoint.
    g_d: decaf377::Element,
    /// Ephemeral secret key.
    esk: ka::Secret,

    // Inputs
    /// Ephemeral public key.
    epk: ka::Public,
}

impl ConstraintSynthesizer<Fq> for OutputCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        // TODO: Use decaf Element gadget for all points here instead of compressing to Fq
        //let g_d = cs.new_witness_variable(|| Ok(self.g_d.vartime_compress_to_field()))?;
        let g_d = AllocatedFp::new_witness(ns!(cs, "diversified_base"), || {
            Ok(self.g_d.vartime_compress_to_field())
        })?;

        // 1. Diversified base is not identity.
        let identity_fq: Fq = decaf377::Element::default().vartime_compress_to_field();
        //let identity = FpVar::<Fq>::new_constant(ns!(cs, "decaf_identity"), identity_fq)?;
        let identity = AllocatedFp::new_constant(ns!(cs, "decaf_identity"), identity_fq)?;

        identity.conditional_enforce_not_equal(&g_d, &Boolean::TRUE)?;

        // 2. Ephemeral public key integrity.
        // Lift esk to proving curve - TODO: is this the best way to do this?
        let esk_fq =
            &Fq::deserialize(&self.esk.to_bytes()[..]).expect("should be infallible since r << q");
        let esk = cs.new_witness_variable(|| Ok(*esk_fq))?;

        let pk = decaf377::Encoding(self.epk.0)
            .vartime_decompress()
            .expect("valid public key");
        let epk = cs.new_input_variable(|| Ok(pk.vartime_compress_to_field()))?;
        //let epk = ProjectiveVar::new()

        // TODO: Figure out how to best factor this logic into gadgets

        Ok(())
    }
}

impl OutputCircuit {
    fn setup_random_circuit<R: RngCore + CryptoRng>(rng: &mut R) -> Result<OutputCircuit> {
        let random_fq = Fq::rand(rng);
        let g_d = decaf377::Element::encode_to_curve(&random_fq);

        let esk = ka::Secret::new(rng);
        let epk = esk.public();

        Ok(OutputCircuit { g_d, esk, epk })
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
        pk: &ProvingKey<Bls12_377>,
        g_d: decaf377::Element,
        esk: ka::Secret,
        epk: ka::Public,
    ) -> Result<OutputProof> {
        let circuit = OutputCircuit { g_d, esk, epk };
        let groth16_proof =
            Groth16::prove(&pk, circuit, rng).map_err(|_| anyhow::anyhow!("invalid proof"))?;
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

#[cfg(test)]
mod test {
    use std::time::Instant;

    use rand_core::OsRng;

    use super::*;

    #[test]
    fn groth16_output_happy() {
        let mut rng = OsRng;

        // Random (non-zero) diversified base
        let random_fq = Fq::rand(&mut rng);
        let g_d = decaf377::Element::encode_to_curve(&random_fq);
        let start = Instant::now();
        let (pk, vk) = OutputProof::setup(&mut rng)
            .expect("can perform test Groth16 setup for output circuit");
        let duration = start.elapsed();
        println!("Time elapsed in setup: {:?}", duration);

        let esk = ka::Secret::new(&mut rng);
        let epk = esk.public();

        let start = Instant::now();
        let proof = OutputProof::new(&mut rng, &pk, g_d, esk, epk)
            .expect("can prove using a test output circuit");
        let duration = start.elapsed();
        println!("Time elapsed in proof creation: {:?}", duration);

        let pk = decaf377::Encoding(epk.0)
            .vartime_decompress()
            .expect("valid public key");
        let epk_fq = pk.vartime_compress_to_field();
        let start = Instant::now();
        assert!(proof.verify(&vk, &[epk_fq]).unwrap());
        let duration = start.elapsed();
        println!("Time elapsed in proof verification: {:?}", duration);
    }

    #[test]
    fn groth16_output_diversified_base_expected_failure() {
        let mut rng = OsRng;

        // Invalid (identity) diversified base
        let g_d = decaf377::Element::default();
        let (pk, vk) = OutputProof::setup(&mut rng)
            .expect("can perform test Groth16 setup for output circuit");

        let esk = ka::Secret::new(&mut rng);
        let epk = esk.public();

        // Cannot form proof with invalid diversified base - this involves only constants and witnesses
        assert!(OutputProof::new(&mut rng, &pk, g_d, esk, epk).is_err());
    }
}
