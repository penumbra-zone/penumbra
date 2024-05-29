use std::process;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::{ArcStateDeltaExt, Snapshot, StateDelta, StateRead, StateWrite, Storage};
use cnidarium_component::Component;
use ibc_types::core::connection::ChainId;
use jmt::RootHash;
use penumbra_auction::component::{Auction, StateReadExt as _, StateWriteExt as _};
use penumbra_community_pool::component::{CommunityPool, StateWriteExt as _};
use penumbra_community_pool::StateReadExt as _;
use penumbra_compact_block::component::CompactBlockManager;
use penumbra_dex::component::StateReadExt as _;
use penumbra_dex::component::{Dex, StateWriteExt as _};
use penumbra_distributions::component::{Distributions, StateReadExt as _, StateWriteExt as _};
use penumbra_fee::component::{Fee, StateReadExt as _, StateWriteExt as _};
use penumbra_funding::component::Funding;
use penumbra_funding::component::{StateReadExt as _, StateWriteExt as _};
use penumbra_governance::component::{Governance, StateReadExt as _, StateWriteExt as _};
use penumbra_ibc::component::{Ibc, StateWriteExt as _};
use penumbra_ibc::StateReadExt as _;
use penumbra_proto::core::app::v1::TransactionsByHeightResponse;
use penumbra_proto::DomainType;
use penumbra_sct::component::clock::EpochRead;
use penumbra_sct::component::sct::Sct;
use penumbra_sct::component::{StateReadExt as _, StateWriteExt as _};
use penumbra_sct::epoch::Epoch;
use penumbra_shielded_pool::component::{ShieldedPool, StateReadExt as _, StateWriteExt as _};
use penumbra_stake::component::{
    stake::ConsensusUpdateRead, Staking, StateReadExt as _, StateWriteExt as _,
};
use penumbra_transaction::Transaction;
use prost::Message as _;
use tendermint::abci::{self, Event};
use tendermint_proto::v0_34::abci::Event as ProtoEvent;

use tendermint::v0_37::abci::{request, response};
use tendermint::validator::Update;
use tokio::time::sleep;
use tracing::{instrument, Instrument};

