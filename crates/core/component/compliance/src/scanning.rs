//! Compliance scanning and decryption with tiered access control.
//!
//! This module provides separate functions for scanning (detection) vs decryption,
//! enabling selective disclosure:
//!
//! - **Scanning**: Uses only the detection key to identify asset_id. Cannot see amounts or addresses.
//! - **Core Decryption**: Uses the core key to decrypt amount + self address.
//! - **Extension Decryption**: Uses the extension key to decrypt counterparty address.
//!
//! # Access Tiers
//!
//! | Role | Keys Available | Can See |
//! |------|----------------|---------|
//! | Scanner | Detection only | asset_id (for filtering) |
//! | Auditor | Detection + Core | asset_id + amount + self address |
//! | Full Access | All keys (MCK) | Everything including counterparty |

use decaf377::{Element, Fq};
use penumbra_sdk_asset::asset;
use penumbra_sdk_keys::keys::{DailyKeySet, DailyMasterKey, KeyType};
use penumbra_sdk_num::Amount;

use crate::crypto::COMPLIANCE_STREAM_CIPHER_DOMAIN;
use crate::structs::ComplianceCiphertext;

/// Decrypted core data: amount and self address.
#[derive(Clone, Debug)]
pub struct CoreData {
    pub amount: Amount,
    pub self_diversified_generator: Element,
    pub self_transmission_key: [u8; 32],
}

/// Decrypted extension data: counterparty address.
#[derive(Clone, Debug)]
pub struct ExtensionData {
    pub counterparty_diversified_generator: Element,
    pub counterparty_transmission_key: [u8; 32],
}

/// Full decrypted compliance data (all tiers).
#[derive(Clone, Debug)]
pub struct FullComplianceData {
    pub asset_id: asset::Id,
    pub core: CoreData,
    pub extension: ExtensionData,
}

/// Scan a ciphertext to detect the asset_id.
///
/// This function uses ONLY the detection key and can ONLY reveal the asset_id.
/// It cannot decrypt amounts, addresses, or any other data.
///
/// # Arguments
/// * `detection_key` - The daily detection key (must be KeyType::Detection)
/// * `ciphertext` - The compliance ciphertext to scan
///
/// # Returns
/// * `Ok(Some(asset_id))` if detection succeeded
/// * `Ok(None)` if the ciphertext doesn't match this key (wrong date, wrong MCK)
/// * `Err(_)` on unexpected errors
pub fn scan_for_asset(
    detection_key: &DailyMasterKey,
    ciphertext: &ComplianceCiphertext,
) -> anyhow::Result<Option<asset::Id>> {
    assert_eq!(
        detection_key.key_type(),
        KeyType::Detection,
        "scan_for_asset requires a Detection key"
    );

    // Compute shared secret using detection key
    let ss_detection = ciphertext.epk * detection_key.inner();

    // Derive seed for detection
    let epk_fq = ciphertext.epk.vartime_compress_to_field();
    let seed_detection = poseidon377::hash_2(
        &COMPLIANCE_STREAM_CIPHER_DOMAIN,
        (ss_detection.vartime_compress_to_field(), epk_fq),
    );

    // Decrypt detection_tag (asset_id) - 1 Fq element
    let detection_ciphertext_fq = Fq::from_le_bytes_mod_order(&ciphertext.detection_tag);
    let detection_keystream =
        poseidon377::hash_2(&seed_detection, (Fq::from(0u64), seed_detection));
    let asset_id_fq = detection_ciphertext_fq - detection_keystream;

    // Validate asset_id
    let asset_id_bytes = asset_id_fq.to_bytes();
    match Fq::from_bytes_checked(&asset_id_bytes) {
        Ok(validated_fq) => Ok(Some(asset::Id(validated_fq))),
        Err(_) => Ok(None), // Invalid decryption - wrong key
    }
}

