//! Leaf binding mechanism for counterparty verification.
//!
//! This module provides privacy-preserving leaf hash blinding that enables
//! counterparty verification without leaking which compliance leaves are transacting.
//!
//! # Design
//!
//! For each transaction:
//! 1. A fresh random `tx_blinding_nonce` is generated (shared between spend and output)
//! 2. Each leaf hash is blinded: `Poseidon(leaf_hash, domain_sep, tx_blinding_nonce)`
//! 3. The blinded hashes are exposed as public inputs in the circuit
//! 4. Transaction validation verifies the binding: spend's counterparty == output's self
//!
//! # Privacy Properties
//!
//! - **No linkability**: Same leaf produces different blinded hashes in each transaction
//! - **No tracking**: Observers cannot correlate transactions to specific leaves
//! - **Binding**: Counterparty relationship is cryptographically enforced

use decaf377::{Fq, Fr};
use penumbra_sdk_tct::StateCommitment;
use poseidon377::hash_3;

/// Domain separator for sender/self leaf binding.
pub const DOMAIN_SEP_SENDER: &[u8] = b"penumbra.leaf_binding.sender";

/// Domain separator for counterparty leaf binding.
pub const DOMAIN_SEP_COUNTERPARTY: &[u8] = b"penumbra.leaf_binding.counterparty";

/// Compute a blinded leaf hash commitment for sender/self.
///
/// # Formula
/// ```text
/// blinded_hash = Poseidon(leaf_hash, domain_sep_sender, tx_blinding_nonce)
/// ```
///
/// # Arguments
/// * `leaf_hash` - The unblinded compliance leaf hash (StateCommitment)
/// * `tx_blinding_nonce` - Shared random nonce for this transaction
///
/// # Returns
/// The blinded leaf hash as a StateCommitment (for use as circuit public input)
pub fn blind_sender_leaf(leaf_hash: StateCommitment, tx_blinding_nonce: Fr) -> StateCommitment {
    let domain_sep =
        Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(DOMAIN_SEP_SENDER).as_bytes());

    // Convert Fr to Fq via bytes (both are 32-byte field elements)
    let nonce_fq = Fq::from_le_bytes_mod_order(&tx_blinding_nonce.to_bytes());

    let blinded = hash_3(&domain_sep, (leaf_hash.0, nonce_fq, Fq::from(0u64)));

    StateCommitment(blinded)
}

/// Compute a blinded leaf hash commitment for counterparty.
///
/// # Formula
/// ```text
/// blinded_hash = Poseidon(leaf_hash, domain_sep_counterparty, tx_blinding_nonce)
/// ```
///
/// # Arguments
/// * `leaf_hash` - The unblinded compliance leaf hash (StateCommitment)
/// * `tx_blinding_nonce` - Shared random nonce for this transaction
///
/// # Returns
/// The blinded leaf hash as a StateCommitment (for use as circuit public input)
pub fn blind_counterparty_leaf(
    leaf_hash: StateCommitment,
    tx_blinding_nonce: Fr,
) -> StateCommitment {
    let domain_sep =
        Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(DOMAIN_SEP_COUNTERPARTY).as_bytes());

    // Convert Fr to Fq via bytes (both are 32-byte field elements)
    let nonce_fq = Fq::from_le_bytes_mod_order(&tx_blinding_nonce.to_bytes());

    let blinded = hash_3(&domain_sep, (leaf_hash.0, nonce_fq, Fq::from(0u64)));

    StateCommitment(blinded)
}

pub mod r1cs {
    //! R1CS (circuit) versions of leaf blinding functions.

    use super::*;
    use ark_r1cs_std::prelude::*;
    use ark_relations::r1cs::ConstraintSystemRef;
    use ark_relations::r1cs::SynthesisError;
    use decaf377::r1cs::FqVar;

    /// Compute a blinded leaf hash commitment for sender/self (in-circuit).
    ///
    /// This MUST match the out-of-circuit `blind_sender_leaf` computation exactly.
    pub fn blind_sender_leaf(
        cs: ConstraintSystemRef<Fq>,
        leaf_hash: FqVar,
        tx_blinding_nonce: FqVar,
    ) -> Result<FqVar, SynthesisError> {
        let domain_sep = FqVar::new_constant(
            cs.clone(),
            Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(DOMAIN_SEP_SENDER).as_bytes()),
        )?;

        poseidon377::r1cs::hash_3(
            cs,
            &domain_sep,
            (leaf_hash, tx_blinding_nonce, FqVar::zero()),
        )
    }

    /// Compute a blinded leaf hash commitment for counterparty (in-circuit).
    ///
    /// This MUST match the out-of-circuit `blind_counterparty_leaf` computation exactly.
    pub fn blind_counterparty_leaf(
        cs: ConstraintSystemRef<Fq>,
        leaf_hash: FqVar,
        tx_blinding_nonce: FqVar,
    ) -> Result<FqVar, SynthesisError> {
        let domain_sep = FqVar::new_constant(
            cs.clone(),
            Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(DOMAIN_SEP_COUNTERPARTY).as_bytes()),
        )?;

        poseidon377::r1cs::hash_3(
            cs,
            &domain_sep,
            (leaf_hash, tx_blinding_nonce, FqVar::zero()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blinding_produces_different_outputs() {
        // Same leaf hash with different nonces should produce different blinded hashes
        let leaf_hash = StateCommitment(Fq::from(12345u64));
        let nonce1 = Fr::from(111u64);
        let nonce2 = Fr::from(222u64);

        let blinded1 = blind_sender_leaf(leaf_hash, nonce1);
        let blinded2 = blind_sender_leaf(leaf_hash, nonce2);

        assert_ne!(
            blinded1.0, blinded2.0,
            "Different nonces should produce different blinded hashes"
        );
    }

    #[test]
    fn test_domain_separation() {
        // Same inputs with different domain separators should produce different outputs
        let leaf_hash = StateCommitment(Fq::from(12345u64));
        let nonce = Fr::from(111u64);

        let sender_blind = blind_sender_leaf(leaf_hash, nonce);
        let counterparty_blind = blind_counterparty_leaf(leaf_hash, nonce);

        assert_ne!(
            sender_blind.0, counterparty_blind.0,
            "Different domain separators should produce different outputs"
        );
    }

    #[test]
    fn test_deterministic() {
        // Same inputs should always produce same output
        let leaf_hash = StateCommitment(Fq::from(12345u64));
        let nonce = Fr::from(111u64);

        let blinded1 = blind_sender_leaf(leaf_hash, nonce);
        let blinded2 = blind_sender_leaf(leaf_hash, nonce);

        assert_eq!(blinded1.0, blinded2.0, "Blinding should be deterministic");
    }
}