use crate::action_handler::AppActionHandler;
use crate::genesis::AppState;
use crate::params::change::ParameterChangeExt as _;
use crate::params::AppParameters;
use crate::{CommunityPoolStateReadExt, PenumbraHost};

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
    /// Constructs a new application, using the provided [`Snapshot`].
    /// Callers should ensure that [`App::is_ready`]) returns `true`, but this is not enforced.
    #[instrument(skip_all)]
    pub fn new(snapshot: Snapshot) -> Self {
        tracing::debug!("initializing App instance");

        // We perform the `Arc` wrapping of `State` here to ensure
        // there should be no unexpected copies elsewhere.
        let state = Arc::new(StateDelta::new(snapshot));

        Self { state }
    }

    /// Returns whether the application is ready to start.
    #[instrument(skip_all, ret)]
    pub async fn is_ready(state: Snapshot) -> bool {
        // If the chain is halted, we are not ready to start the application.
        // This is a safety mechanism to prevent the chain from starting if it
        // is in a halted state.
        !state.is_chain_halted().await
    }

    // StateDelta::apply only works when the StateDelta wraps an underlying
    // StateWrite.  But if we want to share the StateDelta with spawned tasks,
    // we usually can't wrap a StateWrite instance, which requires exclusive
    // access. This method "externally" applies the state delta to the
    // inter-block state.
    //
    // Invariant: `state_tx` and `self.state` are the only two references to the
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

    pub async fn init_chain(&mut self, app_state: &AppState) {
        let mut state_tx = self
            .state
            .try_begin_transaction()
            .expect("state Arc should not be referenced elsewhere");
        match app_state {
            AppState::Content(genesis) => {
                state_tx.put_chain_id(genesis.chain_id.clone());
                Sct::init_chain(&mut state_tx, Some(&genesis.sct_content)).await;
                ShieldedPool::init_chain(&mut state_tx, Some(&genesis.shielded_pool_content)).await;
                Distributions::init_chain(&mut state_tx, Some(&genesis.distributions_content))
                    .await;
                Staking::init_chain(
                    &mut state_tx,
                    Some(&(
                        genesis.stake_content.clone(),
                        genesis.shielded_pool_content.clone(),
                    )),
                )
                .await;
                Ibc::init_chain(&mut state_tx, Some(&genesis.ibc_content)).await;
                Auction::init_chain(&mut state_tx, Some(&genesis.auction_content)).await;
                Dex::init_chain(&mut state_tx, Some(&genesis.dex_content)).await;
                CommunityPool::init_chain(&mut state_tx, Some(&genesis.community_pool_content))
                    .await;
                Governance::init_chain(&mut state_tx, Some(&genesis.governance_content)).await;
                Fee::init_chain(&mut state_tx, Some(&genesis.fee_content)).await;
                Funding::init_chain(&mut state_tx, Some(&genesis.funding_content)).await;

                state_tx
                    .finish_block()
                    .await
                    .expect("must be able to finish compact block");
            }
            AppState::Checkpoint(_) => {
                ShieldedPool::init_chain(&mut state_tx, None).await;
                Distributions::init_chain(&mut state_tx, None).await;
                Staking::init_chain(&mut state_tx, None).await;
                Ibc::init_chain(&mut state_tx, None).await;
                Dex::init_chain(&mut state_tx, None).await;
                Governance::init_chain(&mut state_tx, None).await;
                CommunityPool::init_chain(&mut state_tx, None).await;
                Fee::init_chain(&mut state_tx, None).await;
                Funding::init_chain(&mut state_tx, None).await;
            }
        };

        // Apply the state from `init_chain` and return the events generated during genesis. These
        // cannot be emitted at this time due to a limitation of ABCI, but they will be emitted
        // during the first BeginBlock.
        let events = state_tx.apply().1;

        // Commit the genesis events to the state.
        let mut state_tx = self
            .state
            .try_begin_transaction()
            .expect("state Arc should not be referenced elsewhere");
        for (i, event) in events.into_iter().enumerate() {
            let mut event_bytes = Vec::new();
            tendermint_proto::Protobuf::<ProtoEvent>::encode(event, &mut event_bytes)
                .expect("events are always valid and can be encoded");
            state_tx.nonverifiable_put_raw(state_key::deferred_event(i), event_bytes);
        }
        state_tx.apply();
    }

    pub async fn prepare_proposal(
        &mut self,
        proposal: request::PrepareProposal,
    ) -> response::PrepareProposal {
        if self.state.is_chain_halted().await {
            // If we find ourselves preparing a proposal for a halted chain
            // we stop abruptly to prevent any progress.
            // The persistent halt mechanism will prevent restarts until we are ready.
            process::exit(0);
        }

        let mut included_txs = Vec::new();
        let num_candidate_txs = proposal.txs.len();
        tracing::debug!(
            "processing PrepareProposal, found {} candidate transactions",
            num_candidate_txs
        );

        let mut proposal_size_bytes = 0u64;
        let max_proposal_size_bytes = proposal.max_tx_bytes as u64;
        // The CometBFT spec requires that application "MUST" check that the list
        // of transactions in the proposal does not exceed `max_tx_bytes`. And shed
        // excess transactions so as to be "as close as possible" to the target
        // parameter.
        //
        // A couple things to note about this:
        // - `max_tx_bytes` here is an operator controlled parameter
        // - it is different than the homonymous mempool configuration
        //   parameter controlling the maximum size of a single tx.
        // - the motivation for this check is that even though `PrepareProposal`
        //   is only called by the proposer process, CometBFT might not honor
        //   the target, presuming that some transactions might be yanked.
        // For more details, see the specification:
        // - Adapting existing applications to use ABCI+:
        //  https://github.com/cometbft/cometbft/blob/v0.37.5/spec/abci/abci%2B%2B_comet_expected_behavior.md#adapting-existing-applications-that-use-abci
        // - Application requirements:
        // https://github.com/cometbft/cometbft/blob/v0.37.5/spec/abci/abci%2B%2B_app_requirements
        for tx in proposal.txs {
            let tx_len_bytes = tx.len() as u64;
            proposal_size_bytes = proposal_size_bytes.saturating_add(tx_len_bytes);
            if proposal_size_bytes <= max_proposal_size_bytes {
                included_txs.push(tx);
            } else {
                break;
            }
        }
        tracing::debug!(
            "finished processing PrepareProposal, including {}/{} candidate transactions",
            included_txs.len(),
            num_candidate_txs
        );
        response::PrepareProposal { txs: included_txs }
    }

    pub async fn process_proposal(
        &mut self,
        proposal: request::ProcessProposal,
    ) -> response::ProcessProposal {
        tracing::debug!(?proposal, "processing proposal");
        response::ProcessProposal::Accept
    }

    pub async fn begin_block(&mut self, begin_block: &request::BeginBlock) -> Vec<abci::Event> {
        let mut state_tx = StateDelta::new(self.state.clone());

        // Before anything else, emit the deferred events (only present if right after genesis), and
        // delete each one to prevent it from being emitted again.
        for i in 0.. {
            if let Some(event_bytes) = state_tx
                .nonverifiable_get_raw(&state_key::deferred_event(i))
                .await
                .expect("can attempt to get deferred event")
            {
                let event = tendermint_proto::Protobuf::<ProtoEvent>::decode(&event_bytes[..])
                    .expect("deferred event is always valid");
                state_tx.record(event);
                state_tx.nonverifiable_delete(state_key::deferred_event(i));
            } else {
                break;
            }
        }

        // If a app parameter change is scheduled for this block, apply it here,
        // before any other component has executed. This ensures that app
        // parameter changes are consistently applied precisely at the boundary
        // between blocks.
        //
        // Note that because _nothing_ has executed yet, we need to get the
        // current height from the begin_block request, rather than from the
        // state (it will be set by the SCT component, which executes first).
        if let Some(change) = state_tx
            .param_changes_for_height(begin_block.header.height.into())
            .await
            .expect("param changes should always be readable, even if unset")
        {
            let old_params = state_tx
                .get_app_params()
                .await
                .expect("must be able to read app params");
            match change.apply_changes(old_params) {
                Ok(new_params) => {
                    tracing::info!(?change, "applied app parameter change");
                    state_tx.put_app_params(new_params);
                }
                Err(e) => {
                    // N.B. this is an "info" rather than "warn" because it does not report
                    // a problem with _this instance of the application_, but rather is an expected
                    // behavior.
                    tracing::info!(?change, ?e, "failed to apply approved app parameter change");
                }
            }
        }

        // Run each of the begin block handlers for each component, in sequence:
        let mut arc_state_tx = Arc::new(state_tx);
        Sct::begin_block(&mut arc_state_tx, begin_block).await;
        ShieldedPool::begin_block(&mut arc_state_tx, begin_block).await;
        Distributions::begin_block(&mut arc_state_tx, begin_block).await;
        Ibc::begin_block::<PenumbraHost, StateDelta<Arc<StateDelta<cnidarium::Snapshot>>>>(
            &mut arc_state_tx,
            begin_block,
        )
        .await;
        Auction::begin_block(&mut arc_state_tx, begin_block).await;
        Dex::begin_block(&mut arc_state_tx, begin_block).await;
        CommunityPool::begin_block(&mut arc_state_tx, begin_block).await;
        Governance::begin_block(&mut arc_state_tx, begin_block).await;
        Staking::begin_block(&mut arc_state_tx, begin_block).await;
        Fee::begin_block(&mut arc_state_tx, begin_block).await;
        Funding::begin_block(&mut arc_state_tx, begin_block).await;

        let state_tx = Arc::try_unwrap(arc_state_tx)
            .expect("components did not retain copies of shared state");

        // Apply the state from `begin_block` and return the events (we'll append to them if
        // necessary based on the results of applying the Community Pool transactions queued)
        let mut events = self.apply(state_tx);

        // Deliver Community Pool transactions here, before any other block processing (effectively adding
        // synthetic transactions slotted in after the start of the block but before any user
        // transactions)
        let pending_transactions = self
            .state
            .pending_community_pool_transactions()
            .await
            .expect("Community Pool transactions should always be readable");
        for transaction in pending_transactions {
            // NOTE: We are *intentionally* using `deliver_tx_allowing_community_pool_spends` here, rather than
            // `deliver_tx`, because here is the **ONLY** place we want to permit Community Pool spends, when
            // delivering transactions that have been scheduled by the chain itself for delivery.
            tracing::info!(?transaction, "delivering Community Pool transaction");
            match self
                .deliver_tx_allowing_community_pool_spends(Arc::new(transaction))
                .await
            {
                Err(error) => {
                    tracing::warn!(?error, "failed to deliver Community Pool transaction");
                }
                Ok(community_pool_tx_events) => events.extend(community_pool_tx_events),
            }
        }

        events
    }

    /// Wrapper function for [`Self::deliver_tx`]  that decodes from bytes.
    pub async fn deliver_tx_bytes(&mut self, tx_bytes: &[u8]) -> Result<Vec<abci::Event>> {
        let tx = Arc::new(Transaction::decode(tx_bytes).context("decoding transaction")?);
        self.deliver_tx(tx)
            .await
            .context("failed to deliver transaction")
    }

    pub async fn deliver_tx(&mut self, tx: Arc<Transaction>) -> Result<Vec<abci::Event>> {
        // Ensure that any normally-delivered transaction (originating from a user) does not contain
        // any Community Pool spends or outputs; the only place those are permitted is transactions originating
        // from the chain itself:
        anyhow::ensure!(
            tx.community_pool_spends().peekable().peek().is_none(),
            "Community Pool spends are not permitted in user-submitted transactions"
        );
        anyhow::ensure!(
            tx.community_pool_outputs().peekable().peek().is_none(),
            "Community Pool outputs are not permitted in user-submitted transactions"
        );

        // Now that we've ensured that there are not any Community Pool spends or outputs, we can deliver the transaction:
        self.deliver_tx_allowing_community_pool_spends(tx).await
    }

    async fn deliver_tx_allowing_community_pool_spends(
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
            async move { tx2.check_historical(state2).await }.instrument(tracing::Span::current()),
        );

        stateless
            .await
            .context("waiting for check_stateless check tasks")?
            .context("check_stateless failed")?;
        stateful
            .await
            .context("waiting for check_stateful tasks")?
            .context("check_stateful failed")?;

        // At this point, the stateful checks should have completed,
        // leaving us with exclusive access to the Arc<State>.
        let mut state_tx = self
            .state
            .try_begin_transaction()
            .expect("state Arc should be present and unique");

        // Index the transaction:
        let height = state_tx.get_block_height().await?;
        let transaction = Arc::as_ref(&tx).clone();
        state_tx
            .put_block_transaction(height, transaction.into())
            .await
            .context("storing transactions")?;

        tx.check_and_execute(&mut state_tx)
            .await
            .context("executing transaction")?;

        // At this point, we've completed execution successfully with no errors,
        // so we can apply the transaction to the State. Otherwise, we'd have
        // bubbled up an error and dropped the StateTransaction.
        Ok(state_tx.apply().1)
    }

    #[tracing::instrument(skip_all, fields(height = %end_block.height))]
    pub async fn end_block(&mut self, end_block: &request::EndBlock) -> Vec<abci::Event> {
        let state_tx = StateDelta::new(self.state.clone());

        tracing::debug!("running app components' `end_block` hooks");
        let mut arc_state_tx = Arc::new(state_tx);
        Sct::end_block(&mut arc_state_tx, end_block).await;
        ShieldedPool::end_block(&mut arc_state_tx, end_block).await;
        Distributions::end_block(&mut arc_state_tx, end_block).await;
        Ibc::end_block(&mut arc_state_tx, end_block).await;
        Auction::end_block(&mut arc_state_tx, end_block).await;
        Dex::end_block(&mut arc_state_tx, end_block).await;
        CommunityPool::end_block(&mut arc_state_tx, end_block).await;
        Governance::end_block(&mut arc_state_tx, end_block).await;
        Staking::end_block(&mut arc_state_tx, end_block).await;
        Fee::end_block(&mut arc_state_tx, end_block).await;
        Funding::end_block(&mut arc_state_tx, end_block).await;
        let mut state_tx = Arc::try_unwrap(arc_state_tx)
            .expect("components did not retain copies of shared state");
        tracing::debug!("finished app components' `end_block` hooks");

        let current_height = state_tx
            .get_block_height()
            .await
            .expect("able to get block height in end_block");
        let current_epoch = state_tx
            .get_current_epoch()
            .await
            .expect("able to get current epoch in end_block");

        let is_end_epoch = current_epoch.is_scheduled_epoch_end(
            current_height,
            state_tx
                .get_epoch_duration_parameter()
                .await
                .expect("able to get epoch duration in end_block"),
        ) || state_tx.is_epoch_ending_early().await;

        // If a chain upgrade is scheduled for the next block, we trigger an early epoch change
        // so that the upgraded chain starts at a clean epoch boundary.
        let is_chain_upgrade = state_tx
            .is_pre_upgrade_height()
            .await
            .expect("able to detect upgrade heights");

        if is_end_epoch || is_chain_upgrade {
            tracing::info!(%is_end_epoch, %is_chain_upgrade, ?current_height, "ending epoch");

            let mut arc_state_tx = Arc::new(state_tx);

            Sct::end_epoch(&mut arc_state_tx)
                .await
                .expect("able to call end_epoch on Sct component");
            Distributions::end_epoch(&mut arc_state_tx)
                .await
                .expect("able to call end_epoch on Distributions component");
            Ibc::end_epoch(&mut arc_state_tx)
                .await
                .expect("able to call end_epoch on IBC component");
            Auction::end_epoch(&mut arc_state_tx)
                .await
                .expect("able to call end_epoch on auction component");
            Dex::end_epoch(&mut arc_state_tx)
                .await
                .expect("able to call end_epoch on dex component");
            CommunityPool::end_epoch(&mut arc_state_tx)
                .await
                .expect("able to call end_epoch on Community Pool component");
            Governance::end_epoch(&mut arc_state_tx)
                .await
                .expect("able to call end_epoch on Governance component");
            ShieldedPool::end_epoch(&mut arc_state_tx)
                .await
                .expect("able to call end_epoch on shielded pool component");
            Staking::end_epoch(&mut arc_state_tx)
                .await
                .expect("able to call end_epoch on Staking component");
            Fee::end_epoch(&mut arc_state_tx)
                .await
                .expect("able to call end_epoch on Fee component");
            Funding::end_epoch(&mut arc_state_tx)
                .await
                .expect("able to call end_epoch on Funding component");

            let mut state_tx = Arc::try_unwrap(arc_state_tx)
                .expect("components did not retain copies of shared state");

            state_tx
                .finish_epoch()
                .await
                .expect("must be able to finish compact block");

            // set the epoch for the next block
            penumbra_sct::component::clock::EpochManager::put_epoch_by_height(
                &mut state_tx,
                current_height + 1,
                Epoch {
                    index: current_epoch.index + 1,
                    start_height: current_height + 1,
                },
            );

            self.apply(state_tx)
        } else {
            // set the epoch for the next block
            penumbra_sct::component::clock::EpochManager::put_epoch_by_height(
                &mut state_tx,
                current_height + 1,
                current_epoch,
            );

            state_tx
                .finish_block()
                .await
                .expect("must be able to finish compact block");

            self.apply(state_tx)
        }
    }

    /// Commits the application state to persistent storage,
    /// returning the new root hash and storage version.
    ///
    /// This method also resets `self` as if it were constructed
    /// as an empty state over top of the newly written storage.
    pub async fn commit(&mut self, storage: Storage) -> RootHash {
        // We need to extract the State we've built up to commit it.  Fill in a dummy state.
        let dummy_state = StateDelta::new(storage.latest_snapshot());
        let mut state = Arc::try_unwrap(std::mem::replace(&mut self.state, Arc::new(dummy_state)))
            .expect("we have exclusive ownership of the State at commit()");

        // Check if an emergency halt has been signaled.
        let should_halt = state.is_chain_halted().await;

        let is_pre_upgrade_height = state
            .is_pre_upgrade_height()
            .await
            .expect("must be able to read upgrade height");

        // If the next height is an upgrade height, we signal a halt and turn
        // a `halt_bit` on which will prevent the chain from restarting without
        // running a migration.
        if is_pre_upgrade_height {
            tracing::info!("pre-upgrade height reached, signaling halt");
            state.signal_halt();
        }

        // Commit the pending writes, clearing the state.
        let jmt_root = storage
            .commit(state)
            .await
            .expect("must be able to successfully commit to storage");

        // We want to halt the node, but not before we submit an ABCI `Commit`
        // response to `CometBFT`. To do this, we schedule a process exit in `2s`,
        // assuming a `5s` timeout.
        // See #4443 for more context.
        if should_halt || is_pre_upgrade_height {
            tokio::spawn(async move {
                sleep(Duration::from_secs(2)).await;
                tracing::info!("halt signal recorded, exiting process");
                std::process::exit(0);
            });
        }

        tracing::debug!(?jmt_root, "finished committing state");

        // Get the latest version of the state, now that we've committed it.
        self.state = Arc::new(StateDelta::new(storage.latest_snapshot()));

        jmt_root
    }

    pub fn tendermint_validator_updates(&self) -> Vec<Update> {
        self.state
            .cometbft_validator_updates()
            // If the tendermint validator updates are not set, we return an empty
            // update set, signaling no change to Tendermint.
            .unwrap_or_default()
    }
}

