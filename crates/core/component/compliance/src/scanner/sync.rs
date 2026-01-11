//! Compliance scanning for regulated asset transfers.

use anyhow::Result;
use decaf377::Element;
use penumbra_sdk_asset::asset;
use penumbra_sdk_keys::keys::{DailyKeySet, MasterComplianceKey};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::core::component::sct::v1::Nullifier;
use penumbra_sdk_proto::core::transaction::v1::Transaction as ProtoTransaction;
use serde::{Deserialize, Serialize};

use crate::crypto::DecryptedComplianceData;
use crate::structs::ComplianceCiphertext;

use super::decrypt::{decrypt_with_daily_keys, decrypt_with_mck};

/// Partial address from decrypted compliance data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartialAddress {
    pub diversified_generator: [u8; 32],
    pub transmission_key: [u8; 32],
}

impl PartialAddress {
    pub fn new(diversified_generator: Element, transmission_key: [u8; 32]) -> Self {
        Self {
            diversified_generator: diversified_generator.vartime_compress().0,
            transmission_key,
        }
    }
}

impl std::fmt::Display for PartialAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "g_d:{:02x}{:02x}..{:02x}{:02x},pk:{:02x}{:02x}..{:02x}{:02x}",
            self.diversified_generator[0],
            self.diversified_generator[1],
            self.diversified_generator[30],
            self.diversified_generator[31],
            self.transmission_key[0],
            self.transmission_key[1],
            self.transmission_key[30],
            self.transmission_key[31],
        )
    }
}

/// A detected transfer of a regulated asset.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectedTransfer {
    pub height: u64,
    pub action_index: usize,
    pub asset_id: asset::Id,
    pub amount: Amount,
    pub self_address: PartialAddress,
    pub counterparty_address: PartialAddress,
    pub nullifier: Option<Nullifier>,
}

impl std::fmt::Display for DetectedTransfer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "height={}, action={}, asset={}, amount={}, self={}, counterparty={}",
            self.height,
            self.action_index,
            self.asset_id,
            self.amount,
            self.self_address,
            self.counterparty_address,
        )
    }
}

