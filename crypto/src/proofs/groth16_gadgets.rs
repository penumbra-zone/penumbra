#![allow(clippy::too_many_arguments)]
use ark_r1cs_std::{prelude::*, ToBitsGadget};
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Fq,
};

use crate::{
    asset::VALUE_GENERATOR_DOMAIN_SEP, balance::commitment::VALUE_BLINDING_GENERATOR,
    note::NOTECOMMIT_DOMAIN_SEP, nullifier::NULLIFIER_DOMAIN_SEP,
};

/// Check the diversified basepoint is not identity.
pub(crate) fn diversified_basepoint_not_identity(
    cs: ConstraintSystemRef<Fq>,
    // Witness
    g_d: ElementVar,
) -> Result<(), SynthesisError> {
    let identity = ElementVar::new_constant(cs, decaf377::Element::default())?;
    identity.enforce_not_equal(&g_d)?;
    Ok(())
}

/// Check the integrity of the ephemeral public key.
pub(crate) fn ephemeral_public_key_integrity(
    // Witnesses
    esk: Vec<UInt8<Fq>>,
    g_d: ElementVar,
    // Public inputs,
    epk: ElementVar,
) -> Result<(), SynthesisError> {
    let expected_epk = g_d.scalar_mul_le(esk.to_bits_le()?.iter())?;
    expected_epk.enforce_equal(&epk)?;
    Ok(())
}

/// Check the integrity of the value commitment.
pub(crate) fn value_commitment_integrity(
    cs: ConstraintSystemRef<Fq>,
    // Witnesses
    value_amount: Vec<UInt8<Fq>>,
    value_asset_id: FqVar,
    value_blinding: Vec<UInt8<Fq>>,
    // Public inputs,
    commitment: ElementVar,
) -> Result<(), SynthesisError> {
    let value_generator = FqVar::new_constant(cs.clone(), *VALUE_GENERATOR_DOMAIN_SEP)?;
    let value_blinding_generator = ElementVar::new_constant(cs.clone(), *VALUE_BLINDING_GENERATOR)?;

    let hashed_asset_id = poseidon377::r1cs::hash_1(cs.clone(), &value_generator, value_asset_id)?;
    let asset_generator = ElementVar::encode_to_curve(&hashed_asset_id)?;
    let test_commitment = asset_generator.scalar_mul_le(value_amount.to_bits_le()?.iter())?
        + value_blinding_generator.scalar_mul_le(value_blinding.to_bits_le()?.iter())?;

    commitment.enforce_equal(&test_commitment)?;
    Ok(())
}

/// Check the integrity of the note commitment.
pub(crate) fn note_commitment_integrity(
    cs: ConstraintSystemRef<Fq>,
    // Witnesses
    note_blinding: FqVar,
    value_amount: FqVar,
    value_asset_id: FqVar,
    diversified_generator: ElementVar,
    transmission_key_s: FqVar,
    clue_key: FqVar,
    // Public inputs
    commitment: FqVar,
) -> Result<(), SynthesisError> {
    let value_blinding_generator = FqVar::new_constant(cs.clone(), *NOTECOMMIT_DOMAIN_SEP)?;

    let compressed_g_d = diversified_generator.compress_to_field()?;
    let commitment_test = poseidon377::r1cs::hash_6(
        cs,
        &value_blinding_generator,
        (
            note_blinding,
            value_amount,
            value_asset_id,
            compressed_g_d,
            transmission_key_s,
            clue_key,
        ),
    )?;

    commitment.enforce_equal(&commitment_test)?;
    Ok(())
}

