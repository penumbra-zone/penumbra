use std::{collections::BTreeMap, num::NonZeroU64, str::FromStr, sync::Arc, time::Duration};

use anyhow::{anyhow, Context};
use camino::Utf8Path;
use decaf377::Fq;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use penumbra_sdk_auction::auction::AuctionId;
use r2d2_sqlite::{
    rusqlite::{OpenFlags, OptionalExtension},
    SqliteConnectionManager,
};
use sha2::{Digest, Sha256};
use tap::{Tap, TapFallible};
use tokio::{
    sync::broadcast::{self, error::RecvError},
    task::spawn_blocking,
};
use tracing::{error_span, Instrument};
use url::Url;

use penumbra_sdk_app::params::AppParameters;
use penumbra_sdk_asset::{asset, asset::Id, asset::Metadata, Value};
use penumbra_sdk_dex::{
    lp::position::{self, Position, State},
    TradingPair,
};
use penumbra_sdk_fee::GasPrices;
use penumbra_sdk_keys::{keys::AddressIndex, Address, FullViewingKey};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{
    core::app::v1::{
        query_service_client::QueryServiceClient as AppQueryServiceClient, AppParametersRequest,
    },
    DomainType,
};
use penumbra_sdk_sct::{CommitmentSource, Nullifier};
use penumbra_sdk_shielded_pool::{fmd, note, Note, Rseed};
use penumbra_sdk_stake::{DelegationToken, IdentityKey};
use penumbra_sdk_tct as tct;
use penumbra_sdk_transaction::Transaction;
use sct::TreeStore;
use tct::StateCommitment;

use crate::{sync::FilteredBlock, SpendableNoteRecord, SwapRecord};

mod sct;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BalanceEntry {
    pub id: Id,
    pub amount: u128,
    pub address_index: AddressIndex,
}

/// The hash of the schema for the database.
static SCHEMA_HASH: Lazy<String> =
    Lazy::new(|| hex::encode(Sha256::digest(include_str!("storage/schema.sql"))));

