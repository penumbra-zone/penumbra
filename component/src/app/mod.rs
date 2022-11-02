use std::sync::Arc;

use crate::dex::Dex;
use crate::governance::Governance;
use crate::ibc::IBCComponent;
use crate::shielded_pool::ShieldedPool;
use crate::stake::component::Staking;
use crate::{Component, Context};
use anyhow::Result;
use penumbra_chain::params::FmdParameters;
use penumbra_chain::{genesis, StateReadExt as _};
use penumbra_storage2::{AppHash, State, StateRead, StateTransaction, StateWrite, Storage};
use penumbra_transaction::Transaction;
use tendermint::abci::{self, types::ValidatorUpdate};

use tracing::instrument;

pub mod state_key;
/// The Penumbra application, written as a bundle of [`Component`]s.
///
/// The [`App`] is not a [`Component`], but
/// it constructs the components and exposes a [`commit`](App::commit) that
/// commits the changes to the persistent storage and resets its subcomponents.
pub struct App {
    state: Arc<State>,
}

impl App {
    pub async fn new(state: State) -> Self {
        tracing::info!("initializing App instance");
        Self {
            // We perform the `Arc` wrapping of `State` here to ensure
            // there should be no unexpected copies elsewhere.
            state: Arc::new(state),
        }
    }

    #[instrument(skip(self, app_state))]
    async fn init_chain(&mut self, app_state: &genesis::AppState) {
        let mut state = self
            .state
            .get_mut()
            .expect("state Arc should not be referenced elsewhere");
        let state_tx = state.begin_transaction();

        state_tx.put_chain_params(app_state.chain_params.clone());

        // TEMP: Hardcoding FMD parameters until we have a mechanism to change them. See issue #1226.
        state_tx.put_current_fmd_parameters(FmdParameters::default());
        state_tx.put_previous_fmd_parameters(FmdParameters::default());

        // TODO: do we actually need to store the app state here?
        state_tx.put(state_key::app_state().into(), app_state.clone());

        // The genesis block height is 0
        state_tx.put_block_height(0);

        Staking::init_chain(state, app_state).await.unwrap();
        IBCComponent::init_chain(state, app_state).await.unwrap();
        Dex::init_chain(state, app_state).await.unwrap();
        Governance::init_chain(state, app_state).await.unwrap();
        // Shielded pool always executes last.
        ShieldedPool::init_chain(state, app_state).await.unwrap();

        state_tx.apply();
    }

    #[instrument(skip(self, ctx, begin_block))]
    async fn begin_block(&mut self, ctx: Context, begin_block: &abci::request::BeginBlock) {
        let mut state = self
            .state
            .get_mut()
            .expect("state Arc should not be referenced elsewhere");
        let state_tx = state.begin_transaction();

        // store the block height
        state_tx.put_block_height(begin_block.header.height.into());
        // store the block time
        state_tx.put_block_timestamp(begin_block.header.time);

        Staking::begin_block(state, ctx.clone(), begin_block)
            .await
            .unwrap();
        IBCComponent::begin_block(state, ctx.clone(), begin_block)
            .await
            .unwrap();
        Dex::begin_block(state, ctx.clone(), begin_block)
            .await
            .unwrap();
        Governance::begin_block(state, ctx.clone(), begin_block)
            .await
            .unwrap();
        // Shielded pool always executes last.
        ShieldedPool::begin_block(state, ctx.clone(), begin_block)
            .await
            .unwrap();

        state_tx.apply();
    }

    #[instrument(skip(self, ctx, tx))]
    async fn deliver_tx(&mut self, ctx: Context, tx: Arc<Transaction>) -> Result<()> {
        Self::check_tx_stateless(ctx.clone(), tx).await?;
        Self::check_tx_stateful(self.state.clone(), ctx, tx).await?;

        // We need to get a mutable reference to the State here, so we use
        // `Arc::get_mut`. At this point, the stateful checks should have completed,
        // leaving us with exclusive access to the Arc<State>.
        let mut state = self
            .state
            .get_mut()
            .expect("state Arc should not be referenced elsewhere");
        let state_tx = state.begin_transaction();

        Self::execute_tx(state_tx, ctx, tx).await?;

        // At this point, we've completed execution successfully with no errors,
        // so we can apply the transaction to the State. Otherwise, we'd have
        // bubbled up an error and dropped the StateTransaction.
        state_tx.apply();

        Ok(())
    }

