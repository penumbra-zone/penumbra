use std::sync::Arc;

use anyhow::Result;
use penumbra_chain::params::FmdParameters;
use penumbra_chain::NoteSource;
use penumbra_chain::{
    component::{AppHash, StateReadExt as _, StateWriteExt as _},
    genesis,
};
use penumbra_compact_block::component::{StateReadExt as _, StateWriteExt as _};
use penumbra_compact_block::CompactBlock;
use penumbra_component::Component;
use penumbra_crypto::dex::swap::SwapPayload;
use penumbra_crypto::NotePayload;
use penumbra_distributions::component::Distributions;
use penumbra_ibc::component::IBCComponent;
use penumbra_proto::DomainType;
use penumbra_sct::component::SctManager;
use penumbra_shielded_pool::component::{NoteManager, ShieldedPool};
use penumbra_storage::{ArcStateDeltaExt, Snapshot, StateDelta, StateWrite, Storage};
use penumbra_tct as tct;
use penumbra_transaction::Transaction;
use tendermint::abci::{self, Event};
use tendermint::validator::Update;
use tracing::Instrument;

use crate::action_handler::ActionHandler;
use crate::dex::{Dex, SwapManager};
use crate::governance::{Governance, StateReadExt as _};
use crate::stake::component::{Staking, ValidatorUpdates};

pub mod state_key;

/// The inter-block state being written to by the application.
type InterBlockState = Arc<StateDelta<Snapshot>>;

/// The Penumbra application, written as a bundle of [`Component`]s.
///
/// The [`App`] is not a [`Component`], but
/// it constructs the components and exposes a [`commit`](App::commit) that
/// commits the changes to the persistent storage and resets its subcomponents.
pub struct App {
    state: InterBlockState,
}

impl App {
    pub async fn new(snapshot: Snapshot) -> Result<Self> {
        tracing::debug!("initializing App instance");

        // We perform the `Arc` wrapping of `State` here to ensure
        // there should be no unexpected copies elsewhere.
        let state = Arc::new(StateDelta::new(snapshot));

        // If the state says that the chain is halted, we should not proceed. This is a safety check
        // to ensure that automatic restarts by software like systemd do not cause the chain to come
        // back up again after a halt.
        if state.is_chain_halted(TOTAL_HALT_COUNT).await? {
            anyhow::bail!("chain is halted, refusing to restart");
        }

        Ok(Self { state })
    }

    // StateDelta::apply only works when the StateDelta wraps an underlying
    // StateWrite.  But if we want to share the StateDelta with spawned tasks,
    // we usually can't wrap a StateWrite instance, which requires exclusive
    // access. This method "externally" applies the state delta to the
    // inter-block state.
    //
    // Invariant: state_tx and self.state are the only two references to the
    // inter-block state.
    fn apply(&mut self, state_tx: StateDelta<InterBlockState>) -> Vec<Event> {
        let (state2, mut cache) = state_tx.flatten();
        std::mem::drop(state2);
        // Now there is only one reference to the inter-block state: self.state

        let events = cache.take_events();
        cache.apply_to(
            Arc::get_mut(&mut self.state).expect("no other references to inter-block state"),
        );

        events
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

        // The genesis block height is 0
        state_tx.put_block_height(0);

        state_tx.put_epoch_by_height(
            0,
            penumbra_chain::Epoch {
                index: 0,
                start_height: 0,
            },
        );

        // We need to set the epoch for the first block as well, since we set
        // the epoch by height in end_block, and end_block isn't called after init_chain.
        state_tx.put_epoch_by_height(
            1,
            penumbra_chain::Epoch {
                index: 0,
                start_height: 0,
            },
        );

        Distributions::init_chain(&mut state_tx, app_state).await;
        Staking::init_chain(&mut state_tx, app_state).await;
        IBCComponent::init_chain(&mut state_tx, &()).await;
        Dex::init_chain(&mut state_tx, &()).await;
        Governance::init_chain(&mut state_tx, &()).await;
        ShieldedPool::init_chain(&mut state_tx, app_state).await;

        // Create a synthetic height-zero block
        App::finish_block(&mut state_tx).await;

        state_tx.apply();
    }