/// Extract compliance ciphertexts from a transaction.
/// Returns (action_index, ciphertext_bytes, is_spend) tuples.
pub fn extract_compliance_ciphertexts(
    tx: &ProtoTransaction,
) -> Result<Vec<(usize, Vec<u8>, bool)>> {
    use penumbra_sdk_proto::core::transaction::v1::action::Action;

    let actions = match &tx.body {
        Some(body) => &body.actions,
        None => return Ok(vec![]),
    };

    let mut results = Vec::new();
    for (idx, action) in actions.iter().enumerate() {
        match &action.action {
            Some(Action::Output(output)) => {
                if let Some(body) = &output.body {
                    if !body.compliance_ciphertext.is_empty() {
                        results.push((idx, body.compliance_ciphertext.clone(), false));
                    }
                }
            }
            Some(Action::Spend(spend)) => {
                if let Some(body) = &spend.body {
                    if !body.compliance_ciphertext.is_empty() {
                        results.push((idx, body.compliance_ciphertext.clone(), true));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(results)
}

/// Core scan logic - decrypts ciphertexts and builds DetectedTransfer structs.
fn scan_transaction_core<F>(
    tx: &ProtoTransaction,
    height: u64,
    mut decrypt_fn: F,
    target_asset_id: Option<asset::Id>,
) -> Result<Vec<DetectedTransfer>>
where
    F: FnMut(&ComplianceCiphertext) -> Result<DecryptedComplianceData>,
{
    let ciphertexts = extract_compliance_ciphertexts(tx)?;
    let mut detected = Vec::new();

    for (action_index, ciphertext_bytes, _) in ciphertexts {
        let ciphertext = match ComplianceCiphertext::from_bytes(&ciphertext_bytes) {
            Ok(ct) => ct,
            Err(_) => continue,
        };

        let decrypted = match decrypt_fn(&ciphertext) {
            Ok(data) => data,
            Err(_) => continue,
        };

        if let Some(target) = target_asset_id {
            if decrypted.asset_id != target {
                continue;
            }
        }

        detected.push(DetectedTransfer {
            height,
            action_index,
            asset_id: decrypted.asset_id,
            amount: decrypted.amount,
            self_address: PartialAddress::new(
                decrypted.self_diversified_generator,
                decrypted.self_transmission_key,
            ),
            counterparty_address: PartialAddress::new(
                decrypted.counterparty_diversified_generator,
                decrypted.counterparty_transmission_key,
            ),
            nullifier: None,
        });
    }

    Ok(detected)
}

/// Scan a transaction using MasterComplianceKey.
pub fn scan_transaction_for_compliance(
    tx: &ProtoTransaction,
    height: u64,
    mck: &MasterComplianceKey,
    date: u64,
    target_asset_id: Option<asset::Id>,
) -> Result<Vec<DetectedTransfer>> {
    scan_transaction_core(
        tx,
        height,
        |ct| decrypt_with_mck(mck, date, ct),
        target_asset_id,
    )
}

/// Scan multiple transactions using MasterComplianceKey.
pub fn scan_transactions_for_compliance<I>(
    transactions: I,
    mck: &MasterComplianceKey,
    date: u64,
    target_asset_id: Option<asset::Id>,
) -> Result<Vec<DetectedTransfer>>
where
    I: IntoIterator<Item = (u64, ProtoTransaction)>,
{
    let mut all = Vec::new();
    for (height, tx) in transactions {
        all.extend(scan_transaction_for_compliance(
            &tx,
            height,
            mck,
            date,
            target_asset_id,
        )?);
    }
    Ok(all)
}

/// Scan a transaction using pre-derived DailyKeySet.
/// Preferred for production - auditor receives daily keys from issuer.
/// Requires all three key types for full decryption.
pub fn scan_transaction_for_compliance_with_daily_keys(
    tx: &ProtoTransaction,
    height: u64,
    daily_keys: &DailyKeySet,
    target_asset_id: Option<asset::Id>,
) -> Result<Vec<DetectedTransfer>> {
    scan_transaction_core(
        tx,
        height,
        |ct| decrypt_with_daily_keys(daily_keys, ct),
        target_asset_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::encrypt_compliance_details;
    use crate::test_helpers::{make_address, make_mck};
    use penumbra_sdk_num::Amount;
    use penumbra_sdk_proto::core::component::shielded_pool::v1::{Output, OutputBody};
    use penumbra_sdk_proto::core::transaction::v1::{
        action::Action, Action as ActionProto, TransactionBody,
    };

    #[test]
    fn test_extract_empty() {
        let tx = ProtoTransaction {
            body: Some(TransactionBody {
                actions: vec![],
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(extract_compliance_ciphertexts(&tx).unwrap().len(), 0);
    }

    #[test]
    fn test_extract_output() {
        let tx = ProtoTransaction {
            body: Some(TransactionBody {
                actions: vec![ActionProto {
                    action: Some(Action::Output(Output {
                        body: Some(OutputBody {
                            compliance_ciphertext: vec![1, 2, 3],
                            ..Default::default()
                        }),
                        ..Default::default()
                    })),
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let results = extract_compliance_ciphertexts(&tx).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], (0, vec![1, 2, 3], false));
    }

    #[test]
    fn test_scan_roundtrip() {
        let mut rng = rand_core::OsRng;
        let mck = make_mck();
        let date = 19000u64;

        let self_address = make_address(11);
        let counterparty_address = make_address(22);
        let ack = mck.derive_address_key(self_address.diversifier());

        let asset_id = asset::Id(decaf377::Fq::from(12345u64));
        let amount = Amount::from(999u128);

        let (ciphertext, _) = encrypt_compliance_details(
            &mut rng,
            &ack,
            &self_address,
            date,
            asset_id,
            amount,
            counterparty_address.clone(),
        )
        .unwrap();

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

        let detected = scan_transaction_for_compliance(&tx, 100, &mck, date, None).unwrap();
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].asset_id, asset_id);
        assert_eq!(detected[0].amount, amount);
    }

    #[test]
    fn test_scan_filters_by_asset() {
        let mut rng = rand_core::OsRng;
        let mck = make_mck();
        let date = 19001u64;

        let self_address = make_address(33);
        let counterparty_address = make_address(44);
        let ack = mck.derive_address_key(self_address.diversifier());

        let asset_a = asset::Id(decaf377::Fq::from(1111u64));
        let asset_b = asset::Id(decaf377::Fq::from(2222u64));
        let amount = Amount::from(500u128);

        let (ct_a, _) = encrypt_compliance_details(
            &mut rng,
            &ack,
            &self_address,
            date,
            asset_a,
            amount,
            counterparty_address.clone(),
        )
        .unwrap();
        let (ct_b, _) = encrypt_compliance_details(
            &mut rng,
            &ack,
            &self_address,
            date,
            asset_b,
            amount,
            counterparty_address.clone(),
        )
        .unwrap();

        let tx = ProtoTransaction {
            body: Some(TransactionBody {
                actions: vec![
                    ActionProto {
                        action: Some(Action::Output(Output {
                            body: Some(OutputBody {
                                compliance_ciphertext: ct_a.to_bytes(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        })),
                    },
                    ActionProto {
                        action: Some(Action::Output(Output {
                            body: Some(OutputBody {
                                compliance_ciphertext: ct_b.to_bytes(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        })),
                    },
                ],
                ..Default::default()
            }),
            ..Default::default()
        };

        assert_eq!(
            scan_transaction_for_compliance(&tx, 200, &mck, date, None)
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            scan_transaction_for_compliance(&tx, 200, &mck, date, Some(asset_a))
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            scan_transaction_for_compliance(&tx, 200, &mck, date, Some(asset_b))
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_wrong_mck_no_match() {
        let mut rng = rand_core::OsRng;
        let mck1 = make_mck();
        let mck2 = make_mck();
        let date = 19002u64;

        let self_address = make_address(55);
        let counterparty_address = make_address(66);
        let ack1 = mck1.derive_address_key(self_address.diversifier());

        let asset_id = asset::Id(decaf377::Fq::from(3333u64));
        let (ciphertext, _) = encrypt_compliance_details(
            &mut rng,
            &ack1,
            &self_address,
            date,
            asset_id,
            Amount::from(100u128),
            counterparty_address,
        )
        .unwrap();

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

        // Wrong MCK produces garbage, won't match target asset
        let detected =
            scan_transaction_for_compliance(&tx, 300, &mck2, date, Some(asset_id)).unwrap();
        assert_eq!(detected.len(), 0);
    }
}
