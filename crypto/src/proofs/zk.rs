use anyhow::Result;
use ark_r1cs_std::prelude::{Boolean, CurveVar, EqGadget};
use ark_r1cs_std::uint8::UInt8;
use ark_r1cs_std::ToBitsGadget;
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Bls12_377, Fq, Fr,
};
use decaf377_ka as ka;

use ark_ff::ToConstraintField;
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
// 1. Diversified base is not identity (implemented).
// 2. Ephemeral public key integrity (implemented).
// 3. Value commitment integrity (not implemented).
// 4. Note commitment integrity (not implemented).
struct OutputCircuit {
    // Private witnesses
    /// Diversified basepoint.
    g_d: decaf377::Element,
    /// Ephemeral secret key.
    esk: ka::Secret,

    // Public inputs
    /// Ephemeral public key.
    pub epk: ka::Public,
}

impl ConstraintSynthesizer<Fq> for OutputCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fq>) -> ark_relations::r1cs::Result<()> {
        let g_d = ElementVar::new_witness(ns!(cs, "diversified_base"), || Ok(self.g_d))?;

        // 1. Diversified base is not identity.
        let decaf_identity = decaf377::Element::default();
        let identity = ElementVar::new_constant(ns!(cs, "decaf_identity"), decaf_identity)?;
        identity.conditional_enforce_not_equal(&g_d, &Boolean::TRUE)?;

        // 2. Ephemeral public key integrity.
        let esk_arr: [u8; 32] = self.esk.to_bytes();
        let esk_vars = UInt8::new_witness_vec(cs.clone(), &esk_arr)?;

        let pk = decaf377::Encoding(self.epk.0)
            .vartime_decompress()
            .expect("valid public key");
        let epk = ElementVar::new_input(ns!(cs, "epk"), || Ok(pk))?;

        let expected_epk = g_d.scalar_mul_le(esk_vars.to_bits_le()?.iter())?;
        expected_epk.conditional_enforce_equal(&epk, &Boolean::Constant(true))?;

        // TODO: 3. Value commitment integrity
        // Check: value_commitment == -self.value.commit(self.v_blinding)
        // P = a + b = [v] G_v + [v_blinding] H
        // Requires: Asset specific generator (hash to group)

        // TODO: 4. Note commitment integrity
        // Requires: Poseidon gadget

        Ok(())
    }
}

impl OutputCircuit {
    fn setup_circuit() -> Result<OutputCircuit> {
        let g_d = Fr::from(2) * decaf377::basepoint();
        let esk = ka::Secret::new_from_field(Fr::from(666));
        let epk = esk.diversified_public(&g_d);

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
        let circuit = OutputCircuit::setup_circuit()?;
        Groth16::circuit_specific_setup(circuit, rng).map_err(|err| anyhow::anyhow!(err))
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
    pub fn verify(self, vk: &VerifyingKey<Bls12_377>, epk: ka::Public) -> Result<bool> {
        // Ephemeral public key
        let pk = decaf377::Encoding(epk.0)
            .vartime_decompress()
            .expect("valid public key");

        let public_inputs = pk
            .to_field_elements()
            .expect("can convert decaf Elements to constraint field");
        let circuit_pvk = Groth16::process_vk(vk)
            .map_err(|_| anyhow::anyhow!("could not process verifying key"))?;
        Groth16::verify_with_processed_vk(&circuit_pvk, &public_inputs, &self.groth16_proof)
            .map_err(|err| anyhow::anyhow!(err))
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use rand_core::OsRng;

    use crate::keys::{SeedPhrase, SpendKey};

    use super::*;

    #[test]
    fn groth16_output_happy() {
        let start = Instant::now();
        let (pk, vk) = OutputProof::setup(&mut OsRng)
            .expect("can perform test Groth16 setup for output circuit");
        let duration = start.elapsed();
        println!("Time elapsed in setup: {:?}", duration);

        let seed_phrase = SeedPhrase::generate(&mut OsRng);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());
        let g_d = dest.diversified_generator();
        let esk = ka::Secret::new(&mut OsRng);
        let epk = esk.diversified_public(&g_d);

        let start = Instant::now();
        let proof = OutputProof::new(&mut OsRng, &pk, *g_d, esk, epk)
            .expect("can prove using a test output circuit");
        let duration = start.elapsed();
        println!("Time elapsed in proof creation: {:?}", duration);

        let start = Instant::now();
        assert!(proof.verify(&vk, epk).unwrap());
        let duration = start.elapsed();
        println!("Time elapsed in proof verification: {:?}", duration);
    }

    #[test]
    fn groth16_output_incorrect_diversified_base_expected_failure() {
        let (pk, vk) = OutputProof::setup(&mut OsRng)
            .expect("can perform test Groth16 setup for output circuit");

        let seed_phrase = SeedPhrase::generate(&mut OsRng);
        let sk_recipient = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());
        let incorrect_g_d = Fr::from(3) * decaf377::basepoint();
        let g_d = dest.diversified_generator();
        let esk = ka::Secret::new(&mut OsRng);
        let epk = esk.diversified_public(&g_d);
        let wrong_epk = esk.diversified_public(&incorrect_g_d);

        let proof = OutputProof::new(&mut OsRng, &pk, *g_d, esk, epk)
            .expect("can prove using a test output circuit");

        assert!(!proof.verify(&vk, wrong_epk).unwrap());
    }
}
