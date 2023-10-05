use std::sync::Arc;

use anyhow::Result;
use camino::Utf8Path;
use penumbra_keys::Address;
use penumbra_proof_setup::all::{Phase2CeremonyCRS, Phase2CeremonyContribution};
use r2d2_sqlite::{rusqlite::OpenFlags, SqliteConnectionManager};
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;

#[derive(Clone)]
pub struct Storage {
    pool: r2d2::Pool<SqliteConnectionManager>,
    crs: Arc<Mutex<Phase2CeremonyCRS>>,
}

impl Storage {
    /// If the database at `storage_path` exists, [`Self::load`] it, otherwise, [`Self::initialize`] it.
    pub async fn load_or_initialize(storage_path: impl AsRef<Utf8Path>) -> anyhow::Result<Self> {
        if storage_path.as_ref().exists() {
            return Self::load(storage_path).await;
        }

        Self::initialize(storage_path).await
    }

    pub async fn initialize(storage_path: impl AsRef<Utf8Path>) -> anyhow::Result<Self> {
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
            })
        })
        .await?
    }

    fn connect(path: impl AsRef<Utf8Path>) -> anyhow::Result<r2d2::Pool<SqliteConnectionManager>> {
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
    }

    pub async fn load(path: impl AsRef<Utf8Path>) -> anyhow::Result<Self> {
        let root: Phase2CeremonyCRS = Phase2CeremonyCRS::root()?;
        let storage = Self {
            pool: Self::connect(path)?,
            crs: Arc::new(Mutex::new(root.clone())),
        };

        Ok(storage)
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
        contributor: Address,
        contribution: &Phase2CeremonyContribution,
    ) -> Result<()> {
        *self.crs.lock().await = contribution.new_elements();
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let contributor_bytes = contributor.to_vec();
        tx.execute(
            "INSERT INTO phase2_contributions VALUES(NULL, ?1)",
            [contributor_bytes],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub async fn current_slot(&self) -> Result<u64> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let out = tx
            .query_row("SELECT MAX(slot) FROM phase2_contributions", [], |row| {
                row.get::<usize, Option<u64>>(0)
            })?
            .unwrap_or(0);
        Ok(out)
    }
}
