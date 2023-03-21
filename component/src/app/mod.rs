use std::sync::Arc;

use crate::compactblock::view::{StateReadExt as _, StateWriteExt as _};
use crate::sct::view::{StateReadExt as _, StateWriteExt as _};
use anyhow::Result;
use penumbra_chain::params::FmdParameters;
use penumbra_chain::{genesis, AppHash, StateReadExt, StateWriteExt as _};
use penumbra_proto::{DomainType, StateWriteProto};
use penumbra_storage::{ArcStateDeltaExt, Snapshot, StateDelta, StateWrite, Storage};
use penumbra_transaction::Transaction;
use tendermint::abci;
use tendermint::validator::Update;
use tracing::Instrument;

use crate::action_handler::ActionHandler;
use crate::dex::Dex;
use crate::governance::{Governance, StateReadExt as _};
use crate::ibc::IBCComponent;
use crate::shielded_pool::ShieldedPool;
use crate::stake::component::{Staking, ValidatorUpdates};
use crate::stubdex::StubDex;
use crate::Component;

pub mod state_key;
/// The Penumbra application, written as a bundle of [`Component`]s.
///
/// The [`App`] is not a [`Component`], but
/// it constructs the components and exposes a [`commit`](App::commit) that
/// commits the changes to the persistent storage and resets its subcomponents.
pub struct App {
    state: Arc<StateDelta<Snapshot>>,
}

impl App {
    pub fn new(snapshot: Snapshot) -> Self {
        tracing::debug!("initializing App instance");
        Self {
            // We perform the `Arc` wrapping of `State` here to ensure
            // there should be no unexpected copies elsewhere.
            state: Arc::new(StateDelta::new(snapshot)),
        }
    }

    pub async fn init_chain(&mut self, app_state: &genesis::AppState) {
        let mut state_tx = self
            .state
            .try_begin_transaction()
            .expect("state Arc should not be referenced elsewhere");

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
        StubDex::init_chain(&mut state_tx, app_state).await;
        Governance::init_chain(&mut state_tx, app_state).await;
        ShieldedPool::init_chain(&mut state_tx, app_state).await;

        let mut compact_block = state_tx.stub_compact_block();

        // Hard-coded to zero because we are in the genesis block
        // Tendermint starts blocks at 1, so this is a "phantom" compact block
        compact_block.height = 0;

        // Add current FMD parameters to the initial block.
        compact_block.fmd_parameters = Some(state_tx.get_current_fmd_parameters().await.unwrap());

        state_tx.stub_put_compact_block(compact_block);

        App::finish_sct_block(&mut state_tx).await;

        state_tx.apply();
    }

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

        // If a chain parameter change is scheduled for this block, apply it here, before any other
        // component has executed. This ensures that chain parameter changes are consistently
        // applied precisely at the boundary between blocks:
        if let Some(chain_params) = state_tx
            .pending_chain_parameters()
            .await
            .expect("chain params should always be readable")
        {
            tracing::info!(?chain_params, "applying pending chain parameters");
            state_tx.put_chain_params(chain_params);
        }

        // Run each of the begin block handlers for each component, in sequence:
        Staking::begin_block(&mut state_tx, begin_block).await;
        IBCComponent::begin_block(&mut state_tx, begin_block).await;
        StubDex::begin_block(&mut state_tx, begin_block).await;
        Dex::begin_block(&mut state_tx, begin_block).await;
        Governance::begin_block(&mut state_tx, begin_block).await;

        ShieldedPool::begin_block(&mut state_tx, begin_block).await;

        // Apply the state from `begin_block` and return the events (we'll append to them if
        // necessary based on the results of applying the DAO transactions queued)
        let mut events = state_tx.apply().1;

        // Deliver DAO transactions here, before any other block processing (effectively adding
        // synthetic transactions slotted in after the start of the block but before any user
        // transactions)
        let pending_transactions = self
            .state
            .pending_dao_transactions()
            .await
            .expect("DAO transactions should always be readable");
        for transaction in pending_transactions {
            // NOTE: We are *intentionally* using `deliver_tx_allowing_dao_spends` here, rather than
            // `deliver_tx`, because here is the **ONLY** place we want to permit DAO spends, when
            // delivering transactions that have been scheduled by the chain itself for delivery.
            tracing::info!(?transaction, "delivering DAO transaction");
            match self
                .deliver_tx_allowing_dao_spends(Arc::new(transaction))
                .await
            {
                Err(error) => {
                    tracing::warn!(?error, "failed to deliver DAO transaction");
                }
                Ok(dao_tx_events) => events.extend(dao_tx_events),
            }
        }

