use anyhow::Result;
use async_trait::async_trait;
use jmt::{RootHash, Version};
use penumbra_chain::{genesis, View as _};
use penumbra_component::shielded_pool::ShieldedPool;
use penumbra_component::stake::component::Staking;
use penumbra_component::{Component, Context};
use penumbra_component::ibc::IBCComponent;
use penumbra_storage::{State, StateExt, Storage};
use penumbra_transaction::Transaction;
use tendermint::abci::{self, types::ValidatorUpdate};

use tracing::instrument;

use super::state_key;

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
    pub async fn new(storage: Storage) -> Self {
        tracing::info!("initializing App instance");

        // The NCT (and *only* the NCT) is stored outside of the main state,
        // so that the backing format for the NCT isn't consensus-critical.
        // (The NCT data is already committed to by the NCT root, which is in the state).
        let nct = storage.get_nct().await.unwrap();

        // All of the components need to use the *same* shared state.
        let state = storage.state().await.unwrap();

        let staking = Staking::new(state.clone()).await;
        let ibc = IBCComponent::new(state.clone()).await;
        let shielded_pool = ShieldedPool::new(state.clone(), nct).await;

        Self {
            state,
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
        // We want to store the latest NCT in a sidecar part of the storage,
        // rather than the Penumbra state, because the serialization format for
        // the NCT should not be consensus-critical.  We need to grab a copy of
        // the entire NCT, so we can use it to re-instantiate the ShieldedPool.
        let nct = self.shielded_pool.note_commitment_tree();
        storage.put_nct(nct).await?;
        // Commit the pending writes, clearing the state.
        let (root_hash, version) = self.state.write().await.commit(storage.clone()).await?;
        tracing::debug!(?root_hash, version, "finished committing state");

        // Get the latest version of the state, now that we've committed it.
        self.state = storage.state().await?;

        // Now re-instantiate all of the components so they all have the same shared state.
        self.staking = Staking::new(self.state.clone()).await;
        self.ibc = IBCComponent::new(self.state.clone()).await;
        self.shielded_pool = ShieldedPool::new(self.state.clone(), nct.clone()).await;

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
            .put_domain(state_key::app_state(), app_state.clone())
            .await;
        // The genesis block height is 0
        self.state.put_block_height(0).await;

        self.staking.init_chain(app_state).await;
        self.ibc.init_chain(app_state).await;

        // Shielded pool always executes last.
        self.shielded_pool.init_chain(app_state).await;
    }

    #[instrument(skip(self, ctx, begin_block))]
    async fn begin_block(&mut self, ctx: Context, begin_block: &abci::request::BeginBlock) {
        // store the block height
        self.state
            .put_block_height(begin_block.header.height.into())
            .await;
        // store the block time
        self.state
            .put_block_timestamp(begin_block.header.time)
            .await;

        self.staking.begin_block(ctx.clone(), begin_block).await;
        self.ibc.begin_block(ctx.clone(), begin_block).await;
        // Shielded pool always executes last.
        self.shielded_pool
            .begin_block(ctx.clone(), begin_block)
            .await;
    }

    #[instrument(skip(ctx, tx))]
    fn check_tx_stateless(ctx: Context, tx: &Transaction) -> Result<()> {
        Staking::check_tx_stateless(ctx.clone(), tx)?;
        IBCComponent::check_tx_stateless(ctx.clone(), tx)?;
        ShieldedPool::check_tx_stateless(ctx.clone(), tx)?;
        Ok(())
    }

    #[instrument(skip(self, ctx, tx))]
    async fn check_tx_stateful(&self, ctx: Context, tx: &Transaction) -> Result<()> {
        self.staking.check_tx_stateful(ctx.clone(), tx).await?;
        self.ibc.check_tx_stateful(ctx.clone(), tx).await?;

        // Shielded pool always executes last.
        self.shielded_pool
            .check_tx_stateful(ctx.clone(), tx)
            .await?;
        Ok(())
    }

    #[instrument(skip(self, ctx, tx))]
    async fn execute_tx(&mut self, ctx: Context, tx: &Transaction) {
        self.staking.execute_tx(ctx.clone(), tx).await;
        self.ibc.execute_tx(ctx.clone(), tx).await;
        // Shielded pool always executes last.
        self.shielded_pool.execute_tx(ctx.clone(), tx).await;
    }

    #[instrument(skip(self, ctx, end_block))]
    async fn end_block(&mut self, ctx: Context, end_block: &abci::request::EndBlock) {
        self.staking.end_block(ctx.clone(), end_block).await;
        self.ibc.end_block(ctx.clone(), end_block).await;

        // Shielded pool always executes last.
        self.shielded_pool.end_block(ctx.clone(), end_block).await;
    }
}
