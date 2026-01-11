//! SQLite storage backend for compliance scanning.
//!
//! This module provides persistent storage for detected compliance transfers,
//! allowing issuers to track all transfers of their regulated assets.

use anyhow::{Context, Result};
use penumbra_sdk_asset::asset;
use penumbra_sdk_num::Amount;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::sync::{DetectedTransfer, PartialAddress};

/// SQLite-backed storage for compliance scanning results.
///
/// This provides:
/// - Persistent storage of detected transfers
/// - Query capabilities by asset, height
/// - Tracking of last synced height
pub struct ComplianceStorage {
    conn: Arc<Mutex<Connection>>,
}

impl ComplianceStorage {
    /// Create or open a compliance storage database.
    ///
    /// If the database doesn't exist, it will be created with the appropriate schema.
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Initialize schema
        // Note: We store partial addresses as hex-encoded bytes since we can't
        // reconstruct full addresses from compliance ciphertext data.
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS detected_transfers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                height INTEGER NOT NULL,
                action_index INTEGER NOT NULL,
                asset_id TEXT NOT NULL,
                amount TEXT NOT NULL,
                self_div_gen BLOB NOT NULL,
                self_trans_key BLOB NOT NULL,
                counterparty_div_gen BLOB NOT NULL,
                counterparty_trans_key BLOB NOT NULL,
                nullifier TEXT,
                UNIQUE(height, action_index)
            );

            CREATE INDEX IF NOT EXISTS idx_height ON detected_transfers(height);
            CREATE INDEX IF NOT EXISTS idx_asset_id ON detected_transfers(asset_id);
            CREATE INDEX IF NOT EXISTS idx_nullifier ON detected_transfers(nullifier) WHERE nullifier IS NOT NULL;

            CREATE TABLE IF NOT EXISTS sync_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                last_height INTEGER NOT NULL DEFAULT 0
            );

            INSERT OR IGNORE INTO sync_state (id, last_height) VALUES (1, 0);
            "#,
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Save a detected transfer to the database.
    pub fn save_transfer(&self, transfer: &DetectedTransfer) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            INSERT OR REPLACE INTO detected_transfers
                (height, action_index, asset_id, amount, self_div_gen, self_trans_key, counterparty_div_gen, counterparty_trans_key, nullifier)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                transfer.height as i64,
                transfer.action_index as i64,
                transfer.asset_id.to_string(),
                transfer.amount.to_string(),
                transfer.self_address.diversified_generator.as_slice(),
                transfer.self_address.transmission_key.as_slice(),
                transfer.counterparty_address.diversified_generator.as_slice(),
                transfer.counterparty_address.transmission_key.as_slice(),
                transfer.nullifier.as_ref().map(|n| format!("{:?}", n)),
            ],
        )?;

        Ok(())
    }

    /// Save multiple transfers in a single transaction.
    pub fn save_transfers(&self, transfers: &[DetectedTransfer]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;

        for transfer in transfers {
            tx.execute(
                r#"
                INSERT OR REPLACE INTO detected_transfers
                    (height, action_index, asset_id, amount, self_div_gen, self_trans_key, counterparty_div_gen, counterparty_trans_key, nullifier)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                params![
                    transfer.height as i64,
                    transfer.action_index as i64,
                    transfer.asset_id.to_string(),
                    transfer.amount.to_string(),
                    transfer.self_address.diversified_generator.as_slice(),
                    transfer.self_address.transmission_key.as_slice(),
                    transfer.counterparty_address.diversified_generator.as_slice(),
                    transfer.counterparty_address.transmission_key.as_slice(),
                    transfer.nullifier.as_ref().map(|n| format!("{:?}", n)),
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Query transfers with optional filters.
    ///
    /// # Arguments
    /// * `asset_id` - Optional asset ID filter
    /// * `min_height` - Optional minimum height filter
    /// * `max_height` - Optional maximum height filter
    pub fn query_transfers(
        &self,
        asset_id: Option<&asset::Id>,
        min_height: Option<u64>,
        max_height: Option<u64>,
    ) -> Result<Vec<DetectedTransfer>> {
        let conn = self.conn.lock().unwrap();

        // Build dynamic query
        let mut query = String::from(
            "SELECT height, action_index, asset_id, amount, self_div_gen, self_trans_key, counterparty_div_gen, counterparty_trans_key, nullifier FROM detected_transfers WHERE 1=1"
        );
        let mut params_vec: Vec<String> = Vec::new();

        if asset_id.is_some() {
            query.push_str(" AND asset_id = ?");
            params_vec.push(asset_id.unwrap().to_string());
        }

        if let Some(min) = min_height {
            query.push_str(" AND height >= ?");
            params_vec.push(min.to_string());
        }

        if let Some(max) = max_height {
            query.push_str(" AND height <= ?");
            params_vec.push(max.to_string());
        }

        query.push_str(" ORDER BY height, action_index");

        let mut stmt = conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let height: i64 = row.get(0)?;
            let action_index: i64 = row.get(1)?;
            let asset_id_str: String = row.get(2)?;
            let amount_str: String = row.get(3)?;
            let self_div_gen: Vec<u8> = row.get(4)?;
            let self_trans_key: Vec<u8> = row.get(5)?;
            let counterparty_div_gen: Vec<u8> = row.get(6)?;
            let counterparty_trans_key: Vec<u8> = row.get(7)?;
            let _nullifier_str: Option<String> = row.get(8)?;

            Ok((
                height as u64,
                action_index as usize,
                asset_id_str,
                amount_str,
                self_div_gen,
                self_trans_key,
                counterparty_div_gen,
                counterparty_trans_key,
            ))
        })?;

        let mut transfers = Vec::new();
        for row in rows {
            let (
                height,
                action_index,
                asset_id_str,
                amount_str,
                self_div_gen,
                self_trans_key,
                counterparty_div_gen,
                counterparty_trans_key,
            ) = row?;

            let asset_id: asset::Id = asset_id_str.parse().context("invalid asset id")?;
            let amount_u128: u128 = amount_str.parse().context("invalid amount")?;
            let amount = Amount::from(amount_u128);

            // Convert Vec<u8> to [u8; 32]
            let self_div_gen_arr: [u8; 32] = self_div_gen
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid self_div_gen length"))?;
            let self_trans_key_arr: [u8; 32] = self_trans_key
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid self_trans_key length"))?;
            let counterparty_div_gen_arr: [u8; 32] = counterparty_div_gen
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid counterparty_div_gen length"))?;
            let counterparty_trans_key_arr: [u8; 32] = counterparty_trans_key
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid counterparty_trans_key length"))?;

            transfers.push(DetectedTransfer {
                height,
                action_index,
                asset_id,
                amount,
                self_address: PartialAddress {
                    diversified_generator: self_div_gen_arr,
                    transmission_key: self_trans_key_arr,
                },
                counterparty_address: PartialAddress {
                    diversified_generator: counterparty_div_gen_arr,
                    transmission_key: counterparty_trans_key_arr,
                },
                nullifier: None, // TODO: Parse nullifier if needed
            });
        }

        Ok(transfers)
    }

    /// Get the last synced height.
    pub fn last_sync_height(&self) -> Result<u64> {
        let conn = self.conn.lock().unwrap();

        let height: i64 = conn
            .query_row(
                "SELECT last_height FROM sync_state WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0);

        Ok(height as u64)
    }

    /// Update the last synced height.
    pub fn update_sync_height(&self, height: u64) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE sync_state SET last_height = ?1 WHERE id = 1",
            params![height as i64],
        )?;

        Ok(())
    }

    /// Get count of detected transfers.
    pub fn transfer_count(&self) -> Result<u64> {
        let conn = self.conn.lock().unwrap();

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM detected_transfers", [], |row| {
            row.get(0)
        })?;

        Ok(count as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::encrypt_compliance_details;
    use crate::scanner::decrypt::decrypt_with_mck;
    use crate::test_helpers::{make_address, make_mck};
    use penumbra_sdk_num::Amount;
    use tempfile::NamedTempFile;

    #[test]
    fn test_storage_create_and_query() {
        let mut rng = rand_core::OsRng;

        // Create temp database
        let temp_file = NamedTempFile::new().unwrap();
        let storage = ComplianceStorage::new(temp_file.path()).unwrap();

        // Verify initial state
        assert_eq!(storage.last_sync_height().unwrap(), 0);
        assert_eq!(storage.transfer_count().unwrap(), 0);

        // Setup MCK and addresses
        let mck = make_mck();
        let date = 19000u64;

        let self_address = make_address(11);
        let counterparty_address = make_address(22);
        let ack = mck.derive_address_key(self_address.diversifier());

        let asset_id = asset::Id(decaf377::Fq::from(12345u64));
        let amount = Amount::from(1000u64);

        // Encrypt and decrypt to get a real DetectedTransfer
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

        let decrypted = decrypt_with_mck(&mck, date, &ciphertext).unwrap();

        let transfer = DetectedTransfer {
            height: 100,
            action_index: 0,
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
        };

        storage.save_transfer(&transfer).unwrap();

        // Verify count
        assert_eq!(storage.transfer_count().unwrap(), 1);

        // Query all transfers
        let transfers = storage.query_transfers(None, None, None).unwrap();
        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0].height, 100);
        assert_eq!(transfers[0].amount, Amount::from(1000u64));
        assert_eq!(transfers[0].asset_id, asset_id);

        // Update sync height
        storage.update_sync_height(100).unwrap();
        assert_eq!(storage.last_sync_height().unwrap(), 100);
    }

    #[test]
    fn test_query_filters() {
        let mut rng = rand_core::OsRng;

        let temp_file = NamedTempFile::new().unwrap();
        let storage = ComplianceStorage::new(temp_file.path()).unwrap();

        let mck = make_mck();
        let date = 19001u64;

        let self_address = make_address(33);
        let counterparty_address = make_address(44);
        let ack = mck.derive_address_key(self_address.diversifier());

        let asset_a = asset::Id(decaf377::Fq::from(1111u64));
        let asset_b = asset::Id(decaf377::Fq::from(2222u64));

        // Create transfers with different assets and heights
        let mut transfers = Vec::new();
        for (i, asset_id) in [(0, asset_a), (1, asset_b), (2, asset_a)].iter() {
            let (ciphertext, _) = encrypt_compliance_details(
                &mut rng,
                &ack,
                &self_address,
                date,
                *asset_id,
                Amount::from((100 + i * 100) as u64),
                counterparty_address.clone(),
            )
            .unwrap();

            let decrypted = decrypt_with_mck(&mck, date, &ciphertext).unwrap();

            transfers.push(DetectedTransfer {
                height: 100 + *i as u64,
                action_index: 0,
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

        storage.save_transfers(&transfers).unwrap();

        // Query by asset
        let results_a = storage.query_transfers(Some(&asset_a), None, None).unwrap();
        assert_eq!(results_a.len(), 2, "Should find 2 transfers with asset_a");

        let results_b = storage.query_transfers(Some(&asset_b), None, None).unwrap();
        assert_eq!(results_b.len(), 1, "Should find 1 transfer with asset_b");

        // Query by height range
        let results = storage.query_transfers(None, Some(101), Some(101)).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].height, 101);
    }
}