#[async_trait]
pub trait StateReadExt: StateRead {
    async fn get_chain_id(&self) -> Result<String> {
        let raw_chain_id = self
            .get_raw(state_key::data::chain_id())
            .await?
            .expect("chain id is always set");

        Ok(String::from_utf8_lossy(&raw_chain_id).to_string())
    }

    /// Checks a provided chain_id against the chain state.
    ///
    /// Passes through if the provided chain_id is empty or matches, and
    /// otherwise errors.
    async fn check_chain_id(&self, provided: &str) -> Result<()> {
        let chain_id = self
            .get_chain_id()
            .await
            .context(format!("error getting chain id: '{provided}'"))?;
        if provided.is_empty() || provided == chain_id {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "provided chain_id {} does not match chain_id {}",
                provided,
                chain_id
            ))
        }
    }

    /// Gets the chain revision number, from the chain ID
    async fn get_revision_number(&self) -> Result<u64> {
        let cid_str = self.get_chain_id().await?;

        Ok(ChainId::from_string(&cid_str).version())
    }

    /// Returns the set of app parameters
    async fn get_app_params(&self) -> Result<AppParameters> {
        let chain_id = self.get_chain_id().await?;
        let community_pool_params: penumbra_community_pool::params::CommunityPoolParameters =
            self.get_community_pool_params().await?;
        let distributions_params = self.get_distributions_params().await?;
        let ibc_params = self.get_ibc_params().await?;
        let fee_params = self.get_fee_params().await?;
        let funding_params = self.get_funding_params().await?;
        let governance_params = self.get_governance_params().await?;
        let sct_params = self.get_sct_params().await?;
        let shielded_pool_params = self.get_shielded_pool_params().await?;
        let stake_params = self.get_stake_params().await?;
        let dex_params = self.get_dex_params().await?;
        let auction_params = self.get_auction_params().await?;

        Ok(AppParameters {
            chain_id,
            auction_params,
            community_pool_params,
            distributions_params,
            fee_params,
            funding_params,
            governance_params,
            ibc_params,
            sct_params,
            shielded_pool_params,
            stake_params,
            dex_params,
        })
    }

    async fn transactions_by_height(
        &self,
        block_height: u64,
    ) -> Result<TransactionsByHeightResponse> {
        let transactions = match self
            .nonverifiable_get_raw(
                state_key::cometbft_data::transactions_by_height(block_height).as_bytes(),
            )
            .await?
        {
            Some(transactions) => transactions,
            None => TransactionsByHeightResponse {
                transactions: vec![],
                block_height,
            }
            .encode_to_vec(),
        };

        Ok(TransactionsByHeightResponse::decode(&transactions[..])?)
    }
}

