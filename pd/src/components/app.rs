use anyhow::Result;
use async_trait::async_trait;
use jmt::{RootHash, Version};
use penumbra_chain::{genesis, View as _};
use penumbra_component::Component;
use penumbra_ibc::IBCComponent;
use penumbra_shielded_pool::ShieldedPool;
use penumbra_stake::component::Staking;
use penumbra_storage::{State, StateExt, Storage};
use penumbra_transaction::Transaction;
use tendermint::abci::{self, types::ValidatorUpdate};

use tracing::instrument;

/// The Penumbra application, written as a bundle of [`Component`]s.
///
/// The [`App`] is also a [`Component`], but as the top-level component,
/// it constructs the others and exposes a [`commit`](App::commit) that
/// commits the changes to the persistent storage and resets its subcomponents.
pub struct App {
    state: State,
    shielded_pool: ShieldedPool,
    ibc: IBCComponent,
    staking: Staking,
}

impl App {
    #[instrument(skip(storage))]
    pub async fn new(storage: Storage) -> Self {
        let staking = Staking::new(storage.state().await.unwrap()).await;
        let ibc = IBCComponent::new(storage.state().await.unwrap()).await;
        let shielded_pool = ShieldedPool::new(storage.clone()).await;

        Self {
            state: storage.state().await.unwrap(),
            shielded_pool,
            staking,
            ibc,
        }
    }

    /// Commits the application state to persistent storage,
    /// returning the new root hash and storage version.
    ///
    /// This method also resets `self` as if it were constructed
    /// as an empty state over top of the newly written storage.
    #[instrument(skip(self, storage))]
    pub async fn commit(&mut self, storage: Storage) -> Result<(RootHash, Version)> {
        // Commit the pending writes, clearing the state.
        let (root_hash, version) = self.state.write().await.commit(storage.clone()).await?;
        tracing::debug!(?root_hash, version, "finished committing state");
        // Now re-instantiate all of the components:
        self.staking = Staking::new(self.state.clone()).await;
        self.ibc = IBCComponent::new(self.state.clone()).await;
        self.shielded_pool = ShieldedPool::new(storage).await;

        Ok((root_hash, version))
    }

    // TODO: should this just be returned by `commit`? both are called during every `EndBlock`
    pub async fn tm_validator_updates(&self) -> Result<Vec<ValidatorUpdate>> {
        self.staking.tm_validator_updates().await
    }
}

#[async_trait]
impl Component for App {
    #[instrument(skip(self, app_state))]
    async fn init_chain(&mut self, app_state: &genesis::AppState) {
        self.state
            .put_chain_params(app_state.chain_params.clone())
            .await;
        // TODO: do we actually need to store the app state here?
        self.state
            .put_domain(b"genesis/app_state".into(), app_state.clone())
            .await;
        // The genesis block height is 0
        self.state.put_block_height(0).await;

        self.staking.init_chain(app_state).await;
        self.ibc.init_chain(app_state).await;

        // Shielded pool always executes last.
        self.shielded_pool.init_chain(app_state).await;
    }

    #[instrument(skip(self, begin_block))]
    async fn begin_block(&mut self, begin_block: &abci::request::BeginBlock) {
        // store the block height
        self.state
            .put_block_height(begin_block.header.height.into())
            .await;
        // store the block time
        self.state
            .put_block_timestamp(begin_block.header.time)
            .await;

        self.staking.begin_block(begin_block).await;
        self.ibc.begin_block(begin_block).await;
        // Shielded pool always executes last.
        self.shielded_pool.begin_block(begin_block).await;
    }

    #[instrument(skip(tx))]
    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
        Staking::check_tx_stateless(tx)?;
        IBCComponent::check_tx_stateless(tx)?;
        ShieldedPool::check_tx_stateless(tx)?;
        Ok(())
    }

    #[instrument(skip(self, tx))]
    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()> {
        self.staking.check_tx_stateful(tx).await?;
        self.ibc.check_tx_stateful(tx).await?;

        // Shielded pool always executes last.
        self.shielded_pool.check_tx_stateful(tx).await?;
        Ok(())
    }

    #[instrument(skip(self, tx))]
    async fn execute_tx(&mut self, tx: &Transaction) {
        self.staking.execute_tx(tx).await;
        self.ibc.execute_tx(tx).await;
        // Shielded pool always executes last.
        self.shielded_pool.execute_tx(tx).await;
    }

    #[instrument(skip(self, end_block))]
    async fn end_block(&mut self, end_block: &abci::request::EndBlock) {
        self.staking.end_block(end_block).await;
        self.ibc.end_block(end_block).await;

        // Shielded pool always executes last.
        self.shielded_pool.end_block(end_block).await;
    }
}