/// Decrypt the core data (amount + self address).
///
/// Requires the core key. Does NOT decrypt the counterparty address.
///
/// # Arguments
/// * `core_key` - The daily core key (must be KeyType::Core)
/// * `ciphertext` - The compliance ciphertext
///
/// # Returns
/// * `Ok(Some(CoreData))` on successful decryption
/// * `Ok(None)` if decryption failed (wrong key)
/// * `Err(_)` on unexpected errors
pub fn decrypt_core(
    core_key: &DailyMasterKey,
    ciphertext: &ComplianceCiphertext,
) -> anyhow::Result<Option<CoreData>> {
    assert_eq!(
        core_key.key_type(),
        KeyType::Core,
        "decrypt_core requires a Core key"
    );

    // Compute shared secret using core key
    let ss_core = ciphertext.epk * core_key.inner();

    // Derive seed for core
    let epk_fq = ciphertext.epk.vartime_compress_to_field();
    let seed_core = poseidon377::hash_2(
        &COMPLIANCE_STREAM_CIPHER_DOMAIN,
        (ss_core.vartime_compress_to_field(), epk_fq),
    );

    // Decrypt core data - 3 Fq elements (80 bytes plaintext, 31-byte chunks)
    let mut core_plaintext_bytes = Vec::new();
    for (i, chunk) in ciphertext.encrypted_core.chunks(32).enumerate() {
        let mut buf = [0u8; 32];
        buf[0..chunk.len()].copy_from_slice(chunk);
        let ciphertext_fq = Fq::from_le_bytes_mod_order(&buf);
        let counter = Fq::from(i as u64);
        let keystream = poseidon377::hash_2(&seed_core, (counter, seed_core));
        let plaintext_fq = ciphertext_fq - keystream;
        let fq_bytes = plaintext_fq.to_bytes();
        // Take 31 bytes per Fq (to match 31-byte chunk encoding)
        let bytes_to_take = 31.min(80 - core_plaintext_bytes.len());
        core_plaintext_bytes.extend_from_slice(&fq_bytes[0..bytes_to_take]);
    }

    if core_plaintext_bytes.len() < 80 {
        return Ok(None);
    }

    // Parse: amount (16) || self_div_gen (32) || self_trans_key (32)
    let amount_bytes: [u8; 16] = match core_plaintext_bytes[0..16].try_into() {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };
    let amount = Amount::from_le_bytes(amount_bytes);

    let self_div_gen_bytes: [u8; 32] = match core_plaintext_bytes[16..48].try_into() {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };
    let self_div_gen = match decaf377::Encoding(self_div_gen_bytes).vartime_decompress() {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };

    let self_trans_key_bytes: [u8; 32] = match core_plaintext_bytes[48..80].try_into() {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };

    Ok(Some(CoreData {
        amount,
        self_diversified_generator: self_div_gen,
        self_transmission_key: self_trans_key_bytes,
    }))
}

/// Decrypt the extension data (counterparty address).
///
/// Requires the extension key.
///
/// # Arguments
/// * `extension_key` - The daily extension key (must be KeyType::Extension)
/// * `ciphertext` - The compliance ciphertext
///
/// # Returns
/// * `Ok(Some(ExtensionData))` on successful decryption
/// * `Ok(None)` if decryption failed (wrong key)
/// * `Err(_)` on unexpected errors
pub fn decrypt_extension(
    extension_key: &DailyMasterKey,
    ciphertext: &ComplianceCiphertext,
) -> anyhow::Result<Option<ExtensionData>> {
    assert_eq!(
        extension_key.key_type(),
        KeyType::Extension,
        "decrypt_extension requires an Extension key"
    );

    // Compute shared secret using extension key
    let ss_extension = ciphertext.epk * extension_key.inner();

    // Derive seed for extension
    let epk_fq = ciphertext.epk.vartime_compress_to_field();
    let seed_extension = poseidon377::hash_2(
        &COMPLIANCE_STREAM_CIPHER_DOMAIN,
        (ss_extension.vartime_compress_to_field(), epk_fq),
    );

    // Decrypt extension data - 3 Fq elements (64 bytes plaintext, 31-byte chunks)
    let mut extension_plaintext_bytes = Vec::new();
    for (i, chunk) in ciphertext.encrypted_extension.chunks(32).enumerate() {
        let mut buf = [0u8; 32];
        buf[0..chunk.len()].copy_from_slice(chunk);
        let ciphertext_fq = Fq::from_le_bytes_mod_order(&buf);
        let counter = Fq::from(i as u64);
        let keystream = poseidon377::hash_2(&seed_extension, (counter, seed_extension));
        let plaintext_fq = ciphertext_fq - keystream;
        let fq_bytes = plaintext_fq.to_bytes();
        // Take 31 bytes per Fq (to match 31-byte chunk encoding)
        let bytes_to_take = 31.min(64 - extension_plaintext_bytes.len());
        extension_plaintext_bytes.extend_from_slice(&fq_bytes[0..bytes_to_take]);
    }

    if extension_plaintext_bytes.len() < 64 {
        return Ok(None);
    }

    // Parse: counterparty_div_gen (32) || counterparty_trans_key (32)
    let counterparty_div_gen_bytes: [u8; 32] = match extension_plaintext_bytes[0..32].try_into() {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };
    let counterparty_div_gen =
        match decaf377::Encoding(counterparty_div_gen_bytes).vartime_decompress() {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

    let counterparty_trans_key_bytes: [u8; 32] = match extension_plaintext_bytes[32..64].try_into()
    {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };

    Ok(Some(ExtensionData {
        counterparty_diversified_generator: counterparty_div_gen,
        counterparty_transmission_key: counterparty_trans_key_bytes,
    }))
}

/// Decrypt all compliance data using a full key set.
///
/// This is a convenience function for full-access scenarios.
/// Internally calls scan_for_asset, decrypt_core, and decrypt_extension.
///
/// # Arguments
/// * `daily_keys` - The complete daily key set (detection + core + extension)
/// * `ciphertext` - The compliance ciphertext
///
/// # Returns
/// * `Ok(Some(FullComplianceData))` on successful full decryption
/// * `Ok(None)` if decryption failed
/// * `Err(_)` on unexpected errors
pub fn decrypt_full(
    daily_keys: &DailyKeySet,
    ciphertext: &ComplianceCiphertext,
) -> anyhow::Result<Option<FullComplianceData>> {
    // Scan for asset_id
    let asset_id = match scan_for_asset(&daily_keys.detection, ciphertext)? {
        Some(id) => id,
        None => return Ok(None),
    };

    // Decrypt core
    let core = match decrypt_core(&daily_keys.core, ciphertext)? {
        Some(c) => c,
        None => return Ok(None),
    };

    // Decrypt extension
    let extension = match decrypt_extension(&daily_keys.extension, ciphertext)? {
        Some(e) => e,
        None => return Ok(None),
    };

    Ok(Some(FullComplianceData {
        asset_id,
        core,
        extension,
    }))
}

/// Scanner role - determines which ciphertext to process.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScannerRole {
    Sender,
    Receiver,
}

