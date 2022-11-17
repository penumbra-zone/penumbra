#![allow(clippy::too_many_arguments)]
use ark_r1cs_std::prelude::{AllocVar, EqGadget};
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Fq,
};

use crate::note::NOTECOMMIT_DOMAIN_SEP;

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
    note_commitment: FqVar,
) -> Result<(), SynthesisError> {
    let value_blinding_generator = FqVar::new_constant(cs.clone(), *NOTECOMMIT_DOMAIN_SEP)?;

    let compressed_g_d = diversified_generator.compress_to_field()?;
    let note_commitment_test = poseidon377::r1cs::hash_6(
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

    note_commitment.enforce_equal(&note_commitment_test)?;
    Ok(())
}
