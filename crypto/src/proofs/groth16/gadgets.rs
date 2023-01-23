#![allow(clippy::too_many_arguments)]
use ark_r1cs_std::{prelude::*, ToBitsGadget};
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Fq,
};

use crate::{asset::VALUE_GENERATOR_DOMAIN_SEP, balance::commitment::VALUE_BLINDING_GENERATOR};

/// Check the integrity of the value commitment.
pub(crate) fn value_commitment_integrity(
    cs: ConstraintSystemRef<Fq>,
    enforce: &Boolean<Fq>,
    // Witnesses
    value_amount: Vec<UInt8<Fq>>,
    value_asset_id: FqVar,
    value_blinding: Vec<UInt8<Fq>>,
    // Public inputs,
    commitment: ElementVar,
) -> Result<(), SynthesisError> {
    let value_generator = FqVar::new_constant(cs.clone(), *VALUE_GENERATOR_DOMAIN_SEP)?;
    let value_blinding_generator = ElementVar::new_constant(cs.clone(), *VALUE_BLINDING_GENERATOR)?;

    let hashed_asset_id = poseidon377::r1cs::hash_1(cs, &value_generator, value_asset_id)?;
    let asset_generator = ElementVar::encode_to_curve(&hashed_asset_id)?;
    let test_commitment = asset_generator.scalar_mul_le(value_amount.to_bits_le()?.iter())?
        + value_blinding_generator.scalar_mul_le(value_blinding.to_bits_le()?.iter())?;

    commitment.conditional_enforce_equal(&test_commitment, enforce)?;
    Ok(())
}
