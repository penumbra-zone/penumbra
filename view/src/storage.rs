use anyhow::anyhow;
use penumbra_chain::params::ChainParams;
use penumbra_crypto::{
    asset::{self, Id},
    ka::Public,
    keys::{Diversifier, DiversifierIndex},
    note::Commitment,
    Asset, FieldExt, Fq, FullViewingKey, Note, Nullifier, Value,
};
use penumbra_proto::{
    client::oblivious::{oblivious_query_client::ObliviousQueryClient, ChainParamsRequest},
    Protobuf,
};
use penumbra_tct as tct;
use sqlx::{migrate::MigrateDatabase, query, Pool, Sqlite};
use std::path::PathBuf;
use tonic::transport::Channel;

use crate::{sync::ScanResult, NoteRecord};

#[derive(Clone)]
pub struct Storage {
    pool: Pool<Sqlite>,
}

impl Storage {
    /// If the database at `storage_path` exists, [`Self::load`] it, otherwise, [`Self::initialize`] it.
    pub async fn load_or_initialize(
        storage_path: String,
        fvk: &FullViewingKey,
        client: &mut ObliviousQueryClient<Channel>,
    ) -> anyhow::Result<Self> {
        if PathBuf::from(&storage_path).exists() {
            Self::load(storage_path).await
        } else {
            let params = client
                .chain_params(tonic::Request::new(ChainParamsRequest {
                    chain_id: String::new(),
                }))
                .await?
                .into_inner()
                .try_into()?;
            Self::initialize(storage_path, fvk.clone(), params).await
        }
    }

    pub async fn load(storage_path: String) -> anyhow::Result<Self> {
        Ok(Self {
            pool: Pool::<Sqlite>::connect(&storage_path).await?,
        })
    }

    pub async fn initialize(
        storage_path: String,
        fvk: FullViewingKey,
        params: ChainParams,
    ) -> anyhow::Result<Self> {
        tracing::debug!(?storage_path, ?fvk, ?params);
        // We don't want to overwrite existing data,
        // but also, SQLX will complain if the file doesn't already exist
        if PathBuf::from(&storage_path).exists() {
            return Err(anyhow!("Database already exists at: {}", storage_path));
        } else {
            std::fs::File::create(&storage_path)?;
        }
        // Create the SQLite database
        sqlx::Sqlite::create_database(&storage_path);

        let pool = Pool::<Sqlite>::connect(&storage_path).await?;

        // Run migrations
        sqlx::migrate!().run(&pool).await?;

        // Initialize the database state with: empty NCT, chain params, FVK
        let mut tx = pool.begin().await?;

        let nct_bytes = bincode::serialize(&tct::Tree::new())?;
        let chain_params_bytes = &ChainParams::encode_to_vec(&params)[..];
        let fvk_bytes = &FullViewingKey::encode_to_vec(&fvk)[..];

        sqlx::query!(
            "INSERT INTO note_commitment_tree (bytes) VALUES (?)",
            nct_bytes
        )
        .execute(&mut tx)
        .await?;

        sqlx::query!(
            "INSERT INTO chain_params (bytes) VALUES (?)",
            chain_params_bytes
        )
        .execute(&mut tx)
        .await?;

        sqlx::query!("INSERT INTO full_viewing_key (bytes) VALUES (?)", fvk_bytes)
            .execute(&mut tx)
            .await?;

        // Insert -1 as a signaling value for pre-genesis.
        // We just have to be careful to treat negative values as None
        // in last_sync_height.
        sqlx::query!("INSERT INTO sync_height (height) VALUES (?)", -1i64)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;

        Ok(Storage { pool })
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
        .fetch_one(&self.pool)
        .await?;

        // Special-case negative values to None
        Ok(u64::try_from(result.height).ok())
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

        FullViewingKey::decode(result.bytes.as_slice())
    }

