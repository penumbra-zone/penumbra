use std::sync::Arc;

use anyhow::Result;
use penumbra_chain::params::FmdParameters;
use penumbra_chain::{genesis, AppHash, StateWriteExt as _};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{State, StateTransaction, StateWrite, Storage};
use penumbra_transaction::Transaction;
use tendermint::abci::{self, types::ValidatorUpdate};
use tracing::instrument;

use crate::dex::Dex;
use crate::governance::Governance;
use crate::ibc::IBCComponent;
use crate::shielded_pool::ShieldedPool;
use crate::stake::component::{Staking, ValidatorUpdates};
use crate::Component;

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
    pub fn new(state: State) -> Self {
        tracing::info!("initializing App instance");
        Self {
            // We perform the `Arc` wrapping of `State` here to ensure
            // there should be no unexpected copies elsewhere.
            state: Arc::new(state),
        }
    }

    #[instrument(skip(self, app_state))]
    pub async fn init_chain(&mut self, app_state: &genesis::AppState) {
        let state =
            Arc::get_mut(&mut self.state).expect("state Arc should not be referenced elsewhere");

        let mut state_tx = state.begin_transaction();

        state_tx.put_chain_params(app_state.chain_params.clone());

        // TEMP: Hardcoding FMD parameters until we have a mechanism to change them. See issue #1226.
        state_tx.put_current_fmd_parameters(FmdParameters::default());
        state_tx.put_previous_fmd_parameters(FmdParameters::default());

        // TODO: do we actually need to store the app state here?
        state_tx.put(state_key::app_state().into(), app_state.clone());

        // The genesis block height is 0
        state_tx.put_block_height(0);

        Staking::init_chain(&mut state_tx, app_state).await;
        IBCComponent::init_chain(&mut state_tx, app_state).await;
        Dex::init_chain(&mut state_tx, app_state).await;
        Governance::init_chain(&mut state_tx, app_state).await;
        // Shielded pool always executes last.
        ShieldedPool::init_chain(&mut state_tx, app_state).await;

        state_tx.apply();
    }

    #[instrument(skip(self, begin_block))]
    pub async fn begin_block(
        &mut self,
        begin_block: &abci::request::BeginBlock,
    ) -> Vec<abci::Event> {
        let state =
            Arc::get_mut(&mut self.state).expect("state Arc should not be referenced elsewhere");
        let mut state_tx = state.begin_transaction();

        // store the block height
        state_tx.put_block_height(begin_block.header.height.into());
        // store the block time
        state_tx.put_block_timestamp(begin_block.header.time);

        Staking::begin_block(&mut state_tx, begin_block).await;
        IBCComponent::begin_block(&mut state_tx, begin_block).await;
        Dex::begin_block(&mut state_tx, begin_block).await;
        Governance::begin_block(&mut state_tx, begin_block).await;
        // Shielded pool always executes last.
        ShieldedPool::begin_block(&mut state_tx, begin_block).await;

        state_tx.apply()
    }

    #[instrument(skip(self, tx))]
    pub async fn deliver_tx(&mut self, tx: Arc<Transaction>) -> Result<Vec<abci::Event>> {
        Self::check_tx_stateless(tx.clone())?;
        Self::check_tx_stateful(self.state.clone(), tx.clone()).await?;

        // We need to get a mutable reference to the State here, so we use
        // `Arc::get_mut`. At this point, the stateful checks should have completed,
        // leaving us with exclusive access to the Arc<State>.
        let state =
            Arc::get_mut(&mut self.state).expect("state Arc should not be referenced elsewhere");
        let mut state_tx = state.begin_transaction();

        Self::execute_tx(&mut state_tx, tx).await?;

        // At this point, we've completed execution successfully with no errors,
        // so we can apply the transaction to the State. Otherwise, we'd have
        // bubbled up an error and dropped the StateTransaction.
        Ok(state_tx.apply())
    }

    #[instrument(skip(self, end_block))]
    pub async fn end_block(&mut self, end_block: &abci::request::EndBlock) -> Vec<abci::Event> {
        let state =
            Arc::get_mut(&mut self.state).expect("state Arc should not be referenced elsewhere");
        let mut state_tx = state.begin_transaction();

        Staking::end_block(&mut state_tx, end_block).await;
        IBCComponent::end_block(&mut state_tx, end_block).await;
        Dex::end_block(&mut state_tx, end_block).await;
        Governance::end_block(&mut state_tx, end_block).await;
        // Shielded pool always executes last.
        ShieldedPool::end_block(&mut state_tx, end_block).await;

        state_tx.apply()
    }

    /// Commits the application state to persistent storage,
    /// returning the new root hash and storage version.
    ///
    /// This method also resets `self` as if it were constructed
    /// as an empty state over top of the newly written storage.
    ///
    /// TODO: why does this return Result?
    #[instrument(skip(self, storage))]
    pub async fn commit(&mut self, storage: Storage) -> Result<AppHash> {
        // We need to extract the State we've built up to commit it.  Fill in a dummy state.
        let dummy_state = storage.latest_state();
        let state = Arc::try_unwrap(std::mem::replace(&mut self.state, Arc::new(dummy_state)))
            .expect("we have exclusive ownership of the State at commit()");

        // Commit the pending writes, clearing the state.
        let jmt_root = storage.commit(state).await?;
        let app_hash: AppHash = jmt_root.into();

        tracing::debug!(?app_hash, "finished committing state");

        // Get the latest version of the state, now that we've committed it.
        self.state = Arc::new(storage.latest_state());

        Ok(app_hash)
    }

    // TODO: should this just be returned by `commit`? both are called during every `EndBlock`
    pub fn tendermint_validator_updates(&self) -> Vec<ValidatorUpdate> {
        self.state
            .tendermint_validator_updates()
            .expect("tendermint validator updates should be set when called in end_block")
    }

    #[instrument(skip(tx))]
    pub fn check_tx_stateless(tx: Arc<Transaction>) -> Result<()> {
        // TODO: these can all be parallel tasks

        Staking::check_tx_stateless(tx.clone())?;
        IBCComponent::check_tx_stateless(tx.clone())?;
        Dex::check_tx_stateless(tx.clone())?;
        Governance::check_tx_stateless(tx.clone())?;
        ShieldedPool::check_tx_stateless(tx)?;

        Ok(())
    }

    #[instrument(skip(state, tx))]
    async fn check_tx_stateful(state: Arc<State>, tx: Arc<Transaction>) -> Result<()> {
        // TODO: these can all be parallel tasks

        Staking::check_tx_stateful(state.clone(), tx.clone()).await?;
        IBCComponent::check_tx_stateful(state.clone(), tx.clone()).await?;
        Dex::check_tx_stateful(state.clone(), tx.clone()).await?;
        Governance::check_tx_stateful(state.clone(), tx.clone()).await?;
        ShieldedPool::check_tx_stateful(state.clone(), tx.clone()).await?;

        Ok(())
    }

    #[instrument(skip(state, tx))]
    async fn execute_tx(state: &mut StateTransaction<'_>, tx: Arc<Transaction>) -> Result<()> {
        Staking::execute_tx(state, tx.clone()).await?;
        IBCComponent::execute_tx(state, tx.clone()).await?;
        Dex::execute_tx(state, tx.clone()).await?;
        Governance::execute_tx(state, tx.clone()).await?;
        // Shielded pool always executes last.
        ShieldedPool::execute_tx(state, tx.clone()).await?;

        Ok(())
    }
}
