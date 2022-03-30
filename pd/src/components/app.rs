use anyhow::Result;
use async_trait::async_trait;
use jmt::{RootHash, Version};
use penumbra_transaction::Transaction;
use tendermint::abci;

use super::{Component, IBCComponent, Overlay, ShieldedPool};
use crate::{genesis, PenumbraStore, Storage, WriteOverlayExt};

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
    ibc: IBCComponent,
}

impl App {
    /// Commits the application state to persistent storage,
    /// returning the new root hash and storage version.
    ///
    /// This method also resets `self` as if it were constructed
    /// as an empty overlay of the newly written state.
    pub async fn commit(&mut self, storage: Storage) -> Result<(RootHash, Version)> {
        // Commit the pending writes, clearing the overlay.
        let (root_hash, version) = self.overlay.lock().await.commit(storage).await?;
        // Now re-instantiate all of the components:
        self.shielded_pool = ShieldedPool::new(self.overlay.clone()).await?;
        self.ibc = IBCComponent::new(self.overlay.clone()).await?;

        Ok((root_hash, version))
    }
}

#[async_trait]
impl Component for App {
    async fn new(overlay: Overlay) -> Result<Self> {
        let shielded_pool = ShieldedPool::new(overlay.clone()).await?;
        let ibc = IBCComponent::new(overlay.clone()).await?;

        Ok(Self {
            overlay,
            shielded_pool,
            ibc,
        })
    }

    async fn init_chain(&mut self, app_state: &genesis::AppState) -> Result<()> {
        self.overlay
            .put_chain_params(app_state.chain_params.clone())
            .await;
        // TODO: do we actually need to store the app state here?
        self.overlay
            .put_domain(b"genesis/app_state".into(), app_state.clone())
            .await;

        self.shielded_pool.init_chain(app_state).await?;
        self.ibc.init_chain(app_state).await?;
        Ok(())
    }

    async fn begin_block(&mut self, begin_block: &abci::request::BeginBlock) -> Result<()> {
        self.shielded_pool.begin_block(begin_block).await?;
        self.ibc.begin_block(begin_block).await?;
        Ok(())
    }

    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
        ShieldedPool::check_tx_stateless(tx)?;
        IBCComponent::check_tx_stateless(tx)?;
        Ok(())
    }

    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()> {
        self.shielded_pool.check_tx_stateful(tx).await?;
        self.ibc.check_tx_stateful(tx).await?;
        Ok(())
    }

    async fn execute_tx(&mut self, tx: &Transaction) -> Result<()> {
        self.shielded_pool.execute_tx(tx).await?;
        self.ibc.execute_tx(tx).await?;
        Ok(())
    }

    async fn end_block(&mut self, end_block: &abci::request::EndBlock) -> Result<()> {
        // TODO: should these calls be in reverse order from begin_block?
        self.shielded_pool.end_block(end_block).await?;
        self.ibc.end_block(end_block).await?;
        Ok(())
    }
}