/// Nullifier integrity.
#[allow(dead_code)]
pub(crate) fn nullifier_integrity(
    cs: ConstraintSystemRef<Fq>,
    // Witnesses
    note_commitment: FqVar,
    nk: FqVar,
    position: FqVar,
    // Public input
    nullifier: FqVar,
) -> Result<(), SynthesisError> {
    let nullifier_constant = FqVar::new_constant(cs.clone(), *NULLIFIER_DOMAIN_SEP)?;

    let computed_nullifier =
        poseidon377::r1cs::hash_3(cs, &nullifier_constant, (nk, note_commitment, position))?;

    nullifier.enforce_equal(&computed_nullifier)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use ark_ff::{One, PrimeField, ToConstraintField};
    use ark_groth16::{Groth16, ProvingKey, VerifyingKey};
    use ark_relations::r1cs::ConstraintSynthesizer;
    use ark_snark::SNARK;
    use proptest::prelude::*;
    use rand_core::OsRng;
    use std::str::FromStr;

    use super::*;

    use crate::{keys::Diversifier, Address, Note, Value};
    use decaf377::{r1cs::CountConstraints, Bls12_377, Element};
    use decaf377_fmd as fmd;
    use decaf377_ka as ka;

    #[derive(Clone)]
    struct TestNoteCommitmentCircuit {
        // Witnesses
        note: Note,

        // Public input
        pub note_commitment: Fq,
    }

    impl ConstraintSynthesizer<Fq> for TestNoteCommitmentCircuit {
        fn generate_constraints(
            self,
            cs: ark_relations::r1cs::ConstraintSystemRef<Fq>,
        ) -> ark_relations::r1cs::Result<()> {
            // Add witness variables for the note.
            // These variables may be needed for multiple integrity checks, so we do this outside the gadget functions.
            let note_blinding_var =
                FqVar::new_witness(cs.clone(), || Ok(self.note.note_blinding().clone()))?;
            let value_amount_var =
                FqVar::new_witness(cs.clone(), || Ok(Fq::from(self.note.value().amount)))?;
            let value_asset_id_var =
                FqVar::new_witness(cs.clone(), || Ok(self.note.value().asset_id.0))?;
            let diversified_generator_var =
                AllocVar::<Element, Fq>::new_witness(cs.clone(), || {
                    Ok(self.note.diversified_generator().clone())
                })?;
            let transmission_key_s_var =
                FqVar::new_witness(cs.clone(), || Ok(self.note.transmission_key_s().clone()))?;
            let clue_key_var = FqVar::new_witness(cs.clone(), || {
                Ok(Fq::from_le_bytes_mod_order(&self.note.clue_key().0[..]))
            })?;

            // Add public input variable.
            let note_commitment_var = FqVar::new_input(cs.clone(), || Ok(self.note_commitment))?;

            note_commitment_integrity(
                cs,
                note_blinding_var,
                value_amount_var,
                value_asset_id_var,
                diversified_generator_var,
                transmission_key_s_var,
                clue_key_var,
                note_commitment_var,
            )?;

            Ok(())
        }
    }

    impl TestNoteCommitmentCircuit {
        fn generate_test_parameters() -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>) {
            let diversifier_bytes = [1u8; 16];
            let pk_d_bytes = [1u8; 32];
            let clue_key_bytes = [1; 32];
            let diversifier = Diversifier(diversifier_bytes);
            let address = Address::from_components(
                diversifier,
                ka::Public(pk_d_bytes),
                fmd::ClueKey(clue_key_bytes),
            )
            .expect("generated 1 address");
            let note = Note::from_parts(
                address,
                Value::from_str("1upenumbra").expect("valid value"),
                Fq::from(1),
            )
            .expect("can make a note");
            let circuit = TestNoteCommitmentCircuit {
                note: note.clone(),
                note_commitment: note.commit().0,
            };
            let (pk, vk) = Groth16::circuit_specific_setup(circuit, &mut OsRng)
                .expect("can perform circuit specific setup");
            (pk, vk)
        }
    }

    fn fq_strategy() -> BoxedStrategy<Fq> {
        any::<[u8; 32]>()
            .prop_map(|bytes| Fq::from_le_bytes_mod_order(&bytes[..]))
            .boxed()
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
        fn groth16_note_commitment_proof_happy_path(note_blinding in fq_strategy()) {
            let (pk, vk) = TestNoteCommitmentCircuit::generate_test_parameters();
            let mut rng = OsRng;

            // Prover POV
            let diversifier = Diversifier([0u8; 16]);
            let pk_d_bytes = [1u8; 32];
            let clue_key_bytes = [0u8; 32];
            let address = Address::from_components(
                diversifier,
                ka::Public(pk_d_bytes),
                fmd::ClueKey(clue_key_bytes),
            ).unwrap();
            let value = Value::from_str("1upenumbra").expect("this is a valid value");
            let note = Note::from_parts(
                address, value, note_blinding
            ).expect("can make a note");
            let note_commitment = note.commit().0;
            let circuit = TestNoteCommitmentCircuit {
                note,
                note_commitment,
            };
            dbg!(circuit.clone().num_constraints_and_instance_variables());

            let proof = Groth16::prove(&pk, circuit, &mut rng)
                .map_err(|_| anyhow::anyhow!("invalid proof"))
                .expect("can generate proof");

            // Verifier POV
            let processed_pvk = Groth16::process_vk(&vk).expect("can process verifying key");
            let public_inputs = note_commitment.to_field_elements().unwrap();
            let proof_result =
                Groth16::verify_with_processed_vk(&processed_pvk, &public_inputs, &proof).unwrap();

            assert!(proof_result);
        }
    }

    proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
        fn groth16_note_commitment_proof_unhappy_path(note_blinding in fq_strategy()) {
            let (pk, vk) = TestNoteCommitmentCircuit::generate_test_parameters();
            let mut rng = OsRng;

            // Prover POV
            let diversifier = Diversifier([0u8; 16]);
            let pk_d_bytes = [1u8; 32];
            let clue_key_bytes = [0u8; 32];
            let address = Address::from_components(
                diversifier,
                ka::Public(pk_d_bytes),
                fmd::ClueKey(clue_key_bytes),
            ).unwrap();
            let value = Value::from_str("1upenumbra").expect("this is a valid value");
            let note = Note::from_parts(
                address, value, note_blinding
            ).expect("can make a note");
            let note_commitment = note.commit().0;
            let circuit = TestNoteCommitmentCircuit {
                note,
                note_commitment,
            };
            dbg!(circuit.clone().num_constraints_and_instance_variables());

            let proof = Groth16::prove(&pk, circuit, &mut rng)
                .map_err(|_| anyhow::anyhow!("invalid proof"))
                .expect("can generate proof");

            // Verifier POV
            let processed_pvk = Groth16::process_vk(&vk).expect("can process verifying key");
            let incorrect_note_commitment = note_commitment + Fq::one();
            let public_inputs = incorrect_note_commitment.to_field_elements().unwrap();
            let proof_result =
                Groth16::verify_with_processed_vk(&processed_pvk, &public_inputs, &proof).unwrap();

            assert!(!proof_result);
        }
    }
}
