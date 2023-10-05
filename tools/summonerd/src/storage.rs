use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use anyhow::{Context, Result};
use camino::Utf8Path;
use penumbra_keys::Address;
use penumbra_proof_setup::all::{Phase2CeremonyCRS, Phase2CeremonyContribution};
use r2d2_sqlite::{rusqlite::OpenFlags, SqliteConnectionManager};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;
use url::Url;

#[derive(Clone)]
pub struct Storage {
    pool: r2d2::Pool<SqliteConnectionManager>,

    crs: Arc<Mutex<Phase2CeremonyCRS>>,
    slot: Arc<AtomicU64>,
    root: Phase2CeremonyCRS,
}

impl Storage {
    /// If the database at `storage_path` exists, [`Self::load`] it, otherwise, [`Self::initialize`] it.
    pub async fn load_or_initialize(
        storage_path: Option<impl AsRef<Utf8Path>>,
        node: Url,
    ) -> anyhow::Result<Self> {
        if let Some(path) = storage_path.as_ref() {
            if path.as_ref().exists() {
                return Self::load(path).await;
            }
        };

        Self::initialize(storage_path).await
    }

    pub async fn initialize(storage_path: Option<impl AsRef<Utf8Path>>) -> anyhow::Result<Self> {
        tracing::debug!(storage_path = ?storage_path.as_ref().map(AsRef::as_ref));

        // Connect to the database (or create it)
        let pool = Self::connect(storage_path)?;

        spawn_blocking(move || {
            // In one database transaction, populate everything
            let mut conn = pool.get()?;
            let tx = conn.transaction()?;

            // Create the tables
            tx.execute_batch(include_str!("storage/schema.sql"))?;

            tx.commit()?;
            drop(conn);

            let root = Phase2CeremonyCRS::root()?;
            Ok(Storage {
                pool,
                crs: Arc::new(Mutex::new(root.clone())),
                slot: Arc::new(AtomicU64::new(0)),
                root,
            })
        })
        .await?
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
                    // We use `prepare_cached` a fair amount: this is an overestimate of the number
                    // of cached prepared statements likely to be used.
                    conn.set_prepared_statement_cache_capacity(32);
                    Ok(())
                });
            Ok(r2d2::Pool::new(manager)?)
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
        let root: Phase2CeremonyCRS = Phase2CeremonyCRS::root()?;
        let storage = Self {
            pool: Self::connect(Some(path))?,
            crs: Arc::new(Mutex::new(root.clone())),
            slot: Arc::new(AtomicU64::new(0)),
            root,
        };

        Ok(storage)
    }

    pub fn new(root: Phase2CeremonyCRS) -> Self {
        // TODO: This will be removed in favor of load_and_initialize
        Self {
            pool: Self::connect(None).unwrap(),
            crs: Arc::new(Mutex::new(root.clone())),
            slot: Arc::new(AtomicU64::new(0)),
            root,
        }
    }

    pub async fn root(&self) -> Result<Phase2CeremonyCRS> {
        Ok(self.root.clone())
    }

    pub async fn can_contribute(&self, _address: Address) -> Result<()> {
        // Criteria:
        // - Not banned
        // - Bid more than min amount
        // - Hasn't already contributed
        Ok(())
    }

    pub async fn current_crs(&self) -> Result<Phase2CeremonyCRS> {
        Ok(self.crs.lock().await.clone())
    }

    // TODO: Add other stuff here
    pub async fn commit_contribution(
        &self,
        _contributor: Address,
        contribution: &Phase2CeremonyContribution,
    ) -> Result<()> {
        *self.crs.lock().await = contribution.new_elements();
        self.slot.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub async fn current_slot(&self) -> Result<u64> {
        Ok(self.slot.load(Ordering::SeqCst))
    }
}