    pub async fn note_commitment_tree(&self) -> anyhow::Result<tct::Tree> {
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

    pub async fn assets(&self) -> anyhow::Result<Vec<Asset>> {
        let result = sqlx::query!(
            "SELECT *
            FROM assets"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut output: Vec<Asset> = Vec::new();

        for record in result {
            let asset = Asset {
                id: Id::try_from(record.asset_id.as_slice())?,
                denom: asset::REGISTRY
                    .parse_denom(&record.denom)
                    .ok_or_else(|| anyhow::anyhow!("invalid denomination {}", record.denom))?,
            };
            output.push(asset);
        }

        Ok(output)
    }

    pub async fn notes(
        &self,
        include_spent: bool,
        asset_id: Option<asset::Id>,
        diversifier_index: Option<penumbra_crypto::keys::DiversifierIndex>,
        amount_to_spend: u64,
    ) -> anyhow::Result<Vec<NoteRecord>> {
        // If set, return spent notes as well as unspent notes.
        // bool include_spent = 2;
        let spent_clause = match include_spent {
            false => "NULL",
            true => "height_spent",
        };

        // If set, only return notes with the specified asset id.
        // crypto.AssetId asset_id = 3;

        let asset_clause = match asset_id {
            Some(x) => format!("{:?}", x),
            None => "asset_id".to_string(),
        };

        // If set, only return notes with the specified diversifier index.
        // crypto.DiversifierIndex diversifier_index = 4;

        let diversifier_clause = match diversifier_index {
            Some(x) => format!("{:?}", x),
            None => "diversifier_index".to_string(),
        };

        let result = sqlx::query!(
            "SELECT *
            FROM notes
            WHERE height_spent = ?
            AND asset_id = ?
            AND diversifier_index = ?",
            spent_clause,
            asset_clause,
            diversifier_clause
        )
        .fetch_all(&self.pool)
        .await?;

        // If set, stop returning notes once the total exceeds this amount.
        //
        // Ignored if `asset_id` is unset or if `include_spent` is set.
        // uint64 amount_to_spend = 5;
        //TODO: figure out a clever way to only return notes up to the sum using SQL
        let amount_cutoff = !(include_spent || asset_id.is_none());
        let mut amount_total = 0;

        let mut output: Vec<NoteRecord> = Vec::new();

        for record in result {
            let diversifier = Diversifier::try_from(&record.diversifier[..])?;
            let transmission_key = Public(record.transmission_key[..].try_into()?);
            let value = Value {
                amount: record.amount as u64,
                asset_id: asset::Id(Fq::from_bytes(record.asset_id[..].try_into()?)?),
            };
            let note_blinding = Fq::from_bytes(record.blinding_factor[..].try_into()?)?;

            output.push(NoteRecord {
                note_commitment: Commitment::try_from(&record.note_commitment[..])?,
                note: Note::from_parts(diversifier, transmission_key, value, note_blinding)?,
                diversifier_index: DiversifierIndex(record.diversifier_index[..].try_into()?),
                nullifier: Nullifier::try_from(record.nullifier)?,
                height_created: record.height_created as u64,
                height_spent: if record.height_spent == None {
                    None
                } else {
                    Some(record.height_spent.unwrap() as u64)
                }, //height_spent is nullable
                position: (record.position as u64).into(),
            });

            // If we're tracking amounts, accumulate the value of the note
            // and check if we should break out of the loop.
            if amount_cutoff {
                // We know all the notes are of the same type, so adding raw quantities makes sense.
                amount_total += record.amount as u64;
                if amount_total >= amount_to_spend {
                    break;
                }
            }
        }

        if amount_total < amount_to_spend {
            return Err(anyhow!(
                "requested amount of {} exceeds total of {}",
                amount_to_spend,
                amount_total
            ));
        }

        Ok(output)
    }

    pub async fn record_asset(&self, asset: Asset) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        let asset_id = asset.id.to_bytes().to_vec();
        let denom = asset.denom.to_string();
        sqlx::query!(
            "INSERT INTO assets
                    (
                        asset_id,
                        denom
                    )
                    VALUES
                    (
                        ?,
                        ?
                    )",
            asset_id,
            denom,
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn record_block(
        &self,
        scan_result: ScanResult,
        nct: &mut tct::Tree,
    ) -> anyhow::Result<()> {
        //Check that the incoming block height follows the latest recorded height
        let last_sync_height = self.last_sync_height().await?;

        let correct_height = match last_sync_height {
            // Require that the new block follows the last one we scanned.
            Some(cur_height) => scan_result.height == cur_height + 1,
            // Require that the new block represents the initial chain state.
            None => scan_result.height == 0,
        };

        if !correct_height {
            return Err(anyhow::anyhow!(
                "Wrong block height {:?} for latest sync height {:?}",
                scan_result.height,
                last_sync_height
            ));
        }
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
            let position = (u64::from(note_record.position)) as i64;
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
                        nullifier,
                        position
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
                position,
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
                nct.forget(spent_commitment);
            }
        }

        // Update NCT table with current NCT state

        let nct_bytes = bincode::serialize(nct)?;
        sqlx::query!("UPDATE note_commitment_tree SET bytes = ?", nct_bytes)
            .execute(&mut tx)
            .await?;

        // Record block height as latest synced height

        let latest_sync_height = scan_result.height as i64;
        sqlx::query!("UPDATE sync_height SET height = ?", latest_sync_height)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