    pub async fn begin_block(
        &mut self,
        begin_block: &abci::request::BeginBlock,
    ) -> Vec<abci::Event> {
        let mut state_tx = StateDelta::new(self.state.clone());

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
        let mut arc_state_tx = Arc::new(state_tx);
        Distributions::begin_block(&mut arc_state_tx, begin_block).await;
        Staking::begin_block(&mut arc_state_tx, begin_block).await;
        IBCComponent::begin_block(&mut arc_state_tx, begin_block).await;
        Governance::begin_block(&mut arc_state_tx, begin_block).await;
        ShieldedPool::begin_block(&mut arc_state_tx, begin_block).await;

        let state_tx = Arc::try_unwrap(arc_state_tx)
            .expect("components did not retain copies of shared state");

        // Apply the state from `begin_block` and return the events (we'll append to them if
        // necessary based on the results of applying the DAO transactions queued)
        let mut events = self.apply(state_tx);

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
            async move { tx2.check_stateless(()).await }.instrument(tracing::Span::current()),
        );
        let tx2 = tx.clone();
        let state2 = self.state.clone();
        let stateful = tokio::spawn(
            async move { tx2.check_stateful(state2).await }.instrument(tracing::Span::current()),
        );

        stateless.await??;
        stateful.await??;

        // At this point, the stateful checks should have completed,
        // leaving us with exclusive access to the Arc<State>.
        let mut state_tx = self
            .state
            .try_begin_transaction()
            .expect("state Arc should be present and unique");
        tx.execute(&mut state_tx).await?;

