use anyhow::{anyhow, Context};
use camino::Utf8Path;
use futures::Future;
use parking_lot::Mutex;
use penumbra_chain::params::{ChainParameters, FmdParameters};
use penumbra_crypto::{
    asset::{self, Id},
    Amount, Asset, FieldExt, FullViewingKey, Nullifier,
};
use penumbra_proto::{
    client::oblivious::{oblivious_query_client::ObliviousQueryClient, ChainParamsRequest},
    Protobuf,
};
use penumbra_tct as tct;
use penumbra_transaction::Transaction;
use sha2::Digest;
use sqlx::{migrate::MigrateDatabase, query, Pool, Sqlite};
use std::{num::NonZeroU64, sync::Arc};
use tct::Commitment;
use tokio::sync::broadcast::{self, error::RecvError};

use crate::{sync::FilteredBlock, QuarantinedNoteRecord, SpendableNoteRecord};

mod nct;
use nct::TreeStore;

#[derive(Clone)]
pub struct Storage {
    pool: Pool<Sqlite>,

    /// This allows an optimization where we only commit to the database after
    /// scanning a nonempty block.
    ///
    /// If this is `Some`, we have uncommitted empty blocks up to the inner height.
    /// If this is `None`, we don't.
    ///
    /// Using a `NonZeroU64` ensures that `Option<NonZeroU64>` fits in 8 bytes.
    uncommitted_height: Arc<Mutex<Option<NonZeroU64>>>,

    scanned_notes_tx: tokio::sync::broadcast::Sender<SpendableNoteRecord>,
    scanned_nullifiers_tx: tokio::sync::broadcast::Sender<Nullifier>,
}

impl Storage {
    /// If the database at `storage_path` exists, [`Self::load`] it, otherwise, [`Self::initialize`] it.
    pub async fn load_or_initialize(
        storage_path: impl AsRef<Utf8Path>,
        fvk: &FullViewingKey,
        node: String,
        pd_port: u16,
    ) -> anyhow::Result<Self> {
        let storage_path = storage_path.as_ref();
        if storage_path.exists() {
            Self::load(storage_path.as_str()).await
        } else {
            let mut client =
                ObliviousQueryClient::connect(format!("http://{}:{}", node, pd_port)).await?;
            let params = client
                .chain_parameters(tonic::Request::new(ChainParamsRequest {
                    chain_id: String::new(),
                }))
                .await?
                .into_inner()
                .try_into()?;
            Self::initialize(storage_path, fvk.clone(), params).await
        }
    }

    async fn connect(path: &str) -> anyhow::Result<Pool<Sqlite>> {
        use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
        use std::str::FromStr;

        /*
        // "yolo" options for testing
        let options = SqliteConnectOptions::from_str(path)?
            .journal_mode(SqliteJournalMode::Memory)
            .synchronous(SqliteSynchronous::Off);
        */
        let options = SqliteConnectOptions::from_str(path)?
            .journal_mode(SqliteJournalMode::Wal)
            // "Normal" will be consistent, but potentially not durable.
            // Since our data is coming from the chain, durability is not
            // a concern -- if we lose some database transactions, it's as
            // if we rewound syncing a few blocks.
            .synchronous(SqliteSynchronous::Normal)
            // The shared cache allows table-level locking, which makes things faster in concurrent
            // cases, and eliminates database lock errors.
            .shared_cache(true);

        let pool = Pool::<Sqlite>::connect_with(options).await?;

        Ok(pool)
    }

    pub async fn load(path: impl AsRef<Utf8Path>) -> anyhow::Result<Self> {
        Ok(Self {
            pool: Self::connect(path.as_ref().as_str()).await?,
            uncommitted_height: Arc::new(Mutex::new(None)),
            scanned_notes_tx: broadcast::channel(10).0,
            scanned_nullifiers_tx: broadcast::channel(10).0,
        })
    }