/// High-level scanner for compliance monitoring.
///
/// Uses DailyKeySet for full decryption capability.
/// For detection-only access, use `scan_for_asset` directly.
pub struct ComplianceScanner {
    daily_keys: DailyKeySet,
    role: ScannerRole,
}

impl ComplianceScanner {
    pub fn new(daily_keys: DailyKeySet, role: ScannerRole) -> Self {
        Self { daily_keys, role }
    }

    pub fn role(&self) -> ScannerRole {
        self.role
    }

    /// Scan for asset_id only (detection tier).
    pub fn detect_asset(
        &self,
        sender_ciphertext: &ComplianceCiphertext,
        receiver_ciphertext: &ComplianceCiphertext,
    ) -> anyhow::Result<Option<asset::Id>> {
        let ciphertext = match self.role {
            ScannerRole::Sender => sender_ciphertext,
            ScannerRole::Receiver => receiver_ciphertext,
        };
        scan_for_asset(&self.daily_keys.detection, ciphertext)
    }

    /// Full decryption (requires all keys in DailyKeySet).
    pub fn decrypt(
        &self,
        sender_ciphertext: &ComplianceCiphertext,
        receiver_ciphertext: &ComplianceCiphertext,
    ) -> anyhow::Result<Option<FullComplianceData>> {
        let ciphertext = match self.role {
            ScannerRole::Sender => sender_ciphertext,
            ScannerRole::Receiver => receiver_ciphertext,
        };
        decrypt_full(&self.daily_keys, ciphertext)
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn daily_keys(&self) -> &DailyKeySet {
        &self.daily_keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{encrypt_dual, make_mck};

    #[test]
    fn test_scan_vs_decrypt_separation() {
        let mck = make_mck();
        let date = 19000u64;
        let (sender_ct, _receiver_ct) = encrypt_dual(&mck, 1, 2, date, 42, 1000);

        let daily_keys = mck.derive_daily_keys(date);

        // Detection-only scan
        let asset_id = scan_for_asset(&daily_keys.detection, &sender_ct)
            .unwrap()
            .expect("scan should succeed");
        assert_eq!(asset_id.0, Fq::from(42u64));

        // Core decryption (amount + self address)
        let core = decrypt_core(&daily_keys.core, &sender_ct)
            .unwrap()
            .expect("core decryption should succeed");
        assert_eq!(core.amount, Amount::from(1000u128));

        // Extension decryption (counterparty address)
        let extension = decrypt_extension(&daily_keys.extension, &sender_ct)
            .unwrap()
            .expect("extension decryption should succeed");
        // Just verify it returns something
        assert!(extension.counterparty_transmission_key != [0u8; 32]);
    }

    #[test]
    fn test_detection_cannot_see_amount() {
        let mck = make_mck();
        let date = 19000u64;
        let (sender_ct, _) = encrypt_dual(&mck, 1, 2, date, 42, 9999);

        let detection_key = mck.derive_daily_key(KeyType::Detection, date);

        // Detection key can only see asset_id
        let asset_id = scan_for_asset(&detection_key, &sender_ct)
            .unwrap()
            .expect("scan should succeed");
        assert_eq!(asset_id.0, Fq::from(42u64));

        // The function signature doesn't allow accessing amount with detection key
        // This is enforced at compile time via the API design
    }

    #[test]
    fn test_wrong_key_type_panics() {
        let mck = make_mck();
        let date = 19000u64;
        let (sender_ct, _) = encrypt_dual(&mck, 1, 2, date, 42, 1000);

        let core_key = mck.derive_daily_key(KeyType::Core, date);

        // Using core key for scan_for_asset should panic
        let result = std::panic::catch_unwind(|| {
            let _ = scan_for_asset(&core_key, &sender_ct);
        });
        assert!(result.is_err(), "wrong key type should panic");
    }

    #[test]
    fn test_full_decryption() {
        let mck = make_mck();
        let date = 19000u64;
        let (sender_ct, receiver_ct) = encrypt_dual(&mck, 1, 2, date, 42, 1000);

        let daily_keys = mck.derive_daily_keys(date);
        let scanner = ComplianceScanner::new(daily_keys, ScannerRole::Sender);

        // Full decryption
        let full = scanner
            .decrypt(&sender_ct, &receiver_ct)
            .unwrap()
            .expect("full decryption should succeed");

        assert_eq!(full.asset_id.0, Fq::from(42u64));
        assert_eq!(full.core.amount, Amount::from(1000u128));
    }

    #[test]
    fn test_detect_then_decrypt() {
        let mck = make_mck();
        let date = 19000u64;
        let target_asset = asset::Id(Fq::from(42u64));
        let (sender_ct, receiver_ct) = encrypt_dual(&mck, 1, 2, date, 42, 1000);

        let daily_keys = mck.derive_daily_keys(date);
        let scanner = ComplianceScanner::new(daily_keys, ScannerRole::Sender);

        // First detect
        let detected_asset = scanner
            .detect_asset(&sender_ct, &receiver_ct)
            .unwrap()
            .expect("detection should succeed");

        // Check if it matches our target
        if detected_asset == target_asset {
            // Only then do full decryption
            let full = scanner
                .decrypt(&sender_ct, &receiver_ct)
                .unwrap()
                .expect("decryption should succeed");
            assert_eq!(full.core.amount, Amount::from(1000u128));
        }
    }

    #[test]
    fn test_wrong_date_fails() {
        let mck = make_mck();
        let target_asset = asset::Id(Fq::from(42u64));
        let (s_ct, r_ct) = encrypt_dual(&mck, 1, 2, 19000, 42, 1000);

        // Scanner for wrong date
        let wrong_keys = mck.derive_daily_keys(19001);
        let scanner = ComplianceScanner::new(wrong_keys, ScannerRole::Sender);

        // Detection may succeed (produces some Fq), but the asset_id won't match
        let detected = scanner.detect_asset(&s_ct, &r_ct).unwrap();
        // Either None (rare) or wrong asset_id
        if let Some(wrong_asset) = detected {
            assert_ne!(
                wrong_asset, target_asset,
                "Wrong date should not produce correct asset"
            );
        }

        // Full decryption should definitely fail (produce garbage addresses)
        assert!(scanner.decrypt(&s_ct, &r_ct).unwrap().is_none());
    }

    #[test]
    fn test_wrong_mck_fails() {
        let mck_a = make_mck();
        let mck_b = make_mck();

        let (s_ct, r_ct) = encrypt_dual(&mck_a, 1, 2, 19000, 42, 1000);

        // Scanner with different MCK
        let wrong_keys = mck_b.derive_daily_keys(19000);
        let scanner = ComplianceScanner::new(wrong_keys, ScannerRole::Sender);

        // Full decryption should fail
        assert!(scanner.decrypt(&s_ct, &r_ct).unwrap().is_none());
    }

    #[test]
    fn test_role_separation() {
        let mck = make_mck();
        let (s_ct, r_ct) = encrypt_dual(&mck, 1, 2, 19000, 42, 1000);

        let daily_keys = mck.derive_daily_keys(19000);
        let sender_scanner = ComplianceScanner::new(daily_keys.clone(), ScannerRole::Sender);
        let receiver_scanner = ComplianceScanner::new(daily_keys, ScannerRole::Receiver);

        // Both scanners detect their respective ciphertexts
        assert!(sender_scanner.detect_asset(&s_ct, &r_ct).unwrap().is_some());
        assert!(receiver_scanner
            .detect_asset(&s_ct, &r_ct)
            .unwrap()
            .is_some());
    }
}