#[derive(Clone)]
pub struct Storage {
    pool: r2d2::Pool<SqliteConnectionManager>,

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
    #[tracing::instrument(
        skip_all,
        fields(
            path = ?storage_path.as_ref().map(|p| p.as_ref().as_str()),
            url = %node,
        )
    )]
    pub async fn load_or_initialize(
        storage_path: Option<impl AsRef<Utf8Path>>,
        fvk: &FullViewingKey,
        node: Url,
    ) -> anyhow::Result<Self> {
        if let Some(path) = storage_path.as_ref().map(AsRef::as_ref) {
            if path.exists() {
                tracing::debug!(?path, "database exists");
                return Self::load(path).await;
            } else {
                tracing::debug!(?path, "database does not exist");
            }
        };

        let mut client = AppQueryServiceClient::connect(node.to_string())
            .instrument(error_span!("connecting_to_endpoint"))
            .await
            .tap_err(|error| {
                tracing::error!(?error, "failed to connect to app query service endpoint")
            })?
            .tap(|_| tracing::debug!("connected to app query service endpoint"));
        let params = client
            .app_parameters(tonic::Request::new(AppParametersRequest {}))
            .instrument(error_span!("getting_app_parameters"))
            .await?
            .into_inner()
            .try_into()?;

        Self::initialize(storage_path, fvk.clone(), params).await
    }

    fn connect(
        path: Option<impl AsRef<Utf8Path>>,
    ) -> anyhow::Result<r2d2::Pool<SqliteConnectionManager>> {
        if let Some(path) = path {
            let manager = SqliteConnectionManager::file(path.as_ref())
                .with_flags(
                    // Don't allow opening URIs, because they can change the behavior of the database; we
                    // just want to open normal filepaths.
                    OpenFlags::default() & !OpenFlags::SQLITE_OPEN_URI,
                )
                .with_init(|conn| {
                    // "NORMAL" will be consistent, but maybe not durable -- this is fine,
                    // since all our data is being synced from the chain, so if we lose a dbtx,
                    // it's like we're resuming sync from a previous height.
                    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
                    // We use `prepare_cached` a fair amount: this is an overestimate of the number
                    // of cached prepared statements likely to be used.
                    conn.set_prepared_statement_cache_capacity(32);
                    Ok(())
                });
            Ok(r2d2::Pool::builder()
                // We set max_size=1 to avoid "database is locked" sqlite errors,
                // when accessing across multiple threads.
                .max_size(1)
                .build(manager)?)
        } else {
            let manager = SqliteConnectionManager::memory();
            // Max size needs to be set to 1, otherwise a new in-memory database is created for each
            // connection to the pool, which results in very confusing errors.
            //
            // Lifetimes and timeouts are likewise configured to their maximum values, since
            // the in-memory database will disappear on connection close.
            Ok(r2d2::Pool::builder()
                .max_size(1)
                .min_idle(Some(1))
                .max_lifetime(Some(Duration::MAX))
                .idle_timeout(Some(Duration::MAX))
                .build(manager)?)
        }
    }

    pub async fn load(path: impl AsRef<Utf8Path>) -> anyhow::Result<Self> {
        let storage = Self {
            pool: Self::connect(Some(path))?,
            uncommitted_height: Arc::new(Mutex::new(None)),
            scanned_notes_tx: broadcast::channel(128).0,
            scanned_nullifiers_tx: broadcast::channel(512).0,
            scanned_swaps_tx: broadcast::channel(128).0,
        };

        spawn_blocking(move || {
            // Check the version of the software used when first initializing this database.
            // If it doesn't match the current version, we should report the error to the user.
            let actual_schema_hash: String = storage
                .pool
                .get()?
                .query_row("SELECT schema_hash FROM schema_hash", (), |row| {
                    row.get("schema_hash")
                })
                .context("failed to query database schema version: the database was probably created by an old client version, and needs to be reset and resynchronized")?;

            if actual_schema_hash != *SCHEMA_HASH {
                let database_client_version: String = storage
                    .pool
                    .get()?
                    .query_row("SELECT client_version FROM client_version", (), |row| {
                        row.get("client_version")
                    })
                    .context("failed to query client version: the database was probably created by an old client version, and needs to be reset and resynchronized")?;

                anyhow::bail!(
                    "can't load view database created by client version {} using client version {}: they have different schemata, so you need to reset your view database and resynchronize by running pcli view reset",
                    database_client_version,
                    env!("CARGO_PKG_VERSION"),
                );
            }

            Ok(storage)
        })
            .await?
    }

    pub async fn initialize(
        storage_path: Option<impl AsRef<Utf8Path>>,
        fvk: FullViewingKey,
        params: AppParameters,
    ) -> anyhow::Result<Self> {
        tracing::debug!(storage_path = ?storage_path.as_ref().map(AsRef::as_ref), ?fvk, ?params);

        // Connect to the database (or create it)
        let pool = Self::connect(storage_path)?;

        spawn_blocking(move || {
            // In one database transaction, populate everything
            let mut conn = pool.get()?;
            let tx = conn.transaction()?;

            // Create the tables
            tx.execute_batch(include_str!("storage/schema.sql"))?;

            let params_bytes = params.encode_to_vec();
            tx.execute(
                "INSERT INTO kv (k, v) VALUES ('app_params', ?1)",
                [&params_bytes[..]],
            )?;

            let fvk_bytes = fvk.encode_to_vec();
            tx.execute("INSERT INTO kv (k, v) VALUES ('fvk', ?1)", [&fvk_bytes[..]])?;

            // Insert -1 as a signaling value for pre-genesis.
            // We just have to be careful to treat negative values as None
            // in last_sync_height.
            tx.execute("INSERT INTO sync_height (height) VALUES (-1)", ())?;

            // Insert the schema hash into the database
            tx.execute(
                "INSERT INTO schema_hash (schema_hash) VALUES (?1)",
                [&*SCHEMA_HASH],
            )?;

            // Insert the client version into the database
            tx.execute(
                "INSERT INTO client_version (client_version) VALUES (?1)",
                [env!("CARGO_PKG_VERSION")],
            )?;

            tx.commit()?;
            drop(conn);

            Ok(Storage {
                pool,
                uncommitted_height: Arc::new(Mutex::new(None)),
                scanned_notes_tx: broadcast::channel(128).0,
                scanned_nullifiers_tx: broadcast::channel(512).0,
                scanned_swaps_tx: broadcast::channel(128).0,
            })
        })
        .await?
    }

    /// Loads asset metadata from a JSON file and use to update the database.
    pub async fn load_asset_metadata(
        &self,
        registry_path: impl AsRef<Utf8Path>,
    ) -> anyhow::Result<()> {
        tracing::debug!(registry_path = ?registry_path.as_ref(), "loading asset metadata");
        let registry_path = registry_path.as_ref();
        // Parse into a serde_json::Value first so we can get the bits we care about
        let mut registry_json: serde_json::Value = serde_json::from_str(
            std::fs::read_to_string(registry_path)
                .context("failed to read file")?
                .as_str(),
        )
        .context("failed to parse JSON")?;

        let registry: BTreeMap<String, Metadata> = serde_json::value::from_value(
            registry_json
                .get_mut("assetById")
                .ok_or_else(|| anyhow::anyhow!("missing assetById"))?
                .take(),
        )
        .context("could not parse asset registry")?;

        for metadata in registry.into_values() {
            self.record_asset(metadata).await?;
        }

        Ok(())
    }

    /// Query for account balance by address
    pub async fn balances(
        &self,
        address_index: Option<AddressIndex>,
        asset_id: Option<asset::Id>,
    ) -> anyhow::Result<Vec<BalanceEntry>> {
        let pool = self.pool.clone();

        spawn_blocking(move || {
            let query = "SELECT notes.asset_id, notes.amount, spendable_notes.address_index
                FROM    notes
                JOIN    spendable_notes ON notes.note_commitment = spendable_notes.note_commitment
                WHERE   spendable_notes.height_spent IS NULL";

            tracing::debug!(?query);

            // Combine notes of the same asset/address index together
            let mut balances: BTreeMap<AddressIndex, BTreeMap<asset::Id, Amount>> = BTreeMap::new();

            for result in pool.get()?.prepare_cached(query)?.query_map([], |row| {
                let asset_id = row.get::<&str, Vec<u8>>("asset_id")?;
                let amount = row.get::<&str, Vec<u8>>("amount")?;
                let address_index = row.get::<&str, Vec<u8>>("address_index")?;

                Ok((asset_id, amount, address_index))
            })? {
                let (id, amount, index) = result?;

                let id = Id::try_from(id.as_slice())?;

                let amount: Amount = Amount::from_be_bytes(
                    amount
                        .as_slice()
                        .try_into()
                        .expect("amount slice of incorrect length"),
                );

                let index = AddressIndex::try_from(index.as_slice())?;

                // Skip this entry if not captured by address index filter
                if let Some(address_index) = address_index {
                    if address_index != index {
                        continue;
                    }
                }
                if let Some(asset_id) = asset_id {
                    if asset_id != id {
                        continue;
                    }
                }

                balances
                    .entry(index)
                    .or_insert_with(BTreeMap::new)
                    .entry(id)
                    .and_modify(|e| *e += amount)
                    .or_insert(amount);
            }

            let entries = balances
                .into_iter()
                .flat_map(|(index, assets)| {
                    assets.into_iter().map(move |(id, amount)| BalanceEntry {
                        id,
                        amount: amount.into(),
                        address_index: index,
                    })
                })
                .collect::<Vec<_>>();
            Ok(entries)
        })
        .await?
    }

    /// Query for a note by its note commitment, optionally waiting until the note is detected.
    pub async fn note_by_commitment(
        &self,
        note_commitment: tct::StateCommitment,
        await_detection: bool,
    ) -> anyhow::Result<SpendableNoteRecord> {
        // Start subscribing now, before querying for whether we already
        // have the record, so that we can't miss it if we race a write.
        let mut rx = self.scanned_notes_tx.subscribe();

        let pool = self.pool.clone();

        if let Some(record) = spawn_blocking(move || {
            // Check if we already have the record
            pool.get()?
                .prepare(&format!(
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
                        spendable_notes.position,
                        tx.return_address
                    FROM notes
                    JOIN spendable_notes ON notes.note_commitment = spendable_notes.note_commitment
                    LEFT JOIN tx ON spendable_notes.tx_hash = tx.tx_hash
                    WHERE notes.note_commitment = x'{}'",
                    hex::encode(note_commitment.0.to_bytes())
                ))?
                .query_and_then((), |record| record.try_into())?
                .next()
                .transpose()
        })
        .await??
        {
            return Ok(record);
        }

        if !await_detection {
            anyhow::bail!("Note commitment {} not found", note_commitment);
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
                        anyhow::bail!(
                            "Receiver error during note detection: closed (no more active senders)"
                        );
                    }
                    RecvError::Lagged(count) => {
                        anyhow::bail!(
                            "Receiver error during note detection: lagged (by {:?} messages)",
                            count
                        );
                    }
                },
            };
        }
    }

    /// Query for a swap by its swap commitment, optionally waiting until the note is detected.
    pub async fn swap_by_commitment(
        &self,
        swap_commitment: tct::StateCommitment,
        await_detection: bool,
    ) -> anyhow::Result<SwapRecord> {
        // Start subscribing now, before querying for whether we already
        // have the record, so that we can't miss it if we race a write.
        let mut rx = self.scanned_swaps_tx.subscribe();

        let pool = self.pool.clone();

        if let Some(record) = spawn_blocking(move || {
            // Check if we already have the swap record
            pool.get()?
                .prepare(&format!(
                    "SELECT * FROM swaps WHERE swaps.swap_commitment = x'{}'",
                    hex::encode(swap_commitment.0.to_bytes())
                ))?
                .query_and_then((), |record| record.try_into())?
                .next()
                .transpose()
        })
        .await??
        {
            return Ok(record);
        }

        if !await_detection {
            anyhow::bail!("swap commitment {} not found", swap_commitment);
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
                        anyhow::bail!(
                            "Receiver error during swap detection: closed (no more active senders)"
                        );
                    }
                    RecvError::Lagged(count) => {
                        anyhow::bail!(
                            "Receiver error during swap detection: lagged (by {:?} messages)",
                            count
                        );
                    }
                },
            };
        }
    }

    /// Query for all unclaimed swaps.
    pub async fn unclaimed_swaps(&self) -> anyhow::Result<Vec<SwapRecord>> {
        let pool = self.pool.clone();

        let records = spawn_blocking(move || {
            // Check if we already have the swap record
            pool.get()?
                .prepare("SELECT * FROM swaps WHERE swaps.height_claimed is NULL")?
                .query_and_then((), |record| record.try_into())?
                .collect::<anyhow::Result<Vec<_>>>()
        })
        .await??;

        Ok(records)
    }

    /// Query for a nullifier's status, optionally waiting until the nullifier is detected.
    pub async fn nullifier_status(
        &self,
        nullifier: Nullifier,
        await_detection: bool,
    ) -> anyhow::Result<bool> {
        // Start subscribing now, before querying for whether we already have the nullifier, so we
        // can't miss it if we race a write.
        let mut rx = self.scanned_nullifiers_tx.subscribe();

        // Clone the pool handle so that the returned future is 'static
        let pool = self.pool.clone();

        let nullifier_bytes = nullifier.0.to_bytes().to_vec();

        // Check if we already have the nullifier in the set of spent notes
        if let Some(height_spent) = spawn_blocking(move || {
            pool.get()?
                .prepare_cached("SELECT height_spent FROM spendable_notes WHERE nullifier = ?1")?
                .query_and_then([nullifier_bytes], |row| {
                    let height_spent: Option<u64> = row.get("height_spent")?;
                    anyhow::Ok(height_spent)
                })?
                .next()
                .transpose()
        })
        .await??
        {
            let spent = height_spent.is_some();

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
            let new_nullifier = rx.recv().await.context("change subscriber failed")?;

            if new_nullifier == nullifier {
                return Ok(true);
            }
        }
    }

    /// The last block height we've scanned to, if any.
    pub async fn last_sync_height(&self) -> anyhow::Result<Option<u64>> {
        // Check if we have uncommitted blocks beyond the database height.
        if let Some(height) = *self.uncommitted_height.lock() {
            return Ok(Some(height.get()));
        }

        let pool = self.pool.clone();

        spawn_blocking(move || {
            let height: Option<i64> = pool
                .get()?
                .prepare_cached("SELECT height FROM sync_height ORDER BY height DESC LIMIT 1")?
                .query_row([], |row| row.get::<_, Option<i64>>(0))?;

            anyhow::Ok(u64::try_from(height.ok_or_else(|| anyhow!("missing sync height"))?).ok())
        })
        .await?
    }

    pub async fn app_params(&self) -> anyhow::Result<AppParameters> {
        let pool = self.pool.clone();

        spawn_blocking(move || {
            let params_bytes = pool
                .get()?
                .prepare_cached("SELECT v FROM kv WHERE k IS 'app_params' LIMIT 1")?
                .query_row([], |row| row.get::<_, Option<Vec<u8>>>("v"))?
                .ok_or_else(|| anyhow!("missing app_params in kv table"))?;

            AppParameters::decode(params_bytes.as_slice())
        })
        .await?
    }

    pub async fn gas_prices(&self) -> anyhow::Result<GasPrices> {
        let pool = self.pool.clone();

        spawn_blocking(move || {
            let bytes = pool
                .get()?
                .prepare_cached("SELECT v FROM kv WHERE k IS 'gas_prices' LIMIT 1")?
                .query_row([], |row| row.get::<_, Option<Vec<u8>>>("v"))?
                .ok_or_else(|| anyhow!("missing gas_prices in kv table"))?;

            GasPrices::decode(bytes.as_slice())
        })
        .await?
    }

    pub async fn fmd_parameters(&self) -> anyhow::Result<fmd::Parameters> {
        let pool = self.pool.clone();

        spawn_blocking(move || {
            let bytes = pool
                .get()?
                .prepare_cached("SELECT v FROM kv WHERE k IS 'fmd_params' LIMIT 1")?
                .query_row([], |row| row.get::<_, Option<Vec<u8>>>("v"))?
                .ok_or_else(|| anyhow!("missing fmd_params in kv table"))?;

            fmd::Parameters::decode(bytes.as_slice())
        })
        .await?
    }

    pub async fn full_viewing_key(&self) -> anyhow::Result<FullViewingKey> {
        let pool = self.pool.clone();

        spawn_blocking(move || {
            let bytes = pool
                .get()?
                .prepare_cached("SELECT v FROM kv WHERE k is 'fvk' LIMIT 1")?
                .query_row([], |row| row.get::<_, Option<Vec<u8>>>("v"))?
                .ok_or_else(|| anyhow!("missing fvk in kv table"))?;

            FullViewingKey::decode(bytes.as_slice())
        })
        .await?
    }

    pub async fn state_commitment_tree(&self) -> anyhow::Result<tct::Tree> {
        let pool = self.pool.clone();
        spawn_blocking(move || {
            tct::Tree::from_reader(&mut TreeStore(&mut pool.get()?.transaction()?))
        })
        .await?
    }

    /// Returns a tuple of (block height, transaction hash) for all transactions in a given range of block heights.
    pub async fn transaction_hashes(
        &self,
        start_height: Option<u64>,
        end_height: Option<u64>,
    ) -> anyhow::Result<Vec<(u64, Vec<u8>)>> {
        let starting_block = start_height.unwrap_or(0) as i64;
        let ending_block = end_height.unwrap_or(self.last_sync_height().await?.unwrap_or(0)) as i64;

        let pool = self.pool.clone();

        spawn_blocking(move || {
            pool.get()?
                .prepare_cached(
                    "SELECT block_height, tx_hash
                    FROM tx
                    WHERE block_height BETWEEN ?1 AND ?2",
                )?
                .query_and_then([starting_block, ending_block], |row| {
                    let block_height: u64 = row.get("block_height")?;
                    let tx_hash: Vec<u8> = row.get("tx_hash")?;
                    anyhow::Ok((block_height, tx_hash))
                })?
                .collect()
        })
        .await?
    }

    /// Returns a tuple of (block height, transaction hash, transaction) for all transactions in a given range of block heights.
    pub async fn transactions(
        &self,
        start_height: Option<u64>,
        end_height: Option<u64>,
    ) -> anyhow::Result<Vec<(u64, Vec<u8>, Transaction)>> {
        let starting_block = start_height.unwrap_or(0) as i64;
        let ending_block = end_height.unwrap_or(self.last_sync_height().await?.unwrap_or(0)) as i64;

        let pool = self.pool.clone();

        spawn_blocking(move || {
            pool.get()?
                .prepare_cached(
                    "SELECT block_height, tx_hash, tx_bytes
                    FROM tx
                    WHERE block_height BETWEEN ?1 AND ?2",
                )?
                .query_and_then([starting_block, ending_block], |row| {
                    let block_height: u64 = row.get("block_height")?;
                    let tx_hash: Vec<u8> = row.get("tx_hash")?;
                    let tx_bytes: Vec<u8> = row.get("tx_bytes")?;
                    let tx = Transaction::decode(tx_bytes.as_slice())?;
                    anyhow::Ok((block_height, tx_hash, tx))
                })?
                .collect()
        })
        .await?
    }

    pub async fn transaction_by_hash(
        &self,
        tx_hash: &[u8],
    ) -> anyhow::Result<Option<(u64, Transaction)>> {
        let pool = self.pool.clone();
        let tx_hash = tx_hash.to_vec();

        spawn_blocking(move || {
            if let Some((block_height, tx_bytes)) = pool
                .get()?
                .prepare_cached("SELECT block_height, tx_bytes FROM tx WHERE tx_hash = ?1")?
                .query_row([tx_hash], |row| {
                    let block_height: u64 = row.get("block_height")?;
                    let tx_bytes: Vec<u8> = row.get("tx_bytes")?;
                    Ok((block_height, tx_bytes))
                })
                .optional()?
            {
                let tx = Transaction::decode(tx_bytes.as_slice())?;
                Ok(Some((block_height, tx)))
            } else {
                Ok(None)
            }
        })
        .await?
    }

    // Query for a note by its note commitment, optionally waiting until the note is detected.
    pub async fn note_by_nullifier(
        &self,
        nullifier: Nullifier,
        await_detection: bool,
    ) -> anyhow::Result<SpendableNoteRecord> {
        // Start subscribing now, before querying for whether we already
        // have the record, so that we can't miss it if we race a write.
        let mut rx = self.scanned_notes_tx.subscribe();

        // Clone the pool handle so that the returned future is 'static
        let pool = self.pool.clone();

        let nullifier_bytes = nullifier.to_bytes().to_vec();

        if let Some(record) = spawn_blocking(move || {
            let record = pool
                .get()?
                .prepare(&format!(
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
                        spendable_notes.position,
                        tx.return_address
                    FROM notes
                    JOIN spendable_notes ON notes.note_commitment = spendable_notes.note_commitment
                    LEFT JOIN tx ON spendable_notes.tx_hash = tx.tx_hash
                    WHERE hex(spendable_notes.nullifier) = \"{}\"",
                    hex::encode_upper(nullifier_bytes)
                ))?
                .query_and_then((), |row| SpendableNoteRecord::try_from(row))?
                .next()
                .transpose()?;

            anyhow::Ok(record)
        })
        .await??
        {
            return Ok(record);
        }

        if !await_detection {
            anyhow::bail!("Note commitment for nullifier {:?} not found", nullifier);
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
                        anyhow::bail!(
                            "Receiver error during note detection: closed (no more active senders)"
                        );
                    }
                    RecvError::Lagged(count) => {
                        anyhow::bail!(
                            "Receiver error during note detection: lagged (by {:?} messages)",
                            count
                        );
                    }
                },
            };
        }
    }

    pub async fn all_assets(&self) -> anyhow::Result<Vec<Metadata>> {
        let pool = self.pool.clone();

        spawn_blocking(move || {
            pool.get()?
                .prepare_cached("SELECT metadata FROM assets")?
                .query_and_then([], |row| {
                    let metadata_json = row.get::<_, String>("metadata")?;
                    let denom_metadata = serde_json::from_str(&metadata_json)?;

                    anyhow::Ok(denom_metadata)
                })?
                .collect()
        })
        .await?
    }

    pub async fn asset_by_id(&self, id: &Id) -> anyhow::Result<Option<Metadata>> {
        let id = id.to_bytes().to_vec();

        let pool = self.pool.clone();

        spawn_blocking(move || {
            pool.get()?
                .prepare_cached("SELECT metadata FROM assets WHERE asset_id = ?1")?
                .query_and_then([id], |row| {
                    let metadata_json = row.get::<_, String>("metadata")?;
                    let denom_metadata = serde_json::from_str(&metadata_json)?;
                    anyhow::Ok(denom_metadata)
                })?
                .next()
                .transpose()
        })
        .await?
    }

    // Get assets whose denoms match the given SQL LIKE pattern, with the `_` and `%` wildcards,
    // where `\` is the escape character.
    pub async fn assets_matching(&self, pattern: String) -> anyhow::Result<Vec<Metadata>> {
        let pattern = pattern.to_owned();

        let pool = self.pool.clone();

        spawn_blocking(move || {
            pool.get()?
                .prepare_cached("SELECT metadata FROM assets WHERE denom LIKE ?1 ESCAPE '\\'")?
                .query_and_then([pattern], |row| {
                    let metadata_json = row.get::<_, String>("metadata")?;
                    let denom_metadata = serde_json::from_str(&metadata_json)?;
                    anyhow::Ok(denom_metadata)
                })?
                .collect()
        })
        .await?
    }

    pub async fn notes(
        &self,
        include_spent: bool,
        asset_id: Option<asset::Id>,
        address_index: Option<penumbra_sdk_keys::keys::AddressIndex>,
        amount_to_spend: Option<Amount>,
    ) -> anyhow::Result<Vec<SpendableNoteRecord>> {
        // If set, return spent notes as well as unspent notes.
        // bool include_spent = 2;
        let spent_clause = match include_spent {
            false => "NULL",
            true => "height_spent",
        };

        // If set, only return notes with the specified asset id.
        // core.crypto.v1.AssetId asset_id = 3;
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

        // If set, stop returning notes once the total exceeds this amount.
        //
        // Ignored if `asset_id` is unset or if `include_spent` is set.
        // uint64 amount_to_spend = 5;
        //TODO: figure out a clever way to only return notes up to the sum using SQL
        let amount_cutoff = (amount_to_spend.is_some()) && !(include_spent || asset_id.is_none());
        let mut amount_total = Amount::zero();

        let pool = self.pool.clone();

        spawn_blocking(move || {
            let mut output: Vec<SpendableNoteRecord> = Vec::new();

            for result in pool
                .get()?
                .prepare(&format!(
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
                        spendable_notes.position,
                        tx.return_address
                FROM notes
                JOIN spendable_notes ON notes.note_commitment = spendable_notes.note_commitment
                LEFT JOIN tx ON spendable_notes.tx_hash = tx.tx_hash
                WHERE spendable_notes.height_spent IS {spent_clause}
                AND notes.asset_id IS {asset_clause}
                AND spendable_notes.address_index IS {address_clause}"
                ))?
                .query_and_then((), |row| SpendableNoteRecord::try_from(row))?
            {
                let record = result?;

                // Skip notes that don't match the account, since we're
                // not doing account filtering in SQL as a temporary hack (see above)
                if let Some(address_index) = address_index {
                    if record.address_index.account != address_index.account {
                        continue;
                    }
                }
                let amount = record.note.amount();

                // Only display notes of value > 0

                if amount.value() > 0 {
                    output.push(record);
                }

                // If we're tracking amounts, accumulate the value of the note
                // and check if we should break out of the loop.
                if amount_cutoff {
                    // We know all the notes are of the same type, so adding raw quantities makes sense.
                    amount_total += amount;
                    if amount_total >= amount_to_spend.unwrap_or_default() {
                        break;
                    }
                }
            }

            if amount_total < amount_to_spend.unwrap_or_default() {
                anyhow::bail!(
                    "requested amount of {} exceeds total of {}",
                    amount_to_spend.unwrap_or_default(),
                    amount_total
                );
            }

            anyhow::Ok(output)
        })
        .await?
    }

    pub async fn notes_for_voting(
        &self,
        address_index: Option<penumbra_sdk_keys::keys::AddressIndex>,
        votable_at_height: u64,
    ) -> anyhow::Result<Vec<(SpendableNoteRecord, IdentityKey)>> {
        // If set, only return notes with the specified address index.
        // crypto.AddressIndex address_index = 3;
        let address_clause = address_index
            .map(|d| format!("x'{}'", hex::encode(d.to_bytes())))
            .unwrap_or_else(|| "address_index".to_string());

        let pool = self.pool.clone();

        spawn_blocking(move || {
            let mut lock = pool.get()?;
            let dbtx = lock.transaction()?;

            let spendable_note_records: Vec<SpendableNoteRecord> = dbtx
                .prepare(&format!(
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
                ))?
                .query_and_then((), |row| row.try_into())?
                .collect::<anyhow::Result<Vec<_>>>()?;

            // TODO: this could be internalized into the SQL query in principle, but it's easier to
            // do it this way; if it becomes slow, we can do it better
            let mut results = Vec::new();
            for record in spendable_note_records {
                let asset_id = record.note.asset_id().to_bytes().to_vec();
                let denom: String = dbtx.query_row_and_then(
                    "SELECT denom FROM assets WHERE asset_id = ?1",
                    [asset_id],
                    |row| row.get("denom"),
                )?;

                let identity_key = DelegationToken::from_str(&denom)
                    .context("invalid delegation token denom")?
                    .validator();

                results.push((record, identity_key));
            }

            Ok(results)
        }).await?
    }

    #[tracing::instrument(skip(self))]
    pub async fn record_asset(&self, asset: Metadata) -> anyhow::Result<()> {
        tracing::debug!(?asset);

        let asset_id = asset.id().to_bytes().to_vec();
        let denom = asset.base_denom().denom;
        let metadata_json = serde_json::to_string(&asset)?;

        let pool = self.pool.clone();

        spawn_blocking(move || {
            pool.get()?
                .execute(
                    "INSERT OR REPLACE INTO assets (asset_id, denom, metadata) VALUES (?1, ?2, ?3)",
                    (asset_id, denom, metadata_json),
                )
                .map_err(anyhow::Error::from)
        })
        .await??;

        Ok(())
    }

    pub async fn record_auction_with_state(
        &self,
        auction_id: AuctionId,
        auction_state: u64,
    ) -> anyhow::Result<()> {
        let auction_id = auction_id.0.to_vec();

        let pool = self.pool.clone();

        spawn_blocking(move || {
            let mut lock = pool.get()?;
            let tx = lock.transaction()?;
            tx.execute(
                "INSERT OR IGNORE INTO auctions (auction_id, auction_state, note_commitment) VALUES (?1, ?2, NULL)",
                (auction_id.clone(), auction_state),
            )?;
            tx.execute(
                "UPDATE auctions SET auction_state = ?2 WHERE auction_id = ?1",
                (auction_id, auction_state),
            )
                .map_err(anyhow::Error::from)?;

            tx.commit()?;
            Ok::<(), anyhow::Error>(())
            })
            .await??;

        Ok(())
    }

    pub async fn update_auction_with_note_commitment(
        &self,
        auction_id: AuctionId,
        note_commitment: StateCommitment,
    ) -> anyhow::Result<()> {
        let auction_id = auction_id.0.to_vec();
        let blob_nc = note_commitment.0.to_bytes().to_vec();

        let pool = self.pool.clone();

        spawn_blocking(move || {
            pool.get()?
                .execute(
                    "UPDATE auctions SET (note_commitment) = ?1 WHERE auction_id = ?2",
                    (blob_nc, auction_id),
                )
                .map_err(anyhow::Error::from)
        })
        .await??;

        Ok(())
    }

    pub async fn fetch_auctions_by_account(
        &self,
        account_filter: Option<AddressIndex>,
        include_inactive: bool,
    ) -> anyhow::Result<Vec<(AuctionId, SpendableNoteRecord, u64 /* local seqnum */)>> {
        let account_clause = account_filter
            .map(|idx| {
                format!(
                    "AND spendable_notes.address_index = x'{}'",
                    hex::encode(idx.to_bytes())
                )
            })
            .unwrap_or_else(|| "".to_string());

        let active_clause = if !include_inactive {
            "AND auctions.auction_state = 0"
        } else {
            ""
        };

        let query = format!(
            "SELECT auctions.auction_id, spendable_notes.*, notes.*, auctions.auction_state
                 FROM auctions
                 JOIN spendable_notes ON auctions.note_commitment = spendable_notes.note_commitment
                 JOIN notes ON auctions.note_commitment = notes.note_commitment
                 WHERE 1 = 1
                 {account_clause}
                 {active_clause}",
            account_clause = account_clause,
            active_clause = active_clause,
        );

        let pool = self.pool.clone();

        spawn_blocking(move || {
            let mut conn = pool.get()?;
            let tx = conn.transaction()?;

            let spendable_note_records: Vec<(AuctionId, SpendableNoteRecord, u64)> = tx
                .prepare(&query)?
                .query_and_then((), |row| {
                    let raw_auction_id: Vec<u8> = row.get("auction_id")?;
                    let array_auction_id: [u8; 32] = raw_auction_id
                        .try_into()
                        .map_err(|_| anyhow!("auction id must be 32 bytes"))?;
                    let auction_id = AuctionId(array_auction_id);
                    let spendable_note_record: SpendableNoteRecord = row.try_into()?;
                    let local_seq: u64 = row.get("auction_state")?;
                    Ok((auction_id, spendable_note_record, local_seq))
                })?
                .collect::<anyhow::Result<Vec<_>>>()?;

            Ok(spendable_note_records)
        })
        .await?
    }

    pub async fn record_position(&self, position: Position) -> anyhow::Result<()> {
        let position_id = position.id().0.to_vec();

        let position_state = position.state.to_string();
        let trading_pair = position.phi.pair.to_string();

        let pool = self.pool.clone();

        spawn_blocking(move || {
            pool.get()?
                .execute(
                    "INSERT OR REPLACE INTO positions (position_id, position_state, trading_pair) VALUES (?1, ?2, ?3)",
                    (position_id, position_state, trading_pair),
                )
                .map_err(anyhow::Error::from)
        })
            .await??;

        Ok(())
    }

    pub async fn update_position(
        &self,
        position_id: position::Id,
        position_state: position::State,
    ) -> anyhow::Result<()> {
        let position_id = position_id.0.to_vec();
        let position_state = position_state.to_string();

        let pool = self.pool.clone();

        spawn_blocking(move || {
            pool.get()?
                .execute(
                    "UPDATE positions SET (position_state) = ?1 WHERE position_id = ?2",
                    (position_state, position_id),
                )
                .map_err(anyhow::Error::from)
        })
        .await??;

        Ok(())
    }

    pub async fn record_empty_block(&self, height: u64) -> anyhow::Result<()> {
        // Check that the incoming block height follows the latest recorded height
        let last_sync_height = self.last_sync_height().await?.ok_or_else(|| {
            anyhow::anyhow!("invalid: tried to record empty block as genesis block")
        })?;

        if height != last_sync_height + 1 {
            anyhow::bail!(
                "Wrong block height {} for latest sync height {}",
                height,
                last_sync_height
            );
        }

        *self.uncommitted_height.lock() = Some(height.try_into()?);
        Ok(())
    }

    fn record_note_inner(
        dbtx: &r2d2_sqlite::rusqlite::Transaction<'_>,
        note: &Note,
    ) -> anyhow::Result<()> {
        let note_commitment = note.commit().0.to_bytes().to_vec();
        let address = note.address().to_vec();
        let amount = u128::from(note.amount()).to_be_bytes().to_vec();
        let asset_id = note.asset_id().to_bytes().to_vec();
        let rseed = note.rseed().to_bytes().to_vec();

        dbtx.execute(
            "INSERT INTO notes (note_commitment, address, amount, asset_id, rseed)
                VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT (note_commitment)
                DO UPDATE SET
                address = excluded.address,
                amount = excluded.amount,
                asset_id = excluded.asset_id,
                rseed = excluded.rseed",
            (note_commitment, address, amount, asset_id, rseed),
        )?;

        Ok(())
    }

    pub async fn give_advice(&self, note: Note) -> anyhow::Result<()> {
        let pool = self.pool.clone();
        let mut lock = pool.get()?;
        let dbtx = lock.transaction()?;

        Storage::record_note_inner(&dbtx, &note)?;

        dbtx.commit()?;

        Ok(())
    }

    /// Return advice about note contents for use in scanning.
    ///
    /// Given a list of note commitments, this method checks whether any of them
    /// correspond to notes that have been recorded in the database but not yet
    /// observed during scanning.
    pub async fn scan_advice(
        &self,
        note_commitments: Vec<note::StateCommitment>,
    ) -> anyhow::Result<BTreeMap<note::StateCommitment, Note>> {
        if note_commitments.is_empty() {
            return Ok(BTreeMap::new());
        }

        let pool = self.pool.clone();

        // This query gives advice about notes which are known but which have not already been recorded as spendable,
        // in part to avoid revealing information about which notes have been spent.

        spawn_blocking(move || {
            pool.get()?
                .prepare(&format!(
                    "SELECT notes.note_commitment,
                        notes.address,
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
                ))?
                .query_and_then((), |row| {
                    let address = Address::try_from(row.get::<_, Vec<u8>>("address")?)?;
                    let amount = row.get::<_, [u8; 16]>("amount")?;
                    let amount_u128: u128 = u128::from_be_bytes(amount);
                    let asset_id = asset::Id(Fq::from_bytes_checked(&row.get::<_, [u8; 32]>("asset_id")?).expect("asset id malformed"));
                    let rseed = Rseed(row.get::<_, [u8; 32]>("rseed")?);
                    let note = Note::from_parts(
                        address,
                        Value {
                            amount: amount_u128.into(),
                            asset_id,
                        },
                        rseed,
                    )?;
                    anyhow::Ok((note.commit(), note))
                })?
                .collect::<anyhow::Result<BTreeMap<_, _>>>()
        }).await?
    }

    /// Filters for nullifiers whose notes we control
    pub async fn filter_nullifiers(
        &self,
        nullifiers: Vec<Nullifier>,
    ) -> anyhow::Result<Vec<Nullifier>> {
        if nullifiers.is_empty() {
            return Ok(Vec::new());
        }

        let pool = self.pool.clone();

        spawn_blocking(move || {
            pool.get()?
                .prepare(&format!(
                    "SELECT nullifier FROM (SELECT nullifier FROM spendable_notes UNION SELECT nullifier FROM swaps UNION SELECT nullifier FROM tx_by_nullifier) WHERE nullifier IN ({})",
                    nullifiers
                        .iter()
                        .map(|x| format!("x'{}'", hex::encode(x.0.to_bytes())))
                        .collect::<Vec<String>>()
                        .join(",")
                ))?
                .query_and_then((), |row| {
                    let nullifier: Vec<u8> = row.get("nullifier")?;
                    nullifier.as_slice().try_into()
                })?
                .collect()
        })
            .await?
    }

    pub async fn record_block(
        &self,
        filtered_block: FilteredBlock,
        transactions: Vec<Transaction>,
        sct: &mut tct::Tree,
        channel: tonic::transport::Channel,
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
            anyhow::bail!(
                "Wrong block height {} for latest sync height {:?}",
                filtered_block.height,
                last_sync_height
            );
        }

        let pool = self.pool.clone();
        let uncommitted_height = self.uncommitted_height.clone();
        let scanned_notes_tx = self.scanned_notes_tx.clone();
        let scanned_nullifiers_tx = self.scanned_nullifiers_tx.clone();
        let scanned_swaps_tx = self.scanned_swaps_tx.clone();

        let fvk = self.full_viewing_key().await?;

        // If the app parameters have changed, update them.
        let new_app_parameters: Option<AppParameters> = if filtered_block.app_parameters_updated {
            // Fetch the latest parameters
            let mut client = AppQueryServiceClient::new(channel);
            Some(
                client
                    .app_parameters(tonic::Request::new(AppParametersRequest {}))
                    .await?
                    .into_inner()
                    .try_into()?,
            )
        } else {
            None
        };

        // Cloning the SCT is cheap because it's a copy-on-write structure, so we move an owned copy
        // into the spawned thread. This means that if for any reason the thread panics or throws an
        // error, the changes to the SCT will be discarded, just like any changes to the database,
        // so the two stay transactionally in sync, even in the case of errors. This would not be
        // the case if we `std::mem::take` the SCT and move it into the spawned thread, because then
        // an error would mean the updated version would never be put back, and the outcome would be
        // a cleared SCT but a non-empty database.
        let mut new_sct = sct.clone();

        *sct = spawn_blocking(move || {
            let mut lock = pool.get()?;
            let mut dbtx = lock.transaction()?;

            if let Some(params) = new_app_parameters {
                let params_bytes = params.encode_to_vec();
                // We expect app_params to be present already but may as well use an upsert
                dbtx.execute(
                    "INSERT INTO kv (k, v) VALUES ('app_params', ?1)
                    ON CONFLICT(k) DO UPDATE SET v = excluded.v",
                    [&params_bytes[..]],
                )?;
            }

            // Insert new note records into storage
            for note_record in filtered_block.new_notes.values() {
                let note_commitment = note_record.note_commitment.0.to_bytes().to_vec();
                let height_created = filtered_block.height as i64;
                let address_index = note_record.address_index.to_bytes().to_vec();
                let nullifier = note_record.nullifier.to_bytes().to_vec();
                let position = (u64::from(note_record.position)) as i64;
                let source = note_record.source.encode_to_vec();
                // Check if the note is from a transaction, if so, include the tx hash (id)
                let tx_hash = match note_record.source {
                    CommitmentSource::Transaction { id } => id,
                    _ => None,
                };

                // Record the inner note data in the notes table
                Storage::record_note_inner(&dbtx, &note_record.note)?;

                dbtx.execute(
                    "INSERT INTO spendable_notes
                    (note_commitment, nullifier, position, height_created, address_index, source, height_spent, tx_hash)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7)
                    ON CONFLICT (note_commitment)
                    DO UPDATE SET nullifier = excluded.nullifier,
                    position = excluded.position,
                    height_created = excluded.height_created,
                    address_index = excluded.address_index,
                    source = excluded.source,
                    height_spent = excluded.height_spent,
                    tx_hash = excluded.tx_hash",
                    (
                        &note_commitment,
                        &nullifier,
                        &position,
                        &height_created,
                        &address_index,
                        &source,
                        // height_spent is NULL because the note is newly discovered
                        &tx_hash,
                    ),
                )?;
            }

            // Insert new swap records into storage
            for swap in filtered_block.new_swaps.values() {
                let swap_commitment = swap.swap_commitment.0.to_bytes().to_vec();
                let swap_bytes = swap.swap.encode_to_vec();
                let position = (u64::from(swap.position)) as i64;
                let nullifier = swap.nullifier.to_bytes().to_vec();
                let source = swap.source.encode_to_vec();
                let output_data = swap.output_data.encode_to_vec();

                dbtx.execute(
                    "INSERT INTO swaps (swap_commitment, swap, position, nullifier, output_data, height_claimed, source)
                    VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6)
                    ON CONFLICT (swap_commitment)
                    DO UPDATE SET swap = excluded.swap,
                    position = excluded.position,
                    nullifier = excluded.nullifier,
                    output_data = excluded.output_data,
                    height_claimed = excluded.height_claimed,
                    source = excluded.source",
                    (
                        &swap_commitment,
                        &swap_bytes,
                        &position,
                        &nullifier,
                        &output_data,
                        // height_claimed is NULL because the swap is newly discovered
                        &source,
                    ),
                )?;
            }

            // Update any rows of the table with matching nullifiers to have height_spent
            for nullifier in &filtered_block.spent_nullifiers {
                let height_spent = filtered_block.height as i64;
                let nullifier_bytes = nullifier.to_bytes().to_vec();

                let spent_commitment: Option<StateCommitment> = dbtx.prepare_cached(
                    "UPDATE spendable_notes SET height_spent = ?1 WHERE nullifier = ?2 RETURNING note_commitment"
                )?
                    .query_and_then(
                        (height_spent, &nullifier_bytes),
                        |row| {
                            let bytes: Vec<u8> = row.get("note_commitment")?;
                            StateCommitment::try_from(&bytes[..]).context("invalid commitment bytes")
                        },
                    )?
                    .next()
                    .transpose()?;

                let swap_commitment: Option<StateCommitment> = dbtx.prepare_cached(
                    "UPDATE swaps SET height_claimed = ?1 WHERE nullifier = ?2 RETURNING swap_commitment"
                )?
                    .query_and_then(
                        (height_spent, &nullifier_bytes),
                        |row| {
                            let bytes: Vec<u8> = row.get("swap_commitment")?;
                            StateCommitment::try_from(&bytes[..]).context("invalid commitment bytes")
                        },
                    )?
                    .next()
                    .transpose()?;

                // Check denom type
                let spent_denom: String
                    = dbtx.prepare_cached(
                    "SELECT denom FROM assets
                        WHERE asset_id ==
                            (SELECT asset_id FROM notes
                             WHERE note_commitment ==
                                (SELECT note_commitment FROM spendable_notes WHERE nullifier = ?1))"
                )?
                    .query_and_then(
                        [&nullifier_bytes],
                        |row| row.get("denom"),
                    )?
                    .next()
                    .transpose()?
                    .unwrap_or("unknown".to_string());

                // Mark spent notes as spent
                if let Some(spent_commitment) = spent_commitment {
                    tracing::debug!(?nullifier, ?spent_commitment, ?spent_denom, "detected spent note commitment");
                    // Forget spent note commitments from the SCT unless they are delegation tokens,
                    // which must be saved to allow voting on proposals that might or might not be
                    // open presently

                    if DelegationToken::from_str(&spent_denom).is_err() {
                        tracing::debug!(?nullifier, ?spent_commitment, ?spent_denom, "forgetting spent note commitment");
                        new_sct.forget(spent_commitment);
                    }
                };

                // Mark spent swaps as spent
                if let Some(spent_swap_commitment) = swap_commitment {
                    tracing::debug!(?nullifier, ?spent_swap_commitment, "detected and forgetting spent swap commitment");
                    new_sct.forget(spent_swap_commitment);
                };
            }

            // Update SCT table with current SCT state
            new_sct.to_writer(&mut TreeStore(&mut dbtx))?;

            // Record all transactions
            for transaction in transactions {
                let tx_bytes = transaction.encode_to_vec();
                // We have to create an explicit temporary borrow, because the sqlx api is bad (see above)
                let tx_hash_owned = sha2::Sha256::digest(&tx_bytes);
                let tx_hash = tx_hash_owned.as_slice();
                let tx_block_height = filtered_block.height as i64;
                let decrypted_memo = transaction.decrypt_memo(&fvk).ok();
                let memo_text = decrypted_memo.clone().map_or(None,|x| Some(x.text().to_string()));
                let return_address = decrypted_memo.map_or(None, |x| Some(x.return_address().to_vec()));

                tracing::debug!(tx_hash = ?hex::encode(tx_hash), "recording extended transaction");

                dbtx.execute(
                    "INSERT OR IGNORE INTO tx (tx_hash, tx_bytes, block_height, return_address, memo_text) VALUES (?1, ?2, ?3, ?4, ?5)",
                    (&tx_hash, &tx_bytes, tx_block_height, return_address, memo_text),
                )?;

                // Associate all of the spent nullifiers with the transaction by hash.
                for nf in transaction.spent_nullifiers() {
                    let nf_bytes = nf.0.to_bytes().to_vec();
                    dbtx.execute(
                        "INSERT OR IGNORE INTO tx_by_nullifier (nullifier, tx_hash) VALUES (?1, ?2)",
                        (&nf_bytes, &tx_hash),
                    )?;
                }
            }

            // Update FMD parameters if they've changed.
            if filtered_block.fmd_parameters.is_some() {
                let fmd_parameters_bytes =
                    &fmd::Parameters::encode_to_vec(&filtered_block.fmd_parameters.ok_or_else(|| anyhow::anyhow!("missing fmd parameters in filtered block"))?)[..];

                dbtx.execute(
                    "INSERT INTO kv (k, v) VALUES ('fmd_params', ?1)
                    ON CONFLICT(k) DO UPDATE SET v = excluded.v",
                    [&fmd_parameters_bytes],
                )?;
            }

            // Update gas prices if they've changed.
            if filtered_block.gas_prices.is_some() {
                let gas_prices_bytes =
                    &GasPrices::encode_to_vec(&filtered_block.gas_prices.ok_or_else(|| anyhow::anyhow!("missing gas prices in filtered block"))?)[..];

                dbtx.execute(
                    "INSERT INTO kv (k, v) VALUES ('gas_prices', ?1)
                    ON CONFLICT(k) DO UPDATE SET v = excluded.v",
                    [&gas_prices_bytes],
                )?;
            }

            // Record block height as latest synced height
            let latest_sync_height = filtered_block.height as i64;
            dbtx.execute("UPDATE sync_height SET height = ?1", [latest_sync_height])?;

            // Commit the changes to the database
            dbtx.commit()?;

            // IMPORTANT: NO PANICS OR ERRORS PAST THIS POINT
            // If there is a panic or error past this point, the database will be left in out of
            // sync with the in-memory copy of the SCT, which means that it will become corrupted as
            // synchronization continues.

            // It's critical to reset the uncommitted height here, since we've just
            // invalidated it by committing.
            uncommitted_height.lock().take();

            // Broadcast all committed note records to channel
            // Done following tx.commit() to avoid notifying of a new SpendableNoteRecord before it is actually committed to the database

            for note_record in filtered_block.new_notes.values() {
                // This will fail to be broadcast if there is no active receiver (such as on initial
                // sync) The error is ignored, as this isn't a problem, because if there is no
                // active receiver there is nothing to do
                let _ = scanned_notes_tx.send(note_record.clone());
            }

            for nullifier in filtered_block.spent_nullifiers.iter() {
                // This will fail to be broadcast if there is no active receiver (such as on initial
                // sync) The error is ignored, as this isn't a problem, because if there is no
                // active receiver there is nothing to do
                let _ = scanned_nullifiers_tx.send(*nullifier);
            }

            for swap_record in filtered_block.new_swaps.values() {
                // This will fail to be broadcast if there is no active rece∑iver (such as on initial
                // sync) The error is ignored, as this isn't a problem, because if there is no
                // active receiver there is nothing to do
                let _ = scanned_swaps_tx.send(swap_record.clone());
            }

            anyhow::Ok(new_sct)
        })
            .await??;

        Ok(())
    }

    pub async fn owned_position_ids(
        &self,
        position_state: Option<State>,
        trading_pair: Option<TradingPair>,
    ) -> anyhow::Result<Vec<position::Id>> {
        let pool = self.pool.clone();

        let state_clause = match position_state {
            Some(state) => format!("position_state = \"{}\"", state),
            None => "".to_string(),
        };

        let pair_clause = match trading_pair {
            Some(pair) => format!("trading_pair = \"{}\"", pair),
            None => "".to_string(),
        };

        spawn_blocking(move || {
            let mut q = "SELECT position_id FROM positions".to_string();
            match (position_state.is_some(), trading_pair.is_some()) {
                (true, true) => {
                    q = q + " WHERE " + &state_clause + " AND " + &pair_clause;
                }
                (true, false) => {
                    q = q + " WHERE " + &state_clause;
                }
                (false, true) => {
                    q = q + " WHERE " + &pair_clause;
                }
                (false, false) => (),
            };

            pool.get()?
                .prepare_cached(&q)?
                .query_and_then([], |row| {
                    let position_id: Vec<u8> = row.get("position_id")?;
                    Ok(position::Id(position_id.as_slice().try_into()?))
                })?
                .collect()
        })
        .await?
    }

    pub async fn notes_by_sender(
        &self,
        return_address: &Address,
    ) -> anyhow::Result<Vec<SpendableNoteRecord>> {
        let pool = self.pool.clone();

        let query = "SELECT notes.note_commitment,
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
            JOIN tx ON spendable_notes.tx_hash = tx.tx_hash
            WHERE tx.return_address = ?1";

        let return_address = return_address.to_vec();

        let records = spawn_blocking(move || {
            pool.get()?
                .prepare(query)?
                .query_and_then([return_address], |record| record.try_into())?
                .collect::<anyhow::Result<Vec<_>>>()
        })
        .await??;

        Ok(records)
    }

    /// Get all transactions with a matching memo text. The `pattern` argument
    /// should include SQL wildcards, such as `%` and `_`, to match substrings,
    /// e.g. `%foo%`.
    pub async fn transactions_matching_memo(
        &self,
        pattern: String,
    ) -> anyhow::Result<Vec<(u64, Vec<u8>, Transaction, String)>> {
        let pattern = pattern.to_owned();
        tracing::trace!(?pattern, "searching for memos matching");
        let pool = self.pool.clone();

        spawn_blocking(move || {
            pool.get()?
                .prepare_cached("SELECT block_height, tx_hash, tx_bytes, memo_text FROM tx WHERE memo_text LIKE ?1 ESCAPE '\\'")?
                .query_and_then([pattern], |row| {
                    let block_height: u64 = row.get("block_height")?;
                    let tx_hash: Vec<u8> = row.get("tx_hash")?;
                    let tx_bytes: Vec<u8> = row.get("tx_bytes")?;
                    let tx = Transaction::decode(tx_bytes.as_slice())?;
                    let memo_text: String = row.get("memo_text")?;
                    anyhow::Ok((block_height, tx_hash, tx, memo_text))
                })?
                .collect()
        })
        .await?
    }
}