    pub async fn initialize(
        storage_path: impl AsRef<Utf8Path>,
        fvk: FullViewingKey,
        params: ChainParameters,
    ) -> anyhow::Result<Self> {
        let storage_path = storage_path.as_ref();
        tracing::debug!(%storage_path, ?fvk, ?params);
        // We don't want to overwrite existing data,
        // but also, SQLX will complain if the file doesn't already exist
        if storage_path.exists() {
            return Err(anyhow!("Database already exists at: {}", storage_path));
        } else {
            std::fs::File::create(&storage_path)?;
        }
        // Create the SQLite database
        sqlx::Sqlite::create_database(storage_path.as_str());

        let pool = Self::connect(storage_path.as_str()).await?;

        // Run migrations
        sqlx::migrate!().run(&pool).await?;

        // Initialize the database state with: empty NCT, chain params, FVK
        let mut tx = pool.begin().await?;

        let chain_params_bytes = &ChainParameters::encode_to_vec(&params)[..];
        let fvk_bytes = &FullViewingKey::encode_to_vec(&fvk)[..];

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

        Ok(Storage {
            pool,
            uncommitted_height: Arc::new(Mutex::new(None)),
            scanned_notes_tx: broadcast::channel(10).0,
            scanned_nullifiers_tx: broadcast::channel(10).0,
        })
    }

    /// Query for a note by its note commitment, optionally waiting until the note is detected.
    pub fn note_by_commitment(
        &self,
        note_commitment: tct::Commitment,
        await_detection: bool,
    ) -> impl Future<Output = anyhow::Result<SpendableNoteRecord>> {
        // Start subscribing now, before querying for whether we already
        // have the record, so that we can't miss it if we race a write.
        let mut rx = self.scanned_notes_tx.subscribe();

        // Clone the pool handle so that the returned future is 'static
        let pool = self.pool.clone();
        async move {
            // Check if we already have the note
            if let Some(record) = sqlx::query_as::<_, SpendableNoteRecord>(
                format!(
                    "SELECT 
                        notes.note_commitment,
                        notes.height_created,
                        notes.address,
                        notes.amount,
                        notes.asset_id,
                        notes.blinding_factor,
                        notes.address_index,
                        notes.source,
                        spendable_notes.height_spent,
                        spendable_notes.nullifier,
                        spendable_notes.position
                    FROM notes
                    JOIN spendable_notes ON notes.note_commitment = spendable_notes.note_commitment
                    WHERE notes.note_commitment = x'{}'",
                    hex::encode(note_commitment.0.to_bytes())
                )
                .as_str(),
            )
            .fetch_optional(&pool)
            .await?
            {
                return Ok(record);
            }

            if !await_detection {
                return Err(anyhow!("Note commitment {} not found", note_commitment));
            }

            // Otherwise, wait for newly detected notes and check whether they're
            // the requested one.

            loop {
                match rx.recv().await {
                    Ok(record) => {
                        if record.note_commitment == note_commitment {
                            return Ok(record);
                        }
                    }

                    Err(e) => match e {
                        RecvError::Closed => {
                            return Err(anyhow!(
                            "Receiver error during note detection: closed (no more active senders)"
                        ))
                        }
                        RecvError::Lagged(count) => {
                            return Err(anyhow!(
                                "Receiver error during note detection: lagged (by {:?} messages)",
                                count
                            ))
                        }
                    },
                };
            }
        }
    }

    /// Query for a nullifier's status, optionally waiting until the nullifier is detected.
    pub fn nullifier_status(
        &self,
        nullifier: Nullifier,
        await_detection: bool,
    ) -> impl Future<Output = anyhow::Result<bool>> {
        // Start subscribing now, before querying for whether we already have the nullifier, so we
        // can't miss it if we race a write.
        let mut rx = self.scanned_nullifiers_tx.subscribe();

        // Clone the pool handle so that the returned future is 'static
        let pool = self.pool.clone();

        let nullifier_bytes = nullifier.0.to_bytes().to_vec();

        async move {
            // Check if we already have the nullifier in the set of spent notes
            if let Some(record) = sqlx::query!(
                "SELECT nullifier, height_spent FROM spendable_notes WHERE nullifier = ?",
                nullifier_bytes,
            )
            .fetch_optional(&pool)
            .await?
            {
                let spent = record.height_spent.is_some();

                // If we're awaiting detection and the nullifier isn't yet spent, don't return just yet
                if !await_detection || spent {
                    return Ok(spent);
                }
            }

            // After checking the database, if we didn't find it, return `false` unless we are to
            // await detection
            if !await_detection {
                return Ok(false);
            }

            // Otherwise, wait for newly detected nullifiers and check whether they're the requested
            // one.
            loop {
                let new_nullifier = rx.recv().await.context("Change subscriber failed")?;

                if new_nullifier == nullifier {
                    return Ok(true);
                }
            }
        }
    }