    #[instrument(skip(self, ctx, end_block))]
    async fn end_block(&mut self, ctx: Context, end_block: &abci::request::EndBlock) {
        let mut state = self
            .state
            .get_mut()
            .expect("state Arc should not be referenced elsewhere");
        let state_tx = state.begin_transaction();

        Staking::end_block(state, ctx.clone(), end_block)
            .await
            .unwrap();
        IBCComponent::end_block(state, ctx.clone(), end_block)
            .await
            .unwrap();
        Dex::end_block(state, ctx.clone(), end_block).await.unwrap();
        Governance::end_block(state, ctx.clone(), end_block)
            .await
            .unwrap();
        // Shielded pool always executes last.
        ShieldedPool::end_block(state, ctx.clone(), end_block)
            .await
            .unwrap();

        state_tx.apply();
    }

    /// Commits the application state to persistent storage,
    /// returning the new root hash and storage version.
    ///
    /// This method also resets `self` as if it were constructed
    /// as an empty state over top of the newly written storage.
    #[instrument(skip(self, storage))]
    pub async fn commit(&mut self, storage: Storage) -> Result<AppHash> {
        // We need to extract the State we've built up to commit it.  Fill in a dummy state.
        let dummy_state = storage.state();
        let state = Arc::try_unwrap(std::mem::replace(&mut self.state, Arc::new(dummy_state)))
            .expect("we have exclusive ownership of the State at commit()");

        // Commit the pending writes, clearing the state.
        let jmt_root = storage.commit(state)?;
        let app_hash: AppHash = jmt_root.into();

        tracing::debug!(?app_hash, "finished committing state");

        // Get the latest version of the state, now that we've committed it.
        self.state = Arc::new(storage.state());

        Ok(app_hash)
    }

    // TODO: should this just be returned by `commit`? both are called during every `EndBlock`
    pub fn tendermint_validator_updates(&self) -> Vec<ValidatorUpdate> {
        // TODO: replace with self.state read ?
        self.staking.tendermint_validator_updates()
    }

    #[instrument(skip(ctx, tx))]
    fn check_tx_stateless(ctx: Context, tx: Arc<Transaction>) -> Result<()> {
        // TODO: these can all be parallel tasks

        Staking::check_tx_stateless(ctx.clone(), tx)?;
        IBCComponent::check_tx_stateless(ctx.clone(), tx)?;
        Dex::check_tx_stateless(ctx.clone(), tx)?;
        Governance::check_tx_stateless(ctx.clone(), tx)?;
        ShieldedPool::check_tx_stateless(ctx, tx)?;

        Ok(())
    }

    #[instrument(skip(state, ctx, tx))]
    async fn check_tx_stateful(
        state: Arc<State>,
        ctx: Context,
        tx: Arc<Transaction>,
    ) -> Result<()> {
        // TODO: these can all be parallel tasks

        Staking::check_tx_stateful(state.clone(), ctx.clone(), tx.clone()).await?;
        IBCComponent::check_tx_stateful(state.clone(), ctx.clone(), tx.clone()).await?;
        Dex::check_tx_stateful(state.clone(), ctx.clone(), tx.clone()).await?;
        Governance::check_tx_stateful(state.clone(), ctx.clone(), tx.clone()).await?;
        ShieldedPool::check_tx_stateful(state.clone(), ctx.clone(), tx.clone()).await?;

        Ok(())
    }

    #[instrument(skip(state, ctx, tx))]
    async fn execute_tx(
        state: &mut StateTransaction<'_>,
        ctx: Context,
        tx: Arc<Transaction>,
    ) -> Result<()> {
        Staking::execute_tx(state, ctx.clone(), tx.clone()).await?;
        IBCComponent::execute_tx(state, ctx.clone(), tx.clone()).await?;
        Dex::execute_tx(state, ctx.clone(), tx.clone()).await?;
        Governance::execute_tx(state, ctx.clone(), tx.clone()).await?;
        // Shielded pool always executes last.
        ShieldedPool::execute_tx(state, ctx.clone(), tx.clone()).await?;

        Ok(())
    }
}
