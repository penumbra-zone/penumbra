use crate::Storage;
use anyhow;
use std::ops::Deref;
use tempfile::TempDir;

/// A [`Storage`] instance backed by a [`tempfile::TempDir`] for testing.
///
/// The `TempDir` handle is bundled into the `TempStorage`, so the temporary
/// directory is cleaned up when the `TempStorage` instance is dropped.
pub struct TempStorage {
    inner: Storage,
    _dir: TempDir,
}

impl Deref for TempStorage {
    type Target = Storage;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TempStorage {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let dir = tempfile::tempdir()?;
        let db_filepath = dir.path().join("storage.db");
        let inner = Storage::load(db_filepath.clone()).await?;

        Ok(TempStorage { inner, _dir: dir })
    }
}