        // At this point, we've completed execution successfully with no errors,
        // so we can apply the transaction to the State. Otherwise, we'd have
        // bubbled up an error and dropped the StateTransaction.
        Ok(state_tx.apply().1)
    }

    pub async fn end_block(&mut self, end_block: &abci::request::EndBlock) -> Vec<abci::Event> {
        let state_tx = StateDelta::new(self.state.clone());

        let mut arc_state_tx = Arc::new(state_tx);
        Distributions::end_block(&mut arc_state_tx, end_block).await;
        Staking::end_block(&mut arc_state_tx, end_block).await;
        IBCComponent::end_block(&mut arc_state_tx, end_block).await;
        Dex::end_block(&mut arc_state_tx, end_block).await;
        Governance::end_block(&mut arc_state_tx, end_block).await;
        ShieldedPool::end_block(&mut arc_state_tx, end_block).await;
        let mut state_tx = Arc::try_unwrap(arc_state_tx)
            .expect("components did not retain copies of shared state");

        let current_height = state_tx.get_block_height().await.unwrap();
        let current_epoch = state_tx.epoch().await.unwrap();

        let end_epoch = current_epoch
            .is_scheduled_epoch_end(current_height, state_tx.get_epoch_duration().await.unwrap())
            || state_tx.epoch_ending_early();

        if end_epoch {
            tracing::info!(?current_height, "ending epoch");

            let mut arc_state_tx = Arc::new(state_tx);

            Distributions::end_epoch(&mut arc_state_tx).await.unwrap();
            Staking::end_epoch(&mut arc_state_tx).await.unwrap();
            IBCComponent::end_epoch(&mut arc_state_tx).await.unwrap();
            Dex::end_epoch(&mut arc_state_tx).await.unwrap();
            Governance::end_epoch(&mut arc_state_tx).await.unwrap();
            ShieldedPool::end_epoch(&mut arc_state_tx).await.unwrap();

            let mut state_tx = Arc::try_unwrap(arc_state_tx)
                .expect("components did not retain copies of shared state");

            App::finish_epoch(&mut state_tx).await;

            // set the epoch for the next block
            state_tx.put_epoch_by_height(
                current_height + 1,
                penumbra_chain::Epoch {
                    index: current_epoch.index + 1,
                    start_height: current_height + 1,
                },
            );

            self.apply(state_tx)
        } else {
            // set the epoch for the next block
            state_tx.put_epoch_by_height(current_height + 1, current_epoch);
            App::finish_block(&mut state_tx).await;

            self.apply(state_tx)
        }
    }

    /// Finish an SCT block and use the resulting roots to finalize the current `CompactBlock`.
    pub(crate) async fn finish_block<S: StateWrite>(state: S) {
        Self::finish_compact_block(state, false).await;
    }

    /// Finish an SCT block and epoch and use the resulting roots to finalize the current `CompactBlock`.
    pub(crate) async fn finish_epoch<S: StateWrite>(state: S) {
        Self::finish_compact_block(state, true).await;
    }

    // TODO: move this into the compact block crate as a method in its high-level state-write trait
    // once the dex is separated out into its separate crate
    async fn finish_compact_block<S: StateWrite>(mut state: S, end_epoch: bool) {
        // Find out what our block height is (this is set even during the genesis block)
        let height = state
            .get_block_height()
            .await
            .expect("height of block is always set");

        // Check to see if the chain parameters have changed, and include them in the compact block
        // if they have (this is signaled by `penumbra_chain::StateWriteExt::put_chain_params`):
        let chain_parameters = if state.chain_params_changed() || height == 0 {
            Some(state.get_chain_params().await.unwrap())
        } else {
            None
        };

        let fmd_parameters = if height == 0 {
            Some(state.get_current_fmd_parameters().await.unwrap())
        } else {
            None
        };

        // Check to see if a governance proposal has started, and mark this fact if so.
        let proposal_started = state.proposal_started();

        // End the block in the SCT and record the block root, epoch root if applicable, and the SCT
        // itself, storing the resultant block and epoch root if applicable in the compact block.
        let (block_root, epoch_root) = state.end_sct_block(end_epoch).await.expect(
            "end_sct_block should succeed because we should make sure epochs don't last too long",
        );

        // Pull out all the pending state payloads (note and swap)
        let note_payloads = state
            .pending_note_payloads()
            .await
            .into_iter()
            .map(|(pos, note, source)| (pos, (note, source).into()));
        let rolled_up_payloads = state
            .pending_rolled_up_payloads()
            .await
            .into_iter()
            .map(|(pos, commitment)| (pos, commitment.into()));
        let swap_payloads = state
            .pending_swap_payloads()
            .await
            .into_iter()
            .map(|(pos, swap, source)| (pos, (swap, source).into()));

        // Sort the payloads by position and put them in the compact block
        let mut state_payloads = note_payloads
            .chain(rolled_up_payloads)
            .chain(swap_payloads)
            .collect::<Vec<_>>();
        state_payloads.sort_by_key(|(pos, _)| *pos);
        let state_payloads = state_payloads
            .into_iter()
            .map(|(_, payload)| payload)
            .collect();

        // Gather the swap outputs
        let swap_outputs = state
            .object_get::<im::OrdMap<_, _>>(crate::dex::state_key::pending_outputs())
            .unwrap_or_default()
            .into_iter()
            .collect();

        // Add all the pending nullifiers to the compact block
        let nullifiers = state
            .object_get::<im::Vector<_>>(penumbra_shielded_pool::state_key::pending_nullifiers())
            .unwrap_or_default()
            .into_iter()
            .collect();

        // Output the aggregated final compact block
        state.set_compact_block(CompactBlock {
            height,
            state_payloads,
            nullifiers,
            block_root,
            epoch_root,
            proposal_started,
            swap_outputs,
            fmd_parameters,
            chain_parameters,
        });
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
        let should_halt = state
            .is_chain_halted(TOTAL_HALT_COUNT)
            .await
            .expect("must be able to read halt flag");

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
            // If the tendermint validator updates are not set, we return an empty
            // update set, signaling no change to Tendermint.
            .unwrap_or_default()
    }
}

/// The total number of times the chain has been halted.
///
/// Increment this manually after fixing the root cause for a chain halt: updated nodes will then be
/// able to proceed past the block height of the halt.
const TOTAL_HALT_COUNT: u64 = 0;
