use anyhow::Result;
use async_trait::async_trait;
use jmt::{RootHash, Version};
use penumbra_transaction::Transaction;
use tendermint::abci;

use super::{Component, Overlay, ShieldedPool};
use crate::{genesis, Storage};

/// The Penumbra application, written as a bundle of [`Component`]s.
///
/// The [`App`] is also a [`Component`], but as the top-level component,
/// it constructs the others and exposes a [`commit`](App::commit) that
/// commits the changes to the persistent storage and resets its subcomponents.
pub struct App {
    overlay: Overlay,
    // list of components goes here
    // having all the components explicitly is a bit repetitive
    // to invoke compared to Vec of dyn Component or w/e, but
    // leaves open the possibility of having component-specific
    // behavior and is simpler.
    shielded_pool: ShieldedPool,
}

impl App {
    /// Commits the application state to persistent storage,
    /// returning the new root hash and storage version.
    ///
    /// This method also resets `self` as if it were constructed
    /// as an empty overlay of the newly written state.
    pub async fn commit(&mut self, storage: Storage) -> Result<(RootHash, Version)> {
        // Commit the pending writes, clearing the overlay.
        let (root_hash, version) = self.overlay.lock().unwrap().commit(storage).await?;
        // Now re-instantiate all of the components:
        self.shielded_pool = ShieldedPool::new(self.overlay.clone());

        Ok((root_hash, version))
    }
}

#[async_trait]
impl Component for App {
    fn new(overlay: Overlay) -> Self {
        let shielded_pool = ShieldedPool::new(overlay.clone());

        Self {
            overlay,
            shielded_pool,
        }
    }

    fn init_chain(&self, app_state: &genesis::AppState) {
        self.shielded_pool.init_chain(app_state);
    }

    async fn begin_block(&self, begin_block: &abci::request::BeginBlock) {
        self.shielded_pool.begin_block(begin_block).await;
    }

    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
        ShieldedPool::check_tx_stateless(tx)?;
        Ok(())
    }

    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()> {
        self.shielded_pool.check_tx_stateful(tx).await?;
        Ok(())
    }

    async fn execute_tx(&self, tx: &Transaction) {
        self.shielded_pool.execute_tx(tx).await;
    }

    async fn end_block(&self, end_block: &abci::request::EndBlock) {
        // TODO: should these calls be in reverse order from begin_block?
        self.shielded_pool.end_block(end_block).await;
    }
}
