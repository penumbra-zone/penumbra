use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use penumbra_wallet::ClientState;

pub struct ClientStateFile {
    path: PathBuf,
    state: ClientState,
    lock: fslock::LockFile,
}

impl Deref for ClientStateFile {
    type Target = ClientState;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl DerefMut for ClientStateFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl Drop for ClientStateFile {
    fn drop(&mut self) {
        self.lock.unlock().unwrap();
    }
}

impl ClientStateFile {
    /// Create a new wrapper by saving to the provided `path`.
    ///
    /// If you already have a wrapper, use [`Self::commit`].
    pub fn save(state: ClientState, path: PathBuf) -> Result<Self> {
        let lock = lock_wallet(&path)?;

        let wrapper = Self { state, path, lock };
        wrapper.commit()?;
        Ok(wrapper)
    }

    /// Create a new wrapper by loading from the provided `path`.
    pub fn load(path: PathBuf) -> Result<Self> {
        let lock = lock_wallet(&path)?;

        let mut state: ClientState = match std::fs::read(&path) {
            Ok(data) => serde_json::from_slice(&data).context("Could not parse wallet data")?,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => return Err(err).context(
                    "Wallet data not found, run `pcli wallet generate` to generate Penumbra keys",
                ),
                _ => return Err(err.into()),
            },
        };

        // Pruning timeouts on load means every freshly loaded wallet will be up to date on timeouts
        // as of when it is taken off disk
        state.prune_timeouts();

        Ok(Self { state, path, lock })
    }

    /// Commit the client state to disk.
    pub fn commit(&self) -> Result<()> {
        tracing::debug!("committing state");

        let tmp_path = self.path.with_extension("tmp");

        // Write the state to the temp file
        let mut tmp_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)?;
        serde_json::to_writer_pretty(&mut tmp_file, &self.state)?;

        // Overwrite the existing wallet state file, *atomically*
        std::fs::rename(&tmp_path, &self.path)?;

        Ok(())
    }
}

fn lock_wallet(path: &Path) -> Result<fslock::LockFile> {
    let mut lock = fslock::LockFile::open(&path.with_extension("lock"))?;

    // Try to lock the file and note in the log if we are waiting for another process to finish
    tracing::debug!(?path, "Locking wallet file");
    if !lock.try_lock()? {
        tracing::info!(?path, "Waiting to acquire lock for wallet");
        lock.lock()?;
    }

    Ok(lock)
}
