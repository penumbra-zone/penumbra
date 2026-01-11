//! Privacy-preserving detection of compliance ciphertexts.
//!
//! This module provides functions for detecting transactions involving a specific
//! regulated asset by decrypting ONLY the detection_tag (first 32 bytes) of
//! compliance ciphertexts.
//!
//! # Security Model
//!
//! The detector uses a `DailyMasterKey` (detection key) which provides LIMITED
//! access to ciphertext data:
//! - ✅ CAN detect asset_id
//! - ❌ CANNOT decrypt amounts
//! - ❌ CANNOT decrypt addresses
//! - ❌ CANNOT access any other transaction metadata
//!
//! For full decryption, use `decrypt_with_mck()` with proper legal authorization.

use anyhow::Result;
use penumbra_sdk_asset::asset;
use penumbra_sdk_keys::keys::DailyMasterKey;
use penumbra_sdk_proto::core::transaction::v1::Transaction as ProtoTransaction;

use super::sync::extract_compliance_ciphertexts;
use crate::structs::ComplianceCiphertext;

/// Information about a detected transaction containing the target asset.
///
/// This struct contains only the ciphertext and metadata - NOT the decrypted data.
/// The ciphertext can later be fully decrypted using `decrypt_with_mck()` if needed.
#[derive(Clone, Debug)]
pub struct DetectedCiphertext {
    /// The block height where this ciphertext was found.
    pub height: u64,

    /// The transaction index within the block.
    pub tx_index: usize,

    /// The action index within the transaction.
    pub action_index: usize,

    /// The compliance ciphertext (not decrypted, just detected).
    pub ciphertext: ComplianceCiphertext,
}

/// Scan a transaction for ciphertexts matching a specific asset.
///
/// Decrypts only the detection_tag (first 32 bytes) to check asset_id,
/// then calls the callback for each match. Returns the number of matches found.
pub fn scan_transaction<F>(
    detection_key: &DailyMasterKey,
    target_asset_id: asset::Id,
    tx: &ProtoTransaction,
    height: u64,
    tx_index: usize,
    mut callback: F,
) -> Result<usize>
where
    F: FnMut(DetectedCiphertext) -> Result<()>,
{
    // Use shared extraction logic
    let ciphertexts = extract_compliance_ciphertexts(tx)?;

    let mut matches = 0;

    for (action_index, ciphertext_bytes, _) in ciphertexts {
        // Parse ciphertext
        let ciphertext = match ComplianceCiphertext::from_bytes(&ciphertext_bytes) {
            Ok(ct) => ct,
            Err(_) => continue, // Skip malformed ciphertexts
        };

        // Try to detect the asset_id using the detection key
        // This only decrypts the first 32 bytes (detection_tag)
        let detected_asset_id =
            match detection_key.try_detect_asset(&ciphertext.epk, &ciphertext.detection_tag) {
                Ok(asset_id) => asset_id,
                Err(_) => continue, // Detection failed (wrong key or corrupted data)
            };

        // Check if this matches our target asset
        if detected_asset_id == target_asset_id {
            matches += 1;

            // Call the user's callback with the detected ciphertext
            callback(DetectedCiphertext {
                height,
                tx_index,
                action_index,
                ciphertext,
            })?;
        }
    }

    Ok(matches)
}

