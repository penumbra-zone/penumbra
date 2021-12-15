use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::{Context, Result};
use penumbra_wallet::ClientState;

pub struct ClientStateFile {
    path: PathBuf,
    state: ClientState,
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
    }
}

impl ClientStateFile {
    /// Create a new wrapper by saving to the provided `path`.
    ///
    /// If you already have a wrapper, use [`Self::commit`].
    pub fn save(state: ClientState, path: PathBuf) -> Result<Self> {
        let wrapper = Self { state, path };
        wrapper.commit()?;
        Ok(wrapper)
    }

    /// Create a new wrapper by loading from the provided `path`.
    pub fn load(path: PathBuf) -> Result<Self> {
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

        Ok(Self { state, path })
    }

    /// Commit the client state to disk.
    pub fn commit(&self) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;

        serde_json::to_writer_pretty(&mut file, &self.state)?;

        Ok(())
    }
}
