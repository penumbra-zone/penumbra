use std::sync::Arc;

use anyhow::Result;
use penumbra_chain::params::FmdParameters;
use penumbra_chain::{genesis, AppHash, StateWriteExt as _};
use penumbra_proto::{Protobuf, StateWriteProto};
use penumbra_storage::{ArcStateExt, State, Storage};
use penumbra_transaction::Transaction;
use tendermint::abci::{self, types::ValidatorUpdate};
use tracing::instrument;

use crate::action_handler::ActionHandler;
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
        let mut state_tx = self
            .state
            .try_begin_transaction()
            .expect("state Arc should not be referenced elsewhere");

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

    /// Wrapper function for [`Self::deliver_tx`]  that decodes from bytes.
    pub async fn deliver_tx_bytes(&mut self, tx_bytes: &[u8]) -> Result<Vec<abci::Event>> {
        let tx = Arc::new(Transaction::decode(tx_bytes)?);
        self.deliver_tx(tx).await
    }

    #[instrument(skip(self, tx))]
    pub async fn deliver_tx(&mut self, tx: Arc<Transaction>) -> Result<Vec<abci::Event>> {
        // Both stateful and stateless checks take the transaction as
        // verification context.  The separate clone of the Arc<Transaction>
        // means it can be passed through the whole tree of checks.
        tx.check_stateless(tx.clone()).await?;
        tx.check_stateful(self.state.clone(), tx.clone()).await?;

        // At this point, the stateful checks should have completed,
        // leaving us with exclusive access to the Arc<State>.
        let mut state_tx = self
            .state
            .try_begin_transaction()
            .expect("state Arc should not be referenced elsewhere");
        tx.execute(&mut state_tx).await?;

        // At this point, we've completed execution successfully with no errors,
        // so we can apply the transaction to the State. Otherwise, we'd have
        // bubbled up an error and dropped the StateTransaction.
        Ok(state_tx.apply())
    }

    #[instrument(skip(self, end_block))]
    pub async fn end_block(&mut self, end_block: &abci::request::EndBlock) -> Vec<abci::Event> {
        let mut state_tx = self
            .state
            .try_begin_transaction()
            .expect("state Arc should not be referenced elsewhere");

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
    #[instrument(skip(self, storage))]
    pub async fn commit(&mut self, storage: Storage) -> AppHash {
        // We need to extract the State we've built up to commit it.  Fill in a dummy state.
        let dummy_state = storage.latest_state();
        let state = Arc::try_unwrap(std::mem::replace(&mut self.state, Arc::new(dummy_state)))
            .expect("we have exclusive ownership of the State at commit()");

        // Commit the pending writes, clearing the state.
        let jmt_root = storage
            .commit(state)
            .await
            .expect("must be able to successfully commit to storage");
        let app_hash: AppHash = jmt_root.into();

        tracing::debug!(?app_hash, "finished committing state");

        // Get the latest version of the state, now that we've committed it.
        self.state = Arc::new(storage.latest_state());

        app_hash
    }

    // TODO: should this just be returned by `commit`? both are called during every `EndBlock`
    pub fn tendermint_validator_updates(&self) -> Vec<ValidatorUpdate> {
        self.state
            .tendermint_validator_updates()
            .expect("tendermint validator updates should be set when called in end_block")
    }
}
