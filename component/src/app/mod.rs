use std::sync::Arc;

use crate::dex::Dex;
use crate::governance::Governance;
use crate::ibc::IBCComponent;
use crate::shielded_pool::ShieldedPool;
use crate::stake::component::Staking;
use crate::{Component, Context};
use anyhow::Result;
use async_trait::async_trait;
use jmt::Version;
use penumbra_chain::params::FmdParameters;
use penumbra_chain::{genesis, View as _};
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
    shielded_pool: ShieldedPool,
    ibc: IBCComponent,
    staking: Staking,
    dex: Dex,
    governance: Governance,
}

impl App {
    pub async fn new(state: State) -> Self {
        tracing::info!("initializing App instance");

        let staking = Staking::new().await;
        let ibc = IBCComponent::new().await;
        let dex = Dex::new().await;
        let governance = Governance::new().await;
        let shielded_pool = ShieldedPool::new().await;

        Self {
            // We perform the `Arc` wrapping of `State` here to ensure
            // there should be no unexpected copies elsewhere.
            state: Arc::new(state),
            shielded_pool,
            staking,
            ibc,
            dex,
            governance,
        }
    }

    #[instrument(skip(self, app_state))]
    async fn init_chain(&mut self, app_state: &genesis::AppState) {
        self.state
            .put_chain_params(app_state.chain_params.clone())
            .await;

        // TEMP: Hardcoding FMD parameters until we have a mechanism to change them. See issue #1226.
        self.state
            .put_current_fmd_parameters(FmdParameters::default())
            .await;
        self.state
            .put_previous_fmd_parameters(FmdParameters::default())
            .await;

        // TODO: do we actually need to store the app state here?
        self.state
            .put_domain(state_key::app_state().into(), app_state.clone())
            .await;
        // The genesis block height is 0
        self.state.put_block_height(0).await;

        self.staking.init_chain(app_state).await;
        self.ibc.init_chain(app_state).await;
        self.dex.init_chain(app_state).await;
        self.governance.init_chain(app_state).await;

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
        self.dex.begin_block(ctx.clone(), begin_block).await;
        self.governance.begin_block(ctx.clone(), begin_block).await;
        // Shielded pool always executes last.
        self.shielded_pool
            .begin_block(ctx.clone(), begin_block)
            .await;
    }

    #[instrument(skip(self, ctx, tx))]
    async fn deliver_tx(&mut self, ctx: Context, tx: &Transaction) -> Result<()> {
        // stateless
        Self::check_tx_stateless(ctx.clone(), tx).await?;

        // stateful
        self.check_tx_stateful(ctx, tx).await?;

        // We need to get a mutable reference to the State here, so we use
        // `Arc::get_mut`. This should be infallible because we are the only owner of the
        // Arc.
        let mut state = self
            .state
            .get_mut()
            .expect("state Arc should not be referenced elsewhere");
        let state_tx = state.begin_transaction();

        // execute
        self.execute_tx(ctx, tx, state_tx).await?;

        // commit
        state_tx.apply();

        Ok(())
    }

    #[instrument(skip(self, ctx, end_block))]
    async fn end_block(&mut self, ctx: Context, end_block: &abci::request::EndBlock) {
        let state_tx = self.state.begin_transaction();

        self.staking
            .end_block(ctx.clone(), end_block, state_tx)
            .await;
        self.ibc.end_block(ctx.clone(), end_block, state_tx).await;
        self.dex.end_block(ctx.clone(), end_block, state_tx).await;
        self.governance
            .end_block(ctx.clone(), end_block, state_tx)
            .await;

        // Shielded pool always executes last.
        self.shielded_pool
            .end_block(ctx.clone(), end_block, state_tx)
            .await;

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

        tracing::debug!(?app_hash, version, "finished committing state");

        // Get the latest version of the state, now that we've committed it.
        self.state = Arc::new(storage.state());

        Ok((app_hash, version))
    }

    // TODO: should this just be returned by `commit`? both are called during every `EndBlock`
    pub fn tendermint_validator_updates(&self) -> Vec<ValidatorUpdate> {
        self.staking.tendermint_validator_updates()
    }

    #[instrument(skip(ctx, tx))]
    fn check_tx_stateless(ctx: Context, tx: &Transaction) -> Result<()> {
        Staking::check_tx_stateless(ctx.clone(), tx)?;
        IBCComponent::check_tx_stateless(ctx.clone(), tx)?;
        Dex::check_tx_stateless(ctx.clone(), tx)?;
        Governance::check_tx_stateless(ctx.clone(), tx)?;
        ShieldedPool::check_tx_stateless(ctx, tx)?;
        Ok(())
    }

    #[instrument(skip(self, ctx, tx))]
    async fn check_tx_stateful(&self, ctx: Context, tx: &Transaction) -> Result<()> {
        self.staking
            .check_tx_stateful(ctx.clone(), tx, self.state.clone())
            .await?;
        self.ibc
            .check_tx_stateful(ctx.clone(), tx, self.state.clone())
            .await?;
        self.dex
            .check_tx_stateful(ctx.clone(), tx, self.state.clone())
            .await?;
        self.governance
            .check_tx_stateful(ctx.clone(), tx, self.state.clone())
            .await?;

        // Shielded pool always executes last.
        self.shielded_pool
            .check_tx_stateful(ctx.clone(), tx, self.state.clone())
            .await?;
        Ok(())
    }

    #[instrument(skip(self, ctx, tx, state_tx))]
    async fn execute_tx(
        &mut self,
        ctx: Context,
        tx: &Transaction,
        state_tx: &mut StateTransaction<'_>,
    ) -> Result<()> {
        self.staking.execute_tx(ctx.clone(), tx, state_tx).await?;
        self.ibc.execute_tx(ctx.clone(), tx, state_tx).await?;
        self.dex.execute_tx(ctx.clone(), tx, state_tx).await?;
        self.governance
            .execute_tx(ctx.clone(), tx, state_tx)
            .await?;
        // Shielded pool always executes last.
        self.shielded_pool
            .execute_tx(ctx.clone(), tx, state_tx)
            .await?;

        Ok(())
    }
}