impl<
        T: StateRead
            + penumbra_stake::StateReadExt
            + penumbra_governance::component::StateReadExt
            + penumbra_fee::component::StateReadExt
            + penumbra_community_pool::component::StateReadExt
            + penumbra_sct::component::clock::EpochRead
            + penumbra_ibc::component::StateReadExt
            + penumbra_distributions::component::StateReadExt
            + ?Sized,
    > StateReadExt for T
{
}

#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Sets the chain ID.
    fn put_chain_id(&mut self, chain_id: String) {
        self.put_raw(state_key::data::chain_id().into(), chain_id.into_bytes());
    }

    /// Stores the transactions that occurred during a CometBFT block.
    /// This is used to create a durable transaction log for clients to retrieve;
    /// the CometBFT `get_block_by_height` RPC call will only return data for blocks
    /// since the last checkpoint, so we need to store the transactions separately.
    async fn put_block_transaction(
        &mut self,
        height: u64,
        transaction: penumbra_proto::core::transaction::v1::Transaction,
    ) -> Result<()> {
        // Extend the existing transactions with the new one.
        let mut transactions_response = self.transactions_by_height(height).await?;
        transactions_response.transactions = transactions_response
            .transactions
            .into_iter()
            .chain(std::iter::once(transaction))
            .collect();

        self.nonverifiable_put_raw(
            state_key::cometbft_data::transactions_by_height(height).into(),
            transactions_response.encode_to_vec(),
        );
        Ok(())
    }

    /// Writes the app parameters to the state.
    ///
    /// Each component stores its own parameters separately, so this method
    /// splits up the provided parameters structure and writes it out to each component.
    fn put_app_params(&mut self, params: AppParameters) {
        // To make sure we don't forget to write any parts, destructure the entire params
        let AppParameters {
            chain_id,
            auction_params,
            community_pool_params,
            distributions_params,
            fee_params,
            funding_params,
            governance_params,
            ibc_params,
            sct_params,
            shielded_pool_params,
            stake_params,
            dex_params,
        } = params;

        // Ignore writes to the chain_id
        // TODO(erwan): we are momentarily not supporting chain_id changes
        // until the IBC host chain changes land.
        // See: https://github.com/penumbra-zone/penumbra/issues/3617#issuecomment-1917708221
        std::mem::drop(chain_id);

        self.put_auction_params(auction_params);
        self.put_community_pool_params(community_pool_params);
        self.put_distributions_params(distributions_params);
        self.put_fee_params(fee_params);
        self.put_funding_params(funding_params);
        self.put_governance_params(governance_params);
        self.put_ibc_params(ibc_params);
        self.put_sct_params(sct_params);
        self.put_shielded_pool_params(shielded_pool_params);
        self.put_stake_params(stake_params);
        self.put_dex_params(dex_params);
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