        events
    }

    /// Wrapper function for [`Self::deliver_tx`]  that decodes from bytes.
    pub async fn deliver_tx_bytes(&mut self, tx_bytes: &[u8]) -> Result<Vec<abci::Event>> {
        let tx = Arc::new(Transaction::decode(tx_bytes)?);
        self.deliver_tx(tx).await
    }

    pub async fn deliver_tx(&mut self, tx: Arc<Transaction>) -> Result<Vec<abci::Event>> {
        // Ensure that any normally-delivered transaction (originating from a user) does not contain
        // any DAO spends or outputs; the only place those are permitted is transactions originating
        // from the chain itself:
        anyhow::ensure!(
            tx.dao_spends().peekable().peek().is_none(),
            "DAO spends are not permitted in user-submitted transactions"
        );
        anyhow::ensure!(
            tx.dao_outputs().peekable().peek().is_none(),
            "DAO outputs are not permitted in user-submitted transactions"
        );

        // Now that we've ensured that there are not any DAO spends or outputs, we can deliver the transaction:
        self.deliver_tx_allowing_dao_spends(tx).await
    }

    async fn deliver_tx_allowing_dao_spends(
        &mut self,
        tx: Arc<Transaction>,
    ) -> Result<Vec<abci::Event>> {
        // Both stateful and stateless checks take the transaction as
        // verification context.  The separate clone of the Arc<Transaction>
        // means it can be passed through the whole tree of checks.
        //
        // We spawn tasks for each set of checks, to do CPU-bound stateless checks
        // and I/O-bound stateful checks at the same time.
        let tx2 = tx.clone();
        let stateless = tokio::spawn(
            async move { tx2.check_stateless(tx2.clone()).await }
                .instrument(tracing::Span::current()),
        );
        let tx2 = tx.clone();
        let state = self.state.clone();
        let stateful = tokio::spawn(
            async move { tx2.check_stateful(state).await }.instrument(tracing::Span::current()),
        );

        stateless.await??;
        stateful.await??;

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
        Ok(state_tx.apply().1)
    }

    pub async fn end_block(&mut self, end_block: &abci::request::EndBlock) -> Vec<abci::Event> {
        let mut state_tx = self
            .state
            .try_begin_transaction()
            .expect("state Arc should not be referenced elsewhere");

        Staking::end_block(&mut state_tx, end_block).await;
        IBCComponent::end_block(&mut state_tx, end_block).await;
        StubDex::end_block(&mut state_tx, end_block).await;
        Dex::end_block(&mut state_tx, end_block).await;
        Governance::end_block(&mut state_tx, end_block).await;
        ShieldedPool::end_block(&mut state_tx, end_block).await;

        let end_epoch = state_tx.epoch().await.unwrap().is_epoch_end(
            state_tx
                .get_block_height()
                .await
                .expect("block height should be set"),
        );

        if end_epoch {
            Staking::end_epoch(&mut state_tx).await.unwrap();
            IBCComponent::end_epoch(&mut state_tx).await.unwrap();
            StubDex::end_epoch(&mut state_tx).await.unwrap();
            Dex::end_epoch(&mut state_tx).await.unwrap();
            Governance::end_epoch(&mut state_tx).await.unwrap();
            ShieldedPool::end_epoch(&mut state_tx).await.unwrap();

            App::finish_sct_epoch(&mut state_tx).await;
        } else {
            App::finish_sct_block(&mut state_tx).await;
        }

        state_tx.apply().1
    }

    /// Finish an SCT block and use the resulting roots to finalize the current `CompactBlock`.
    pub(crate) async fn finish_sct_block<S: StateWrite>(state: S) {
        Self::finish_sct_inner(state, false).await;
    }

    /// Finish an SCT block and epoch and use the resulting roots to finalize the current `CompactBlock`.
    pub(crate) async fn finish_sct_epoch<S: StateWrite>(state: S) {
        Self::finish_sct_inner(state, true).await;
    }

    async fn finish_sct_inner<S: StateWrite>(mut state: S, end_epoch: bool) {
        let height = state
            .get_block_height()
            .await
            .expect("block height should be set");

        // Grab the compact block and SCT from the state to do final processing.
        let mut compact_block = state.stub_compact_block();
        let mut state_commitment_tree = state.stub_state_commitment_tree().await;

        // Set the height of the compact block.
        compact_block.height = height;

        // Check to see if the chain parameters have changed, and include them in the compact block
        // if they have (this is signaled by `penumbra_chain::StateWriteExt::put_chain_params`):
        if state.chain_params_changed() {
            compact_block.chain_parameters = Some(state.get_chain_params().await.unwrap());
        }

        // Close the block in the SCT
        let block_root = state_commitment_tree
            .end_block()
            .expect("ending a block in the state commitment tree can never fail");

        // Put the block root in the compact block
        compact_block.block_root = block_root;

        // If the block ends an epoch, also close the epoch in the SCT
        let epoch_root = if end_epoch {
            let epoch_root = state_commitment_tree
                .end_epoch()
                .expect("ending an epoch in the state commitment tree can never fail");
            Some(epoch_root)
        } else {
            None
        };
        // Put the epoch root, if any, in the compact block
        compact_block.epoch_root = epoch_root;

        state.set_compact_block(compact_block.clone());

        state
            .write_sct(
                compact_block.height,
                state_commitment_tree,
                block_root,
                epoch_root,
            )
            .await;
    }

    /// Commits the application state to persistent storage,
    /// returning the new root hash and storage version.
    ///
    /// This method also resets `self` as if it were constructed
    /// as an empty state over top of the newly written storage.
    pub async fn commit(&mut self, storage: Storage) -> AppHash {
        // We need to extract the State we've built up to commit it.  Fill in a dummy state.
        let dummy_state = StateDelta::new(storage.latest_snapshot());
        let state = Arc::try_unwrap(std::mem::replace(&mut self.state, Arc::new(dummy_state)))
            .expect("we have exclusive ownership of the State at commit()");

        // Check if someone has signaled that we should halt.
        let should_halt = state.should_halt();

        // Commit the pending writes, clearing the state.
        let jmt_root = storage
            .commit(state)
            .await
            .expect("must be able to successfully commit to storage");

        // If we should halt, we should end the process here.
        if should_halt {
            tracing::info!("committed block when a chain halt was signaled; exiting now");
            std::process::exit(0);
        }

        let app_hash: AppHash = jmt_root.into();

        tracing::debug!(?app_hash, "finished committing state");

        // Get the latest version of the state, now that we've committed it.
        self.state = Arc::new(StateDelta::new(storage.latest_snapshot()));

        app_hash
    }

    // TODO: should this just be returned by `commit`? both are called during every `EndBlock`
    pub fn tendermint_validator_updates(&self) -> Vec<Update> {
        self.state
            .tendermint_validator_updates()
            .expect("tendermint validator updates should be set when called in end_block")
    }
}
