use anyhow::{anyhow, Context};
use camino::Utf8Path;
use futures::Future;
use parking_lot::Mutex;
use penumbra_chain::params::{ChainParameters, FmdParameters};
use penumbra_crypto::{
    asset::{self, Denom, Id},
    note,
    stake::{DelegationToken, IdentityKey},
    Address, Amount, Asset, FieldExt, Fq, FullViewingKey, Note, Nullifier, Rseed, Value,
};
use penumbra_proto::{
    client::v1alpha1::{
        oblivious_query_service_client::ObliviousQueryServiceClient, ChainParametersRequest,
    },
    DomainType,
};
use penumbra_tct as tct;
use penumbra_transaction::Transaction;
use sha2::Digest;
use sqlx::{migrate::MigrateDatabase, query, Pool, Row, Sqlite};
use std::{collections::BTreeMap, num::NonZeroU64, str::FromStr, sync::Arc};
use tct::Commitment;
use tokio::sync::broadcast::{self, error::RecvError};

use crate::{sync::FilteredBlock, SpendableNoteRecord, SwapRecord};

mod sct;
use sct::TreeStore;

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
    scanned_swaps_tx: tokio::sync::broadcast::Sender<SwapRecord>,
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
                ObliviousQueryServiceClient::connect(format!("http://{node}:{pd_port}")).await?;
            let params = client
                .chain_parameters(tonic::Request::new(ChainParametersRequest {
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
            scanned_swaps_tx: broadcast::channel(10).0,
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
            std::fs::File::create(storage_path)?;
        }
        // Create the SQLite database
        sqlx::Sqlite::create_database(storage_path.as_str());

        let pool = Self::connect(storage_path.as_str()).await?;

        // Run migrations
        sqlx::migrate!().run(&pool).await?;

        // Initialize the database state with: empty SCT, chain params, FVK
        let mut tx = pool.begin().await?;

        let chain_params_bytes = &ChainParameters::encode_to_vec(&params)[..];
        sqlx::query!(
            "INSERT INTO chain_params (bytes) VALUES (?)",
            chain_params_bytes
        )
        .execute(&mut tx)
        .await?;

        let fvk_bytes = &FullViewingKey::encode_to_vec(&fvk)[..];
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
            scanned_swaps_tx: broadcast::channel(10).0,
        })
    }

    /// Query for account balance by address
    pub async fn balance_by_address(&self, address: Address) -> anyhow::Result<BTreeMap<Id, u64>> {
        let address = address.to_vec();

        let result = sqlx::query!(
            "SELECT notes.asset_id,
                    notes.amount
            FROM    notes
            JOIN    spendable_notes ON notes.note_commitment = spendable_notes.note_commitment
            WHERE   spendable_notes.height_spent IS NULL
            AND     notes.address IS ?",
            address
        )
        .fetch_all(&self.pool)
        .await?;

        let mut balance_by_address = BTreeMap::new();

        for record in result {
            balance_by_address
                .entry(Id::try_from(record.asset_id.as_slice())?)
                .and_modify(|x| *x += record.amount as u64)
                .or_insert(record.amount as u64);
        }

        Ok(balance_by_address)
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
                        spendable_notes.height_created,
                        notes.address,
                        notes.amount,
                        notes.asset_id,
                        notes.rseed,
                        spendable_notes.address_index,
                        spendable_notes.source,
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

    /// Query for a swap by its swap commitment, optionally waiting until the note is detected.
    pub fn swap_by_commitment(
        &self,
        swap_commitment: tct::Commitment,
        await_detection: bool,
    ) -> impl Future<Output = anyhow::Result<SwapRecord>> {
        // Start subscribing now, before querying for whether we already
        // have the record, so that we can't miss it if we race a write.
        let mut rx = self.scanned_swaps_tx.subscribe();

        // Clone the pool handle so that the returned future is 'static
        let pool = self.pool.clone();
        async move {
            // Check if we already have the note
            if let Some(record) = sqlx::query_as::<_, SwapRecord>(
                format!(
                    "SELECT * FROM swaps WHERE swaps.swap_commitment = x'{}'",
                    hex::encode(swap_commitment.0.to_bytes())
                )
                .as_str(),
            )
            .fetch_optional(&pool)
            .await?
            {
                return Ok(record);
            }

            if !await_detection {
                return Err(anyhow!("swap commitment {} not found", swap_commitment));
            }

            // Otherwise, wait for newly detected swaps and check whether they're
            // the requested one.

            loop {
                match rx.recv().await {
                    Ok(record) => {
                        if record.swap_commitment == swap_commitment {
                            return Ok(record);
                        }
                    }

                    Err(e) => match e {
                        RecvError::Closed => {
                            return Err(anyhow!(
                            "Receiver error during swap detection: closed (no more active senders)"
                        ))
                        }
                        RecvError::Lagged(count) => {
                            return Err(anyhow!(
                                "Receiver error during swap detection: lagged (by {:?} messages)",
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
        Ok(u64::try_from(
            result
                .height
                .ok_or_else(|| anyhow!("missing sync height"))?,
        )
        .ok())
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

    pub async fn state_commitment_tree(&self) -> anyhow::Result<tct::Tree> {
        let mut tx = self.pool.begin().await?;
        let tree = tct::Tree::from_async_reader(&mut TreeStore(&mut tx)).await?;
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

    pub async fn transaction_by_hash(&self, tx_hash: &[u8]) -> anyhow::Result<Option<Transaction>> {
        let result = sqlx::query!(
            "SELECT block_height, tx_hash, tx_bytes
            FROM tx
            WHERE tx_hash = ?",
            tx_hash
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(match result {
            Some(record) => Some(Transaction::decode(record.tx_bytes.as_slice())?),
            None => None,
        })
    }

    // Query for a note by its note commitment, optionally waiting until the note is detected.
    pub fn note_by_nullifier(
        &self,
        nullifier: Nullifier,
        await_detection: bool,
    ) -> impl Future<Output = anyhow::Result<SpendableNoteRecord>> {
        // Start subscribing now, before querying for whether we already
        // have the record, so that we can't miss it if we race a write.
        let mut rx = self.scanned_notes_tx.subscribe();

        // Clone the pool handle so that the returned future is 'static
        let pool = self.pool.clone();

        let nullifier_bytes = nullifier.to_bytes().to_vec();
        async move {
            // Check if we already have the note
            if let Some(record) = sqlx::query_as::<_, SpendableNoteRecord>(
                // TODO: would really be better to use a prepared statement here rather than manually
                // quoting the nullifier bytes. tried to get the `sqlx::query_as!` macro to work
                // but the types didn't work out easily.
                format!(
                    "SELECT
                        notes.note_commitment,
                        spendable_notes.height_created,
                        notes.address,
                        notes.amount,
                        notes.asset_id,
                        notes.rseed,
                        spendable_notes.address_index,
                        spendable_notes.source,
                        spendable_notes.height_spent,
                        spendable_notes.nullifier,
                        spendable_notes.position
                    FROM notes
                    JOIN spendable_notes ON notes.note_commitment = spendable_notes.note_commitment
                    WHERE hex(spendable_notes.nullifier) = \"{}\"",
                    hex::encode_upper(nullifier_bytes)
                )
                .as_str(),
            )
            .fetch_optional(&pool)
            .await?
            {
                return Ok(record);
            }

            if !await_detection {
                return Err(anyhow!(
                    "Note commitment for nullifier {:?} not found",
                    nullifier
                ));
            }

            // Otherwise, wait for newly detected notes and check whether they're
            // the requested one.

            loop {
                match rx.recv().await {
                    Ok(record) => {
                        if record.nullifier == nullifier {
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

    pub async fn all_assets(&self) -> anyhow::Result<Vec<Asset>> {
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

    pub async fn asset_by_denom(&self, denom: &Denom) -> anyhow::Result<Option<Asset>> {
        let denom_string = denom.to_string();

        let result = sqlx::query!(
            "SELECT *
            FROM assets
            WHERE denom = ?",
            denom_string
        )
        .fetch_optional(&self.pool)
        .await?;

        result
            .map(|record| {
                Ok(Asset {
                    id: Id::try_from(record.asset_id.as_slice())?,
                    denom: denom.clone(),
                })
            })
            .transpose()
    }

    // Get assets whose denoms match the given SQL LIKE pattern, with the `_` and `%` wildcards,
    // where `\` is the escape character.
    pub async fn assets_matching(&self, pattern: &str) -> anyhow::Result<Vec<Asset>> {
        let result = sqlx::query!(
            "SELECT *
            FROM assets
            WHERE denom LIKE ?
            ESCAPE '\'",
            pattern
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
        // core.crypto.v1alpha1.AssetId asset_id = 3;
        let asset_clause = asset_id
            .map(|id| format!("x'{}'", hex::encode(id.to_bytes())))
            .unwrap_or_else(|| "asset_id".to_string());

        // If set, only return notes with the specified address index.
        // crypto.AddressIndex address_index = 4;
        // This isn't what we want any more, we need to be indexing notes
        // by *account*, not just by address index.
        // For now, just do filtering in software.
        /*
        let address_clause = address_index
            .map(|d| format!("x'{}'", hex::encode(d.to_bytes())))
            .unwrap_or_else(|| "address_index".to_string());
         */
        let address_clause = "address_index".to_string();

        let result = sqlx::query_as::<_, SpendableNoteRecord>(
            format!(
                "SELECT notes.note_commitment,
                        spendable_notes.height_created,
                        notes.address,
                        notes.amount,
                        notes.asset_id,
                        notes.rseed,
                        spendable_notes.address_index,
                        spendable_notes.source,
                        spendable_notes.height_spent,
                        spendable_notes.nullifier,
                        spendable_notes.position
            FROM notes
            JOIN spendable_notes ON notes.note_commitment = spendable_notes.note_commitment
            WHERE spendable_notes.height_spent IS {spent_clause}
            AND notes.asset_id IS {asset_clause}
            AND spendable_notes.address_index IS {address_clause}"
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
            // Skip notes that don't match the account, since we're
            // not doing account filtering in SQL as a temporary hack (see above)
            if let Some(address_index) = address_index {
                if record.address_index.account != address_index.account {
                    continue;
                }
            }
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

    pub async fn notes_for_voting(
        &self,
        address_index: Option<penumbra_crypto::keys::AddressIndex>,
        votable_at_height: u64,
    ) -> anyhow::Result<Vec<(SpendableNoteRecord, IdentityKey)>> {
        // If set, only return notes with the specified address index.
        // crypto.AddressIndex address_index = 3;
        let address_clause = address_index
            .map(|d| format!("x'{}'", hex::encode(d.to_bytes())))
            .unwrap_or_else(|| "address_index".to_string());

        let spendable_note_records = sqlx::query_as::<_, SpendableNoteRecord>(
            format!(
                "SELECT notes.note_commitment,
                        spendable_notes.height_created,
                        notes.address,
                        notes.amount,
                        notes.asset_id,
                        notes.rseed,
                        spendable_notes.address_index,
                        spendable_notes.source,
                        spendable_notes.height_spent,
                        spendable_notes.nullifier,
                        spendable_notes.position
                FROM
                    notes JOIN spendable_notes ON notes.note_commitment = spendable_notes.note_commitment
                WHERE
                    spendable_notes.address_index IS {address_clause}
                    AND notes.asset_id IN (
                        SELECT asset_id FROM assets WHERE denom LIKE '_delegation\\_%' ESCAPE '\\'
                    )
                    AND ((spendable_notes.height_spent IS NULL) OR (spendable_notes.height_spent > {votable_at_height}))
                    AND (spendable_notes.height_created < {votable_at_height})
                ",
            )
            .as_str(),
        )
        .fetch_all(&self.pool)
        .await?;

        // TODO: this could be internalized into the SQL query in principle, but it's easier to do
        // it this way; if it becomes slow, we can do it better
        let mut results = Vec::new();
        for record in spendable_note_records {
            let asset_id = record.note.asset_id().to_bytes().to_vec();
            let denom = sqlx::query!("SELECT denom FROM assets WHERE asset_id = ?", asset_id)
                .fetch_one(&self.pool)
                .await?
                .denom;

            let identity_key = DelegationToken::from_str(&denom)
                .context("invalid delegation token denom")?
                .validator();

            results.push((record, identity_key));
        }

        Ok(results)
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

    pub async fn give_advice(&self, note: Note) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        let note_commitment = note.commit().0.to_bytes().to_vec();
        let address = note.address().to_vec();
        let amount = u64::from(note.amount()) as i64;
        let asset_id = note.asset_id().to_bytes().to_vec();
        let rseed = note.rseed().to_bytes().to_vec();

        sqlx::query!(
            "INSERT INTO notes
                    (
                        note_commitment,
                        address,
                        amount,
                        asset_id,
                        rseed
                    )
                    VALUES
                    (?, ?, ?, ?, ?)",
            note_commitment,
            address,
            amount,
            asset_id,
            rseed,
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Return advice about note contents for use in scanning.
    ///
    /// Given a list of note commitments, this method checks whether any of them
    /// correspond to notes that have been recorded in the database but not yet
    /// observed during scanning.
    pub async fn scan_advice(
        &self,
        note_commitments: Vec<note::Commitment>,
    ) -> anyhow::Result<BTreeMap<note::Commitment, Note>> {
        if note_commitments.is_empty() {
            return Ok(BTreeMap::new());
        }

        let rows = sqlx::query(
            format!(
                "SELECT notes.address,
                        notes.amount,
                        notes.asset_id,
                        notes.rseed
                FROM notes
                LEFT OUTER JOIN spendable_notes ON notes.note_commitment = spendable_notes.note_commitment
                WHERE (spendable_notes.note_commitment IS NULL) AND (notes.note_commitment IN ({}))",
                note_commitments
                    .iter()
                    .map(|cm| format!("x'{}'", hex::encode(cm.0.to_bytes())))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .as_str(),
        )
        .fetch_all(&self.pool)
        .await?;

        let mut notes = BTreeMap::new();
        for row in rows {
            let address = Address::try_from(row.get::<&[u8], _>("address"))?;
            let amount = (row.get::<i64, _>("amount") as u64).into();
            let asset_id = asset::Id(Fq::from_bytes(
                row.get::<&[u8], _>("asset_id")
                    .try_into()
                    .expect("32 bytes"),
            )?);
            let rseed = Rseed(row.get::<&[u8], _>("rseed").try_into().expect("32 bytes"));

            let note = Note::from_parts(address, Value { amount, asset_id }, rseed).unwrap();

            notes.insert(note.commit(), note);
        }

        Ok(notes)
    }

    /// Filters for nullifiers whose notes we control
    pub async fn filter_nullifiers(
        &self,
        nullifiers: Vec<Nullifier>,
    ) -> anyhow::Result<Vec<Nullifier>> {
        if nullifiers.is_empty() {
            return Ok(Vec::new());
        }
        Ok(sqlx::query_as::<_, SpendableNoteRecord>(
            format!(
                "SELECT notes.note_commitment,
                        spendable_notes.height_created,
                        notes.address,
                        notes.amount,
                        notes.asset_id,
                        notes.rseed,
                        spendable_notes.address_index,
                        spendable_notes.source,
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
        sct: &mut tct::Tree,
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

        // If the chain parameters have changed, update them.
        if let Some(params) = filtered_block.chain_parameters {
            let chain_params_bytes = &ChainParameters::encode_to_vec(&params)[..];
            sqlx::query!(
                "INSERT INTO chain_params (bytes) VALUES (?)",
                chain_params_bytes
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
            let rseed = note_record.note.rseed().to_bytes().to_vec();
            let address_index = note_record.address_index.to_bytes().to_vec();
            let nullifier = note_record.nullifier.to_bytes().to_vec();
            let position = (u64::from(note_record.position)) as i64;
            let source = note_record.source.to_bytes().to_vec();

            // We might have already seen the notes in the form of advice,
            // so we use ON CONFLICT DO NOTHING to skip re-inserting them
            // in that case.
            sqlx::query!(
                "INSERT INTO notes
                    (
                        note_commitment,
                        address,
                        amount,
                        asset_id,
                        rseed
                    )
                VALUES (?, ?, ?, ?, ?)
                ON CONFLICT DO NOTHING",
                note_commitment,
                address,
                amount,
                asset_id,
                rseed,
            )
            .execute(&mut dbtx)
            .await?;

            sqlx::query!(
                "INSERT INTO spendable_notes
                    (
                        note_commitment,
                        nullifier,
                        position,
                        height_created,
                        address_index,
                        source,
                        height_spent
                    )
                    VALUES
                    (?, ?, ?, ?, ?, ?, NULL)",
                note_commitment,
                nullifier,
                position,
                height_created,
                address_index,
                source,
                // height_spent is NULL
            )
            .execute(&mut dbtx)
            .await?;
        }

        for swap in &filtered_block.new_swaps {
            let swap_commitment = swap.swap_commitment.0.to_bytes().to_vec();
            let swap_bytes = swap.swap.encode_to_vec();
            let position = (u64::from(swap.position)) as i64;
            let nullifier = swap.nullifier.to_bytes().to_vec();
            let source = swap.source.to_bytes().to_vec();
            let output_data = swap.output_data.encode_to_vec();

            sqlx::query!(
                "INSERT INTO swaps (swap_commitment, swap, position, nullifier, output_data, height_claimed, source)
                VALUES (?, ?, ?, ?, ?, NULL, ?)",
                swap_commitment,
                swap_bytes,
                position,
                nullifier,
                output_data,
                // height_claimed is NULL
                source,
            ).execute(&mut dbtx).await?;
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
            // TODO: mark spent swaps as spent

            if let Some(bytes) = spent_commitment_bytes {
                // Forget spent note commitments from the SCT unless they are delegation tokens,
                // which must be saved to allow voting on proposals that might or might not be open
                // presently
                let spent_commitment = Commitment::try_from(
                    bytes
                        .note_commitment
                        .ok_or_else(|| anyhow!("missing note commitment"))?
                        .as_slice(),
                )?;

                // Check if it's a delegation token, and only forget it if it's not:
                let spent_denom: String = sqlx::query!(
                        "SELECT assets.denom
                        FROM spendable_notes JOIN notes LEFT JOIN assets ON notes.asset_id == assets.asset_id
                        WHERE nullifier = ?",
                        nullifier,
                    )
                    .fetch_one(&mut dbtx)
                    .await?
                    .denom
                    .ok_or_else(|| anyhow!("denom must exist for note we know about"))?;
                if DelegationToken::from_str(&spent_denom).is_ok() {
                    sct.forget(spent_commitment);
                }
            }
        }

        // Update SCT table with current SCT state
        sct.to_async_writer(&mut TreeStore(&mut dbtx)).await?;

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

        for nullifier in filtered_block.spent_nullifiers.iter() {
            // This will fail to be broadcast if there is no active receiver (such as on initial sync)
            // The error is ignored, as this isn't a problem, because if there is no active receiver there is nothing to do
            let _ = self.scanned_nullifiers_tx.send(*nullifier);
        }

        Ok(())
    }
}
