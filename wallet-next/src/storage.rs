use penumbra_chain::params::ChainParams;
use penumbra_crypto::{
    merkle::{NoteCommitmentTree, Tree},
    note::Commitment,
    FieldExt,
};
use penumbra_proto::{crypto::FullViewingKey, Message, Protobuf};
use sqlx::{query, Pool, Sqlite};

use crate::sync::ScanResult;

#[derive(Clone)]
pub struct Storage {
    pool: Pool<Sqlite>,
}

impl Storage {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn migrate(self: &Storage) -> anyhow::Result<()> {
        sqlx::migrate!().run(&self.pool).await.map_err(Into::into)
    }

    /// The last block height we've scanned to, if any.
    pub async fn last_sync_height(&self) -> anyhow::Result<Option<u64>> {
        let result = sqlx::query!(
            r#"
            SELECT height
            FROM sync_height
            ORDER BY height DESC
            LIMIT 1
        "#
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| row.height as u64))
    }

    pub async fn chain_params(&self) -> anyhow::Result<ChainParams> {
        let result = query!(
            r#"
            SELECT bytes
            FROM chain_params
            LIMIT 1
        "#
        )
        .fetch_one(&self.pool)
        .await?;

        ChainParams::decode(result.bytes.as_slice())
    }

    pub async fn full_viewing_key(&self) -> anyhow::Result<FullViewingKey> {
        let result = query!(
            r#"
            SELECT bytes
            FROM full_viewing_key
            LIMIT 1
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(FullViewingKey::decode(result.bytes.as_slice())?)
    }

    pub async fn note_commitment_tree(&self) -> anyhow::Result<NoteCommitmentTree> {
        let result = query!(
            r#"
            SELECT bytes
            FROM note_commitment_tree
            LIMIT 1
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(bincode::deserialize(result.bytes.as_slice())?)
    }

    pub async fn record_block(
        &self,
        scan_result: ScanResult,
        nct: &mut NoteCommitmentTree,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        // Insert all new note records
        for note_record in scan_result.new_notes {
            // https://github.com/launchbadge/sqlx/issues/1430
            // https://github.com/launchbadge/sqlx/issues/1151
            // For some reason we can't use any temporaries with the query! macro
            // any more, even though we did so just fine in the past, e.g.,
            // https://github.com/penumbra-zone/penumbra/blob/e857a7ae2b11b36514a5ac83f8e0b174fa10a65f/pd/src/state/writer.rs#L201-L207
            let note_commitment = note_record.note_commitment.0.to_bytes().to_vec();
            let height_created = scan_result.height as i64;
            let diversifier = note_record.note.diversifier().0.to_vec();
            let amount = note_record.note.amount() as i64;
            let asset_id = note_record.note.asset_id().to_bytes().to_vec();
            let transmission_key = note_record.note.transmission_key().0.to_vec();
            let blinding_factor = note_record.note.note_blinding().to_bytes().to_vec();
            let diversifier_index = note_record.diversifier_index.0.to_vec();
            let nullifier = note_record.nullifier.to_bytes().to_vec();
            sqlx::query!(
                "INSERT INTO notes
                    (
                        note_commitment,
                        height_spent,
                        height_created,
                        diversifier,
                        amount,
                        asset_id,
                        transmission_key,
                        blinding_factor,
                        diversifier_index,
                        nullifier
                    )
                    VALUES 
                    (
                        ?,
                        NULL,
                        ?,
                        ?,
                        ?,
                        ?,
                        ?,
                        ?,
                        ?,
                        ?
                    )",
                note_commitment,
                // height_spent is NULL
                height_created,
                diversifier,
                amount,
                asset_id,
                transmission_key,
                blinding_factor,
                diversifier_index,
                nullifier,
            )
            .execute(&mut tx)
            .await?;
        }

        // Update any rows of the table with matching nullifiers to have height_spent
        for nullifier in scan_result.spent_nullifiers {
            // https://github.com/launchbadge/sqlx/issues/1430
            // https://github.com/launchbadge/sqlx/issues/1151
            // For some reason we can't use any temporaries with the query! macro
            // any more, even though we did so just fine in the past, e.g.,
            // https://github.com/penumbra-zone/penumbra/blob/e857a7ae2b11b36514a5ac83f8e0b174fa10a65f/pd/src/state/writer.rs#L201-L207
            let height_spent = scan_result.height as i64;
            let nullifier = nullifier.to_bytes().to_vec();
            let spent_commitment_bytes = sqlx::query!(
                "UPDATE notes SET height_spent = ? WHERE nullifier = ? RETURNING note_commitment",
                height_spent,
                nullifier,
            )
            .fetch_optional(&mut tx)
            .await?;

            if let Some(bytes) = spent_commitment_bytes {
                // Forget spent note commitments from the NCT
                let spent_commitment = Commitment::try_from(bytes.note_commitment.as_slice())?;
                nct.remove_witness(&spent_commitment);
            }
        }

        // Update NCT table with current NCT state

        let nct_bytes = bincode::serialize(nct)?;
        sqlx::query!("UPDATE note_commitment_tree SET bytes = ?", nct_bytes)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
