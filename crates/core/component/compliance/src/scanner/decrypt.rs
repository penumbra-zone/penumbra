//! Full decryption of compliance ciphertexts.
//!
//! With tiered key types, full decryption requires all three daily keys
//! (Detection, Core, Extension). For selective disclosure, use the
//! individual decrypt functions in crypto.rs.

use anyhow::{Context, Result};
use penumbra_sdk_keys::keys::{DailyKeySet, MasterComplianceKey};

use crate::crypto::{decrypt_compliance_details, DecryptedComplianceData};
use crate::structs::ComplianceCiphertext;

/// Decrypt using MasterComplianceKey (derives all daily keys internally).
///
/// This derives all three key types (Detection, Core, Extension) and
/// decrypts the complete ciphertext.
pub fn decrypt_with_mck(
    mck: &MasterComplianceKey,
    date: u64,
    ciphertext: &ComplianceCiphertext,
) -> Result<DecryptedComplianceData> {
    let daily_keys = mck.derive_daily_keys(date);
    decrypt_with_daily_keys(&daily_keys, ciphertext)
}

/// Decrypt using pre-derived DailyKeySet.
///
/// Preferred for production - auditor receives daily keys from issuer.
/// Requires all three key types for full decryption.
pub fn decrypt_with_daily_keys(
    daily_keys: &DailyKeySet,
    ciphertext: &ComplianceCiphertext,
) -> Result<DecryptedComplianceData> {
    let epk = ciphertext.epk;

    // Compute shared secrets for each key type
    let ss_detection = epk * daily_keys.detection.inner();
    let ss_core = epk * daily_keys.core.inner();
    let ss_extension = epk * daily_keys.extension.inner();

    decrypt_compliance_details(&ss_detection, &ss_core, &ss_extension, &epk, ciphertext)
        .context("failed to decrypt compliance ciphertext with daily keys")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::encrypt_compliance_details;
    use crate::test_helpers::{make_address, make_mck, make_wallet};
    use penumbra_sdk_asset::asset;
    use penumbra_sdk_num::Amount;

    #[test]
    fn test_decrypt_with_mck_roundtrip() {
        let mut rng = rand_core::OsRng;

        // Setup: Create a master key and derive a wallet key
        let mck = make_mck();
        let (ack, self_address) = make_wallet(&mck, 7);

        // Counterparty address
        let counterparty_address = make_address(8);

        // Test data
        let date = 19000u64;
        let asset_id = asset::Id(decaf377::Fq::from(12345u64));
        let amount = Amount::from(999u128);

        // Encrypt
        let (ciphertext, _ephemeral) = encrypt_compliance_details(
            &mut rng,
            &ack,
            &self_address,
            date,
            asset_id,
            amount,
            counterparty_address.clone(),
        )
        .expect("encryption should succeed");

        // Decrypt using MCK
        let decrypted = decrypt_with_mck(&mck, date, &ciphertext)
            .expect("decryption should succeed with correct MCK");

        // Verify all fields match
        assert_eq!(decrypted.asset_id, asset_id, "asset_id should match");
        assert_eq!(decrypted.amount, amount, "amount should match");

        // Verify self address components
        assert_eq!(
            decrypted.self_diversified_generator,
            *self_address.diversified_generator(),
            "self diversified generator should match"
        );
        assert_eq!(
            decrypted.self_transmission_key,
            self_address.transmission_key().0,
            "self transmission key should match"
        );

        // Verify counterparty address components
        assert_eq!(
            decrypted.counterparty_diversified_generator,
            *counterparty_address.diversified_generator(),
            "counterparty diversified generator should match"
        );
        assert_eq!(
            decrypted.counterparty_transmission_key,
            counterparty_address.transmission_key().0,
            "counterparty transmission key should match"
        );
    }

    #[test]
    fn test_decrypt_with_wrong_mck_fails() {
        let mut rng = rand_core::OsRng;

        // Create two different MCKs
        let mck1 = make_mck();
        let mck2 = make_mck();

        let (ack1, self_address) = make_wallet(&mck1, 9);

        // Simple counterparty
        let counterparty_address = self_address.clone();

        // Encrypt with mck1
        let date = 19001u64;
        let asset_id = asset::Id(decaf377::Fq::from(54321u64));
        let amount = Amount::from(777u128);

        let (ciphertext, _) = encrypt_compliance_details(
            &mut rng,
            &ack1,
            &self_address,
            date,
            asset_id,
            amount,
            counterparty_address,
        )
        .expect("encryption should succeed");

        // Try to decrypt with mck2 (wrong key)
        let result = decrypt_with_mck(&mck2, date, &ciphertext);

        // Decryption will succeed but produce garbage data
        // (Encryption is malleable - any key will "decrypt" to something)
        if let Ok(decrypted) = result {
            // The decrypted data should NOT match the original
            assert_ne!(
                decrypted.asset_id, asset_id,
                "wrong MCK should not decrypt to correct asset_id"
            );
            assert_ne!(
                decrypted.amount, amount,
                "wrong MCK should not decrypt to correct amount"
            );
        }
    }

    #[test]
    fn test_decrypt_with_wrong_date_fails() {
        let mut rng = rand_core::OsRng;

        let mck = make_mck();
        let (ack, self_address) = make_wallet(&mck, 10);

        let counterparty_address = self_address.clone();

        // Encrypt for date 19000
        let encryption_date = 19000u64;
        let asset_id = asset::Id(decaf377::Fq::from(11111u64));
        let amount = Amount::from(555u128);

        let (ciphertext, _) = encrypt_compliance_details(
            &mut rng,
            &ack,
            &self_address,
            encryption_date,
            asset_id,
            amount,
            counterparty_address,
        )
        .expect("encryption should succeed");

        // Try to decrypt with wrong date (19001 instead of 19000)
        let wrong_date = 19001u64;
        let result = decrypt_with_mck(&mck, wrong_date, &ciphertext);

        // Decryption will succeed but produce wrong data
        if let Ok(decrypted) = result {
            assert_ne!(
                decrypted.asset_id, asset_id,
                "wrong date should not decrypt to correct asset_id"
            );
            assert_ne!(
                decrypted.amount, amount,
                "wrong date should not decrypt to correct amount"
            );
        }
    }
}