    /// The last block height we've scanned to, if any.
    pub async fn last_sync_height(&self) -> anyhow::Result<Option<u64>> {
        // Check if we have uncommitted blocks beyond the database height.
        if let Some(height) = *self.uncommitted_height.lock() {
            return Ok(Some(height.get()));
        }

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

    pub async fn chain_params(&self) -> anyhow::Result<ChainParameters> {
        let result = query!(
            r#"
            SELECT bytes
            FROM chain_params
            LIMIT 1
        "#
        )
        .fetch_one(&self.pool)
        .await?;

        ChainParameters::decode(result.bytes.as_slice())
    }

    pub async fn fmd_parameters(&self) -> anyhow::Result<FmdParameters> {
        let result = query!(
            r#"
            SELECT bytes
            FROM fmd_parameters
            LIMIT 1
        "#
        )
        .fetch_one(&self.pool)
        .await?;

        FmdParameters::decode(result.bytes.as_slice())
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
        let mut tx = self.pool.begin().await?;
        let tree = tct::Tree::deserialize(&mut TreeStore(&mut tx)).await?;
        tx.commit().await?;
        Ok(tree)
    }
    /// Returns a tuple of (block height, transaction hash) for all transactions in a given range of block heights.
    pub async fn transaction_hashes(
        &self,
        start_height: Option<u64>,
        end_height: Option<u64>,
    ) -> anyhow::Result<Vec<(u64, Vec<u8>)>> {
        let starting_block = start_height.unwrap_or(0) as i64;
        let ending_block = end_height.unwrap_or(self.last_sync_height().await?.unwrap_or(0)) as i64;

        let result = sqlx::query!(
            "SELECT block_height, tx_hash
            FROM tx
            WHERE block_height BETWEEN ? AND ?",
            starting_block,
            ending_block
        )
        .fetch_all(&self.pool)
        .await?;

        let mut output: Vec<(u64, Vec<u8>)> = Vec::new();

        for record in result {
            output.push((record.block_height as u64, record.tx_hash));
        }

        Ok(output)
    }
    /// Returns a tuple of (block height, transaction hash, transaction) for all transactions in a given range of block heights.
    pub async fn transactions(
        &self,
        start_height: Option<u64>,
        end_height: Option<u64>,
    ) -> anyhow::Result<Vec<(u64, Vec<u8>, Transaction)>> {
        let starting_block = start_height.unwrap_or(0) as i64;
        let ending_block = end_height.unwrap_or(self.last_sync_height().await?.unwrap_or(0)) as i64;

        let result = sqlx::query!(
            "SELECT block_height, tx_hash, tx_bytes
            FROM tx
            WHERE block_height BETWEEN ? AND ?",
            starting_block,
            ending_block
        )
        .fetch_all(&self.pool)
        .await?;

        let mut output: Vec<(u64, Vec<u8>, Transaction)> = Vec::new();

        for record in result {
            output.push((
                record.block_height as u64,
                record.tx_hash,
                Transaction::decode(record.tx_bytes.as_slice())?,
            ));
        }

        Ok(output)
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
        address_index: Option<penumbra_crypto::keys::AddressIndex>,
        amount_to_spend: u64,
    ) -> anyhow::Result<Vec<SpendableNoteRecord>> {
        // If set, return spent notes as well as unspent notes.
        // bool include_spent = 2;
        let spent_clause = match include_spent {
            false => "NULL",
            true => "height_spent",
        };

        // If set, only return notes with the specified asset id.
        // crypto.AssetId asset_id = 3;

        let asset_clause = asset_id
            .map(|id| format!("x'{}'", hex::encode(&id.to_bytes())))
            .unwrap_or_else(|| "asset_id".to_string());

        // If set, only return notes with the specified address index.
        // crypto.AddressIndex address_index = 4;
        let address_clause = address_index
            .map(|d| format!("x'{}'", hex::encode(&d.to_bytes())))
            .unwrap_or_else(|| "address_index".to_string());

        let result = sqlx::query_as::<_, SpendableNoteRecord>(
            format!(
                "SELECT notes.note_commitment,
                        notes.height_created,
                        notes.address,
                        notes.amount,
                        notes.asset_id,
                        notes.blinding_factor,
                        notes.address_index,
                        notes.source,
                        spendable_notes.height_spent,
                        spendable_notes.nullifier,
                        spendable_notes.position
            FROM notes
            JOIN spendable_notes ON notes.note_commitment = spendable_notes.note_commitment
            WHERE spendable_notes.height_spent IS {}
            AND notes.asset_id IS {}
            AND notes.address_index IS {}",
                spent_clause, asset_clause, address_clause
            )
            .as_str(),
        )
        .fetch_all(&self.pool)
        .await?;

        // If set, stop returning notes once the total exceeds this amount.
        //
        // Ignored if `asset_id` is unset or if `include_spent` is set.
        // uint64 amount_to_spend = 5;
        //TODO: figure out a clever way to only return notes up to the sum using SQL
        let amount_cutoff = (amount_to_spend != 0) && !(include_spent || asset_id.is_none());
        let mut amount_total = Amount::zero();

        let mut output: Vec<SpendableNoteRecord> = Vec::new();

        for record in result.into_iter() {
            let amount = record.note.amount();
            output.push(record);
            // If we're tracking amounts, accumulate the value of the note
            // and check if we should break out of the loop.
            if amount_cutoff {
                // We know all the notes are of the same type, so adding raw quantities makes sense.
                amount_total = amount_total + amount;
                if amount_total >= amount_to_spend.into() {
                    break;
                }
            }
        }

        if amount_total < amount_to_spend.into() {
            return Err(anyhow!(
                "requested amount of {} exceeds total of {}",
                amount_to_spend,
                amount_total
            ));
        }

        Ok(output)
    }

    pub async fn quarantined_notes(&self) -> anyhow::Result<Vec<QuarantinedNoteRecord>> {
        let result = sqlx::query_as::<_, QuarantinedNoteRecord>(
            "SELECT notes.note_commitment,
                        notes.height_created,
                        notes.address,
                        notes.amount,
                        notes.asset_id,
                        notes.blinding_factor,
                        notes.address_index,
                        notes.source,
                        quarantined_notes.unbonding_epoch,
                        quarantined_notes.identity_key
                        FROM notes 
                        JOIN quarantined_notes 
                        ON quarantined_notes.note_commitment = notes.note_commitment",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
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

    pub async fn record_empty_block(&self, height: u64) -> anyhow::Result<()> {
        //Check that the incoming block height follows the latest recorded height
        let last_sync_height = self.last_sync_height().await?.ok_or_else(|| {
            anyhow::anyhow!("invalid: tried to record empty block as genesis block")
        })?;

        if height != last_sync_height + 1 {
            return Err(anyhow::anyhow!(
                "Wrong block height {} for latest sync height {}",
                height,
                last_sync_height
            ));
        }

        *self.uncommitted_height.lock() = Some(height.try_into().unwrap());
        Ok(())
    }

    /// Filters for nullifiers whose notes we control
    pub async fn filter_nullifiers(
        &self,
        nullifiers: Vec<Nullifier>,
    ) -> anyhow::Result<Vec<Nullifier>> {
        if nullifiers.is_empty() {
            return Ok(Vec::new());
        }
        // pub note_commitment: note::Commitment,
        //     pub note: Note,
        //     pub address_index: AddressIndex,
        //     pub nullifier: Nullifier,
        //     pub height_created: u64,
        //     pub height_spent: Option<u64>,
        //     pub position: tct::Position,
        //     pub source: NoteSource,
        Ok(sqlx::query_as::<_, SpendableNoteRecord>(
            format!(
                "SELECT notes.note_commitment,
                        notes.height_created,
                        notes.address,
                        notes.amount,
                        notes.asset_id,
                        notes.blinding_factor,
                        notes.address_index,
                        notes.source,
                        spendable_notes.height_spent,
                        spendable_notes.nullifier,
                        spendable_notes.position
                FROM notes
                JOIN spendable_notes ON notes.note_commitment = spendable_notes.note_commitment
                WHERE spendable_notes.nullifier IN ({})",
                nullifiers
                    .iter()
                    .map(|x| format!("x'{}'", hex::encode(x.0.to_bytes())))
                    .collect::<Vec<String>>()
                    .join(",")
            )
            .as_str(),
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|x| x.nullifier)
        .collect())
    }

    pub async fn record_block(
        &self,
        filtered_block: FilteredBlock,
        transactions: Vec<Transaction>,
        nct: &mut tct::Tree,
    ) -> anyhow::Result<()> {
        //Check that the incoming block height follows the latest recorded height
        let last_sync_height = self.last_sync_height().await?;

        let correct_height = match last_sync_height {
            // Require that the new block follows the last one we scanned.
            Some(cur_height) => filtered_block.height == cur_height + 1,
            // Require that the new block represents the initial chain state.
            None => filtered_block.height == 0,
        };

        if !correct_height {
            return Err(anyhow::anyhow!(
                "Wrong block height {} for latest sync height {:?}",
                filtered_block.height,
                last_sync_height
            ));
        }
        let mut dbtx = self.pool.begin().await?;

        // Insert all quarantined note commitments into storage
        for quarantined_note_record in &filtered_block.new_quarantined_notes {
            let note_commitment = quarantined_note_record
                .note_commitment
                .0
                .to_bytes()
                .to_vec();
            let height_created = filtered_block.height as i64;
            let address = quarantined_note_record.note.address().to_vec();
            let amount = u64::from(quarantined_note_record.note.amount()) as i64;
            let asset_id = quarantined_note_record.note.asset_id().to_bytes().to_vec();
            let blinding_factor = quarantined_note_record
                .note
                .note_blinding()
                .to_bytes()
                .to_vec();
            let address_index = quarantined_note_record.address_index.to_bytes().to_vec();
            let unbonding_epoch = quarantined_note_record.unbonding_epoch as i64;
            let identity_key = quarantined_note_record.identity_key.encode_to_vec();
            let source = quarantined_note_record.source.to_bytes().to_vec();

            sqlx::query!(
                "INSERT INTO notes
                    (
                        note_commitment,
                        height_created,
                        address,
                        amount,
                        asset_id,
                        blinding_factor,
                        address_index,
                        source
                    )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                note_commitment,
                height_created,
                address,
                amount,
                asset_id,
                blinding_factor,
                address_index,
                source,
            )
            .execute(&mut dbtx)
            .await?;

            sqlx::query!(
                "INSERT INTO quarantined_notes
                    (
                        note_commitment,
                        unbonding_epoch,
                        identity_key
                    )
                VALUES (?, ?, ?)",
                note_commitment,
                unbonding_epoch,
                identity_key,
            )
            .execute(&mut dbtx)
            .await?;
        }

        // Insert new note records into storage
        for note_record in &filtered_block.new_notes {
            // https://github.com/launchbadge/sqlx/issues/1430
            // https://github.com/launchbadge/sqlx/issues/1151
            // For some reason we can't use any temporaries with the query! macro
            // any more, even though we did so just fine in the past, e.g.,
            // https://github.com/penumbra-zone/penumbra/blob/e857a7ae2b11b36514a5ac83f8e0b174fa10a65f/pd/src/state/writer.rs#L201-L207
            let note_commitment = note_record.note_commitment.0.to_bytes().to_vec();
            let height_created = filtered_block.height as i64;
            let address = note_record.note.address().to_vec();
            let amount = u64::from(note_record.note.amount()) as i64;
            let asset_id = note_record.note.asset_id().to_bytes().to_vec();
            let blinding_factor = note_record.note.note_blinding().to_bytes().to_vec();
            let address_index = note_record.address_index.to_bytes().to_vec();
            let nullifier = note_record.nullifier.to_bytes().to_vec();
            let position = (u64::from(note_record.position)) as i64;
            let source = note_record.source.to_bytes().to_vec();

            sqlx::query!(
                "INSERT INTO notes
                    (
                        note_commitment,
                        height_created,
                        address,
                        amount,
                        asset_id,
                        blinding_factor,
                        address_index,
                        source
                    )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                note_commitment,
                height_created,
                address,
                amount,
                asset_id,
                blinding_factor,
                address_index,
                source,
            )
            .execute(&mut dbtx)
            .await?;

            sqlx::query!(
                "INSERT INTO spendable_notes
                    (
                        note_commitment,
                        height_spent,
                        nullifier,
                        position
                    )
                    VALUES
                    (
                        ?,
                        NULL,
                        ?,
                        ?
                    )",
                note_commitment,
                // height_spent is NULL
                nullifier,
                position
            )
            .execute(&mut dbtx)
            .await?;

            // If this note corresponded to a previously quarantined note, delete it from quarantine
            // also, because it is now applied
            sqlx::query!(
                "DELETE FROM quarantined_notes WHERE note_commitment = ?",
                note_commitment,
            )
            .execute(&mut dbtx)
            .await?;
        }

        // Add all quarantined nullifiers to storage and mark notes as spent, *without* forgetting
        // them from the NCT (because they could be rolled back)
        for (identity_key, quarantined_nullifiers) in &filtered_block.spent_quarantined_nullifiers {
            let identity_key = identity_key.encode_to_vec();
            for quarantined_nullifier in quarantined_nullifiers {
                let height_spent = filtered_block.height as i64;
                let nullifier = quarantined_nullifier.to_bytes().to_vec();

                // Track the quarantined nullifier
                sqlx::query!(
                    "INSERT INTO quarantined_nullifiers
                        (
                            identity_key,
                            nullifier
                        )
                    VALUES (?, ?)",
                    identity_key,
                    nullifier,
                )
                .execute(&mut dbtx)
                .await?;

                // Mark the note as spent
                sqlx::query!(
                    "UPDATE spendable_notes SET height_spent = ? WHERE nullifier = ?",
                    height_spent,
                    nullifier,
                )
                .execute(&mut dbtx)
                .await?;
            }
        }

        // Update any rows of the table with matching nullifiers to have height_spent
        for nullifier in &filtered_block.spent_nullifiers {
            // https://github.com/launchbadge/sqlx/issues/1430
            // https://github.com/launchbadge/sqlx/issues/1151
            // For some reason we can't use any temporaries with the query! macro
            // any more, even though we did so just fine in the past, e.g.,
            // https://github.com/penumbra-zone/penumbra/blob/e857a7ae2b11b36514a5ac83f8e0b174fa10a65f/pd/src/state/writer.rs#L201-L207
            let height_spent = filtered_block.height as i64;
            let nullifier = nullifier.to_bytes().to_vec();
            let spent_commitment_bytes = sqlx::query!(
                "UPDATE spendable_notes SET height_spent = ? WHERE nullifier = ? RETURNING note_commitment",
                height_spent,
                nullifier,
            )
            .fetch_optional(&mut dbtx)
            .await?;

            if let Some(bytes) = spent_commitment_bytes {
                // Forget spent note commitments from the NCT
                let spent_commitment = Commitment::try_from(bytes.note_commitment.as_slice())?;
                nct.forget(spent_commitment);
            }

            // If the nullifier was previously quarantined, remove it from the list of quarantined
            // nullifiers, because it has now been spent
            sqlx::query!(
                "DELETE FROM quarantined_nullifiers WHERE nullifier = ?",
                nullifier,
            )
            .execute(&mut dbtx)
            .await?;
        }

        // For any slashed validator, remove all quarantined notes and nullifiers for that
        // validator, and un-spend all spent notes that were referred to by all rolled back
        // nullifiers
        for identity_key in &filtered_block.slashed_validators {
            let identity_key = identity_key.encode_to_vec();

            // Delete all quarantined notes for this validator
            sqlx::query!(
                "DELETE FROM quarantined_notes WHERE identity_key = ?",
                identity_key,
            )
            .execute(&mut dbtx)
            .await?;

            // Collect all the currently quarantined nullifiers for this validator, deleting them in
            // the process
            let rolled_back_nullifiers = sqlx::query!(
                "DELETE FROM quarantined_nullifiers WHERE identity_key = ? RETURNING nullifier",
                identity_key,
            )
            .fetch_all(&mut dbtx)
            .await?;

            // For each such nullifier, roll back the spend of the note associated with it, marking
            // that note as spendable again
            for rolled_back_nullifier in rolled_back_nullifiers {
                let rolled_back_nullifier = rolled_back_nullifier.nullifier.to_vec();
                sqlx::query!(
                    "UPDATE spendable_notes SET height_spent = NULL WHERE nullifier = ?",
                    rolled_back_nullifier,
                )
                .execute(&mut dbtx)
                .await?;
            }
        }

        // Update NCT table with current NCT state
        nct.serialize(&mut TreeStore(&mut dbtx)).await?;

        // Record all transactions
        for transaction in transactions {
            let tx_bytes = transaction.encode_to_vec();
            // We have to create an explicit temporary borrow, because the sqlx api is bad (see above)
            let tx_hash_owned = sha2::Sha256::digest(&tx_bytes);
            let tx_hash = tx_hash_owned.as_slice();
            let tx_block_height = filtered_block.height as i64;

            tracing::debug!(tx_hash = ?hex::encode(tx_hash), "recording extended transaction");

            sqlx::query!(
                "INSERT INTO tx (tx_hash, tx_bytes, block_height) VALUES (?, ?, ?)",
                tx_hash,
                tx_bytes,
                tx_block_height,
            )
            .execute(&mut dbtx)
            .await?;

            // Associate all of the spent nullifiers with the transaction by hash.
            for nf in transaction.spent_nullifiers() {
                let nf_bytes = nf.0.to_bytes().to_vec();
                sqlx::query!(
                    "INSERT INTO tx_by_nullifier (nullifier, tx_hash) VALUES (?, ?)",
                    nf_bytes,
                    tx_hash,
                )
                .execute(&mut dbtx)
                .await?;
            }
        }

        // Update FMD parameters if they've changed.
        if filtered_block.fmd_parameters.is_some() {
            let fmd_parameters_bytes =
                &FmdParameters::encode_to_vec(&filtered_block.fmd_parameters.unwrap())[..];

            sqlx::query!(
                "INSERT INTO fmd_parameters (bytes) VALUES (?)",
                fmd_parameters_bytes
            )
            .execute(&mut dbtx)
            .await?;
        }

        // Record block height as latest synced height

        let latest_sync_height = filtered_block.height as i64;
        sqlx::query!("UPDATE sync_height SET height = ?", latest_sync_height)
            .execute(&mut dbtx)
            .await?;

        dbtx.commit().await?;
        // It's critical to reset the uncommitted height here, since we've just
        // invalidated it by committing.
        self.uncommitted_height.lock().take();

        // Broadcast all committed note records to channel
        // Done following tx.commit() to avoid notifying of a new SpendableNoteRecord before it is actually committed to the database

        for note_record in &filtered_block.new_notes {
            // This will fail to be broadcast if there is no active receiver (such as on initial sync)
            // The error is ignored, as this isn't a problem, because if there is no active receiver there is nothing to do
            let _ = self.scanned_notes_tx.send(note_record.clone());
        }

        for nullifier in filtered_block.spent_nullifiers.iter().chain(
            filtered_block
                .spent_quarantined_nullifiers
                .values()
                .flatten(),
        ) {
            // This will fail to be broadcast if there is no active receiver (such as on initial sync)
            // The error is ignored, as this isn't a problem, because if there is no active receiver there is nothing to do
            let _ = self.scanned_nullifiers_tx.send(*nullifier);
        }

        Ok(())
    }
}