/// Batch scan multiple transactions for a specific asset.
pub fn scan_transactions<F, I>(
    detection_key: &DailyMasterKey,
    target_asset_id: asset::Id,
    transactions: I,
    mut callback: F,
) -> Result<usize>
where
    F: FnMut(DetectedCiphertext) -> Result<()>,
    I: IntoIterator<Item = (u64, usize, ProtoTransaction)>,
{
    let mut total_matches = 0;

    for (height, tx_index, tx) in transactions {
        let matches = scan_transaction(
            detection_key,
            target_asset_id,
            &tx,
            height,
            tx_index,
            &mut callback,
        )?;
        total_matches += matches;
    }

    Ok(total_matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::encrypt_compliance_details;
    use crate::test_helpers::{make_mck, make_wallet};
    use penumbra_sdk_num::Amount;
    use penumbra_sdk_proto::core::component::shielded_pool::v1::{Output, OutputBody};
    use penumbra_sdk_proto::core::transaction::v1::{
        action::Action, Action as ActionProto, TransactionBody,
    };

    #[test]
    fn test_scan_transaction_finds_matching_asset() {
        let mut rng = rand_core::OsRng;

        // Setup: Create MCK and derive detection key
        let mck = make_mck();
        let date = 19000u64;
        let detection_key = mck.derive_daily_key(penumbra_sdk_keys::keys::KeyType::Detection, date);

        // Create wallet key and address
        let (ack, address) = make_wallet(&mck, 11);

        // Create test asset and encrypt
        let target_asset = asset::Id(decaf377::Fq::from(9999u64));
        let amount = Amount::from(123u128);

        let (ciphertext, _) = encrypt_compliance_details(
            &mut rng,
            &ack,
            &address,
            date,
            target_asset,
            amount,
            address.clone(),
        )
        .expect("encryption should succeed");

        // Create a transaction with this ciphertext
        let tx = ProtoTransaction {
            body: Some(TransactionBody {
                actions: vec![ActionProto {
                    action: Some(Action::Output(Output {
                        body: Some(OutputBody {
                            compliance_ciphertext: ciphertext.to_bytes(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    })),
                }],
                ..Default::default()
            }),
            ..Default::default()
        };

        // Scan the transaction
        let mut detected_count = 0;
        let result = scan_transaction(&detection_key, target_asset, &tx, 100, 0, |detected| {
            detected_count += 1;
            assert_eq!(detected.height, 100);
            assert_eq!(detected.tx_index, 0);
            assert_eq!(detected.action_index, 0);
            Ok(())
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1, "Should find exactly 1 match");
        assert_eq!(detected_count, 1, "Callback should be called once");
    }

    #[test]
    fn test_scan_transaction_ignores_different_asset() {
        let mut rng = rand_core::OsRng;

        // Setup
        let mck = make_mck();
        let date = 19001u64;
        let detection_key = mck.derive_daily_key(penumbra_sdk_keys::keys::KeyType::Detection, date);

        let (ack, address) = make_wallet(&mck, 12);

        // Encrypt with asset_id = 5555
        let encrypted_asset = asset::Id(decaf377::Fq::from(5555u64));
        let amount = Amount::from(999u128);

        let (ciphertext, _) = encrypt_compliance_details(
            &mut rng,
            &ack,
            &address,
            date,
            encrypted_asset,
            amount,
            address.clone(),
        )
        .expect("encryption should succeed");

        // Create transaction
        let tx = ProtoTransaction {
            body: Some(TransactionBody {
                actions: vec![ActionProto {
                    action: Some(Action::Output(Output {
                        body: Some(OutputBody {
                            compliance_ciphertext: ciphertext.to_bytes(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    })),
                }],
                ..Default::default()
            }),
            ..Default::default()
        };

        // Scan for different asset (7777)
        let target_asset = asset::Id(decaf377::Fq::from(7777u64));
        let mut callback_called = false;

        let result = scan_transaction(&detection_key, target_asset, &tx, 200, 0, |_| {
            callback_called = true;
            Ok(())
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0, "Should find no matches");
        assert!(!callback_called, "Callback should not be called");
    }

    #[test]
    fn test_scan_transaction_with_wrong_detection_key() {
        let mut rng = rand_core::OsRng;

        // Create two different MCKs
        let mck1 = make_mck();
        let mck2 = make_mck();

        let date = 19002u64;
        let detection_key2 =
            mck2.derive_daily_key(penumbra_sdk_keys::keys::KeyType::Detection, date);

        // Encrypt with mck1
        let (ack1, address) = make_wallet(&mck1, 13);

        let target_asset = asset::Id(decaf377::Fq::from(1111u64));
        let amount = Amount::from(456u128);

        let (ciphertext, _) = encrypt_compliance_details(
            &mut rng,
            &ack1,
            &address,
            date,
            target_asset,
            amount,
            address.clone(),
        )
        .expect("encryption should succeed");

        // Create transaction
        let tx = ProtoTransaction {
            body: Some(TransactionBody {
                actions: vec![ActionProto {
                    action: Some(Action::Output(Output {
                        body: Some(OutputBody {
                            compliance_ciphertext: ciphertext.to_bytes(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    })),
                }],
                ..Default::default()
            }),
            ..Default::default()
        };

        // Try to detect with wrong key (detection_key2)
        let mut callback_called = false;

        let result = scan_transaction(&detection_key2, target_asset, &tx, 300, 0, |_| {
            callback_called = true;
            Ok(())
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0, "Wrong key should find no matches");
        assert!(
            !callback_called,
            "Callback should not be called with wrong key"
        );
    }

    #[test]
    fn test_scan_transactions_batch() {
        let mut rng = rand_core::OsRng;

        let mck = make_mck();
        let date = 19003u64;
        let detection_key = mck.derive_daily_key(penumbra_sdk_keys::keys::KeyType::Detection, date);

        let (ack, address) = make_wallet(&mck, 14);

        let target_asset = asset::Id(decaf377::Fq::from(3333u64));

        // Create 3 transactions, 2 with target asset, 1 with different asset
        let mut txs = Vec::new();

        for i in 0..3 {
            let asset_id = if i == 1 {
                asset::Id(decaf377::Fq::from(4444u64)) // Different asset
            } else {
                target_asset
            };

            let (ciphertext, _) = encrypt_compliance_details(
                &mut rng,
                &ack,
                &address,
                date,
                asset_id,
                Amount::from((100 + i) as u128),
                address.clone(),
            )
            .expect("encryption should succeed");

            let tx = ProtoTransaction {
                body: Some(TransactionBody {
                    actions: vec![ActionProto {
                        action: Some(Action::Output(Output {
                            body: Some(OutputBody {
                                compliance_ciphertext: ciphertext.to_bytes(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        })),
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            };

            txs.push((1000 + i as u64, i, tx));
        }

        // Scan all transactions
        let mut detected = Vec::new();
        let result = scan_transactions(&detection_key, target_asset, txs, |d| {
            detected.push(d);
            Ok(())
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2, "Should find 2 matches (tx 0 and tx 2)");
        assert_eq!(detected.len(), 2);
        assert_eq!(detected[0].height, 1000);
        assert_eq!(detected[1].height, 1002);
    }
}
