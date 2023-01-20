#![allow(clippy::too_many_arguments)]
use ark_ff::PrimeField;
use ark_nonnative_field::NonNativeFieldVar;
use ark_r1cs_std::{prelude::*, ToBitsGadget};
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Element, FieldExt, Fq, Fr,
};
use once_cell::sync::Lazy;

use crate::{
    asset::VALUE_GENERATOR_DOMAIN_SEP, balance::commitment::VALUE_BLINDING_GENERATOR,
    keys::IVK_DOMAIN_SEP,
};

pub(crate) static SPENDAUTH_BASEPOINT: Lazy<Element> = Lazy::new(decaf377::basepoint);

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

/// Check integrity of the diversified address.
pub(crate) fn diversified_address_integrity(
    cs: ConstraintSystemRef<Fq>,
    enforce: &Boolean<Fq>,
    // Witnesses
    ak: FqVar,
    nk: FqVar,
    transmission_key: ElementVar,
    diversified_generator: ElementVar,
) -> Result<(), SynthesisError> {
    let ivk_domain_sep = FqVar::new_constant(cs.clone(), *IVK_DOMAIN_SEP)?;
    let ivk_mod_q = poseidon377::r1cs::hash_2(cs.clone(), &ivk_domain_sep, (nk, ak))?;

    // Reduce `ivk_mod_q` modulo r
    let inner_ivk_mod_q: Fq = ivk_mod_q.value().unwrap_or_default();
    let ivk_mod_r = Fr::from_le_bytes_mod_order(&inner_ivk_mod_q.to_bytes());
    let ivk =
        NonNativeFieldVar::<Fr, Fq>::new_variable(cs, || Ok(ivk_mod_r), AllocationMode::Witness)?;

    // Now add constraints to demonstrate the transmission key = [ivk] g_d
    let ivk_vars = ivk.to_bits_le()?;
    let test_transmission_key =
        diversified_generator.scalar_mul_le(ivk_vars.to_bits_le()?.iter())?;
    transmission_key.conditional_enforce_equal(&test_transmission_key, enforce)?;
    Ok(())
}

/// Check integrity of randomized verification key.
pub(crate) fn rk_integrity(
    cs: ConstraintSystemRef<Fq>,
    enforce: &Boolean<Fq>,
    // Witnesses
    ak: ElementVar,
    spend_auth_randomizer: Vec<UInt8<Fq>>,
    // Public inputs
    rk: FqVar,
) -> Result<(), SynthesisError> {
    let spend_auth_basepoint_var = ElementVar::new_constant(cs, *SPENDAUTH_BASEPOINT)?;
    let point =
        ak + spend_auth_basepoint_var.scalar_mul_le(spend_auth_randomizer.to_bits_le()?.iter())?;
    let computed_rk = ElementVar::compress_to_field(&point)?;
    rk.conditional_enforce_equal(&computed_rk, enforce)?;
    Ok(())
}
