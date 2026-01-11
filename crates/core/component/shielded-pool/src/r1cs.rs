//! R1CS gadgets for compliance-related ZK constraints.
//!
//! Provides constraint gadgets for:
//! - Asset Registry verification: proving an asset's regulatory status
//! - Compliance Registry verification: proving user authorization
//! - Time-based key derivation: deriving keys from CVK + timestamp
//! - Compliance ciphertext binding: proving correct encryption of note data

use ark_r1cs_std::{
    alloc::AllocVar,
    boolean::Boolean,
    prelude::{EqGadget, FieldVar},
};
use ark_relations::r1cs::{ConstraintSystemRef, SynthesisError};
use decaf377::{r1cs::FqVar, Fq};

use decaf377::r1cs::ElementVar;
use penumbra_sdk_compliance::{r1cs::verify_quad_path, structs::MerklePath};

// ============================================================================
// Public Input Packing Helpers
// ============================================================================

/// Packs a 176-byte compliance ciphertext into Field Elements (Fq).
///
/// The ciphertext is chunked into 31-byte pieces to fit safely inside a single Fq
/// (which has ~254 bits of capacity). This matches the chunking logic expected by
/// the circuit's public input allocation.
///
/// # Arguments
///
/// * `ciphertext` - The 176-byte compliance ciphertext
///
/// Verify an asset's registration status in the Asset Registry.
///
/// Proves that `asset_id` has regulatory status `is_regulated` at the given `claimed_anchor`.
/// Uses a Merkle path witness to prove inclusion. Enforcement is conditional: only enforces
/// if the anchor is non-zero (zero anchor = skip verification for untracked assets).
pub fn verify_asset_registry(
    cs: ConstraintSystemRef<Fq>,
    asset_id: FqVar,
    is_regulated: Boolean<Fq>,
    witness_path: &MerklePath,
    witness_position: FqVar,
    claimed_anchor: FqVar,
) -> Result<(), SynthesisError> {
    let zero_domain_sep = FqVar::new_constant(cs.clone(), Fq::from(0u64))?;
    let status_fq = is_regulated.select(&FqVar::one(), &FqVar::zero())?;

    // Construct asset leaf: hash_2(domain_sep, asset_id, status)
    let asset_leaf =
        poseidon377::r1cs::hash_2(cs.clone(), &zero_domain_sep, (asset_id, status_fq))?;

    // Verify Merkle path from leaf to root
    let calculated_root = verify_quad_path(
        cs.clone(),
        asset_leaf,
        witness_path,
        witness_position.clone(),
    )?;

    // Conditionally enforce equality only if anchor is non-zero
    let is_real_anchor = claimed_anchor.is_neq(&FqVar::zero())?;
    calculated_root.conditional_enforce_equal(&claimed_anchor, &is_real_anchor)?;

    Ok(())
}

/// Verify user authorization in the Compliance Registry.
///
/// Proves that a user (identified by address components) is authorized to transact
/// with the given asset at the specified anchor. Enforcement is conditional: only
/// enforces if the asset is regulated AND the anchor is non-zero.
pub fn verify_compliance_registry(
    cs: ConstraintSystemRef<Fq>,
    diversified_generator: ElementVar,
    transmission_key: ElementVar,
    cvk: ElementVar,
    asset_id: FqVar,
    is_regulated: Boolean<Fq>,
    witness_path: &MerklePath,
    witness_position: FqVar,
    claimed_anchor: FqVar,
) -> Result<(), SynthesisError> {
    let is_dummy_anchor =
        claimed_anchor.is_eq(&FqVar::new_constant(cs.clone(), Fq::from(0u64))?)?;
    let should_enforce = is_regulated.and(&is_dummy_anchor.not())?;

    let leaf_domain_sep = FqVar::new_constant(
        cs.clone(),
        Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.compliance.leaf").as_bytes()),
    )?;

    let g_d_fq = diversified_generator.compress_to_field()?;
    let pk_d_fq = transmission_key.compress_to_field()?;
    let cvk_pk_fq = cvk.compress_to_field()?;

    // Construct compliance leaf: hash_4(domain_sep, g_d, pk_d, cvk, asset_id)
    let compliance_leaf = poseidon377::r1cs::hash_4(
        cs.clone(),
        &leaf_domain_sep,
        (g_d_fq, pk_d_fq, cvk_pk_fq, asset_id),
    )?;

    // Verify Merkle path from leaf to root
    let calculated_root = verify_quad_path(
        cs.clone(),
        compliance_leaf,
        witness_path,
        witness_position.clone(),
    )?;

    // Conditionally enforce: only if regulated AND anchor is real
    calculated_root.conditional_enforce_equal(&claimed_anchor, &should_enforce)?;

    Ok(())
}
