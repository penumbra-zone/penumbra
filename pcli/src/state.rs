use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use penumbra_wallet::{ClientState, SyncCheckpoint};

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
        self.commit().unwrap();
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
        // It is safe to commit without again acquiring the lock, because we cannot construct `Self`
        // without first acquiring the lock via `lock_wallet`.
        commit_regardless_of_lock(&self.state, &self.path)
    }

    /// Synchronize the client state with the current state of the chain using the light wallet protocol.
    pub async fn sync(&mut self, wallet_uri: String) -> Result<()> {
        let checkpoint = SyncEvery {
            interval: 1000,
            path: &self.path,
        };

        self.state.sync(wallet_uri, checkpoint).await?;

        Ok(())
    }
}

/// A sync-checkpointing strategy that atomically commits the wallet to disk at `path` every time
/// the block height is a multiple of `interval`.
struct SyncEvery<'path> {
    /// Path to commit the wallet to.
    path: &'path Path,
    /// Sync to disk every `interval` blocks.
    interval: u32,
}

impl<'a, 'path> SyncCheckpoint<'a> for SyncEvery<'path> {
    type Future = std::future::Ready<Result<()>>;

    fn per_block(
        &mut self,
        state: &'a ClientState,
        block_height: u32,
        is_final_block: bool,
    ) -> Self::Future {
        if is_final_block || block_height % self.interval == 0 {
            // It is safe to call this method because we know we have the lock, since we
            // can't construct `Self` without first acquiring the lock via `lock_wallet`.
            tracing::info!(height = ?block_height, "syncing...");
            std::future::ready(commit_regardless_of_lock(state, self.path))
        } else {
            std::future::ready(Ok(()))
        }
    }
}

/// Acquire a lock on the wallet file.
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

/// Atomically commit a given client state to a given path, without checking the file lock.
///
/// This should only be used if we are sure that we own the lock on the file, to avoid race
/// conditions on the filesystem. This is ensured by making sure that we acquire the lock when
/// constructing a [`ClientStateFile`].
fn commit_regardless_of_lock(state: &ClientState, path: &Path) -> anyhow::Result<()> {
    // Open a new named temp file (this has to be a named temp file because we need to persist
    // it and there's no platform-independent way to do this using an anonymous temp file)
    let tmp = tempfile::NamedTempFile::new()?;

    // Write the state to the temp file
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(tmp.path())?;
    serde_json::to_writer_pretty(&mut file, state)?;

    // Overwrite the existing wallet state file, *atomically*
    tmp.persist(path)?;

    Ok(())
}
