//! Helper functions for generating compliance ciphertexts and leaves.
//!
//! This module provides client-side utilities for creating compliance proofs
//! using the ACK (Wallet Compliance Key) system.

use anyhow::Result;
use penumbra_sdk_asset::asset;
use penumbra_sdk_compliance::{
    crypto, structs::ComplianceCiphertext, ComplianceLeaf, BLACK_HOLE_ACK,
};
use penumbra_sdk_keys::{keys::AddressComplianceKey, Address};
use penumbra_sdk_num::Amount;
use rand_core::{CryptoRng, RngCore};

/// Generated compliance data for a transaction action.
///
/// This struct groups all the compliance-related data generated during transaction
/// construction, making it easier to pass around and use in proofs.
#[derive(Clone, Debug)]
pub struct GeneratedComplianceData {
    /// The compliance ciphertext containing encrypted transaction details.
    /// This gets included in the transaction's public inputs.
    pub ciphertext: Vec<u8>,
    /// The compliance leaf for the user's registry entry.
    /// This is used as a witness in the ZK proof.
    pub leaf: penumbra_sdk_compliance::ComplianceLeaf,
    /// The ephemeral secret used to encrypt the compliance data.
    /// This must be provided as a private witness to the circuit.
    pub ephemeral_secret: decaf377::Fr,
}

/// Generate compliance ciphertext and leaf for a transaction.
///
/// This function handles both regulated and unregulated assets:
/// - **Regulated**: Encrypts to the real user's ACK from the registry
/// - **Unregulated**: Encrypts to BLACK_HOLE_ACK (unlinkable)
///
/// # Arguments
///
/// * `rng` - Random number generator for ephemeral key generation
/// * `is_regulated` - Whether the asset requires compliance
/// * `user_ack` - The user's Wallet Compliance Key (from registry, or dummy if unregulated)
/// * `user_address` - The user's address (contains diversifier)
/// * `date` - The current date (Unix day index: timestamp / 86400)
/// * `asset_id` - The asset being transacted
/// * `amount` - The amount being transacted
/// * `counterparty_address` - The other party's address in the transaction
///
/// # Returns
///
/// A `GeneratedComplianceData` struct containing the ciphertext bytes, compliance leaf,
/// and ephemeral secret needed by the circuit as a private witness.
pub fn generate_compliance_details(
    rng: &mut (impl RngCore + CryptoRng),
    is_regulated: bool,
    user_ack: &AddressComplianceKey,
    user_address: &Address,
    date: u64,
    asset_id: asset::Id,
    amount: Amount,
    counterparty_address: Address,
) -> Result<GeneratedComplianceData> {
    let (ciphertext, leaf, ephemeral_secret) = if is_regulated {
        // Regulated: Use real ACK from user's registry entry
        let (ciphertext, ephemeral_secret) = crypto::encrypt_compliance_details(
            rng,
            user_ack,
            user_address,
            date,
            asset_id,
            amount,
            counterparty_address,
        )?;

        // Create the compliance leaf for ZK proof
        let leaf = ComplianceLeaf {
            address: user_address.clone(),
            key: user_ack.clone(),
            asset_id,
        };

        (ciphertext, leaf, ephemeral_secret)
    } else {
        // Unregulated: Encrypt to BLACK_HOLE_ACK
        // This makes the transaction unlinkable to any specific user
        let black_hole_ack = AddressComplianceKey::new(*BLACK_HOLE_ACK);

        let (ciphertext, ephemeral_secret) = crypto::encrypt_compliance_details(
            rng,
            &black_hole_ack,
            user_address, // Still use user's diversifier for consistency
            date,
            asset_id,
            amount,
            counterparty_address,
        )?;

        // Create a dummy leaf (will be self-validating in ZK proof)
        // The conditional enforcement in the circuit will skip verification
        let dummy_leaf = ComplianceLeaf {
            address: user_address.clone(),
            key: user_ack.clone(), // Can be any ACK, will be ignored
            asset_id,
        };

        (ciphertext, dummy_leaf, ephemeral_secret)
    };

    Ok(GeneratedComplianceData {
        ciphertext: ciphertext.to_bytes(),
        leaf,
        ephemeral_secret,
    })
}

/// Helper to convert Unix timestamp (seconds) to day index for ACK derivation.
#[inline]
pub fn timestamp_to_day_index(timestamp: u64) -> u64 {
    timestamp / 86400 // 86400 seconds per day
}

/// Serialize compliance ciphertext to bytes for inclusion in public inputs.
///
/// The format is:
/// - 32 bytes: ephemeral public key (compressed point)
/// - 32 bytes: detection tag
/// - Variable: encrypted core (asset_id + amount) with auth tag
/// - Variable: encrypted extension (counterparty address) with auth tag
pub fn serialize_compliance_ciphertext(ciphertext: &ComplianceCiphertext) -> Vec<u8> {
    ciphertext.to_bytes()
}
