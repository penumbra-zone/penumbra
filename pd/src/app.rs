use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex, RwLock},
    task::{Context, Poll},
};

use anyhow::anyhow;
use bytes::Bytes;
use futures::future::FutureExt;
use penumbra_crypto::{
    asset,
    merkle::{self, NoteCommitmentTree, TreeExt},
    note, Nullifier,
};
use penumbra_proto::Protobuf;
use penumbra_stake::{Epoch, IdentityKey, RateData, ValidatorStatus};
use penumbra_transaction::Transaction;
use tendermint::abci::{
    request::{self, BeginBlock, CheckTxKind, EndBlock},
    response, Request, Response,
};
use tower::Service;
use tower_abci::BoxError;
use tracing::{instrument, Instrument, Span};

use crate::{
    db::schema,
    genesis,
    verify::{mark_genesis_as_verified, StatefulTransactionExt, StatelessTransactionExt},
    PendingBlock, RequestExt, Sequencer, State,
};

const ABCI_INFO_VERSION: &str = env!("VERGEN_GIT_SEMVER");

const NUM_RECENT_ANCHORS: usize = 64;

/// The Penumbra ABCI application.
#[derive(Debug)]
pub struct App {
    state: State,

    /// Written to the database after every block commit.
    note_commitment_tree: merkle::BridgeTree<note::Commitment, { merkle::DEPTH as u8 }>,

    /// Recent anchors of the note commitment tree.
    recent_anchors: VecDeque<merkle::Root>,

    /// We want to prevent two transactions from spending the same note in the
    /// same block.  Our only control over whether transactions will appear in a
    /// block is in `CheckTx`, which gates access to the entire mempool, so we
    /// want to enforce that no two transactions in the mempool spend the same
    /// note.
    ///
    /// To do this, we add a mempool transaction's nullifiers to this set in
    /// `CheckTx` and remove them when we see they've been committed to a block
    /// (in `Commit`).  This means that if Tendermint pulls transactions from
    /// the mempool as part of default block proposer logic, no conflicting
    /// transactions can appear.
    ///
    /// However, it doesn't prevent a malicious validator from proposing
    /// conflicting transactions, so we need to ensure (in `DeliverTx`) that we
    /// ignore invalid transactions.
    mempool_nullifiers: Arc<Mutex<BTreeSet<Nullifier>>>,

    /// Contains all queued state changes for the duration of a block.  This is
    /// set to Some at the beginning of BeginBlock and consumed (and reset to
    /// None) in Commit.
    pending_block: Option<Arc<Mutex<PendingBlock>>>,

    /// Used to allow asynchronous requests to be processed sequentially.
    sequencer: Sequencer,

    /// Epoch duration in blocks
    epoch_duration: u64,

    /// Rate data for the next epoch.
    ///
    /// The rate data is updated only at the epoch boundary but read by both
    /// CheckTx and DeliverTx, so it's kept in an RwLock.  Using the RwLock in
    /// an async context should be fine, because there should be no contention
    /// on reads, as the only time we acquire a write lock is when processing an
    /// epoch boundary (in EndBlock), and we don't process other messages at
    /// that time.
    next_rate_data: Arc<RwLock<BTreeMap<IdentityKey, RateData>>>,
}

impl App {
    /// Create the application with the given DB state.
    #[instrument(skip(state))]
    pub async fn new(state: State) -> Result<Self, anyhow::Error> {
        let note_commitment_tree = state.note_commitment_tree().await?;
        let genesis_config = state.genesis_configuration().await?;
        let recent_anchors = state.recent_anchors(NUM_RECENT_ANCHORS).await?;
        let epoch_duration = genesis_config.epoch_duration;

        // Fetch the next rate data, if any. If there's none, it's because we're
        // pre-genesis, and we'll process an empty list, then overwrite it when
        // we process the genesis block.
        let next_rate_data = state
            .rate_data(
                Epoch::from_height(state.height().await?.value(), epoch_duration)
                    .next()
                    .index,
            )
            .await?
            .into_iter()
            .map(|rate_data| (rate_data.identity_key.clone(), rate_data))
            .collect();

        Ok(Self {
            state,
            note_commitment_tree,
            recent_anchors: recent_anchors,
            mempool_nullifiers: Arc::new(Default::default()),
            pending_block: None,
            sequencer: Default::default(),
            epoch_duration,
            next_rate_data: Arc::new(RwLock::new(next_rate_data)),
        })
    }

    fn init_genesis(
        &mut self,
        init_chain: request::InitChain,
    ) -> impl Future<Output = Result<Response, BoxError>> {
        tracing::info!(?init_chain);
        let mut genesis_block = PendingBlock::new(NoteCommitmentTree::new(0), self.epoch_duration);
        genesis_block.set_height(0);

        // Note that errors cannot be handled in InitChain, the application must crash.
        let app_state: genesis::AppState = serde_json::from_slice(&init_chain.app_state_bytes)
            .expect("can parse app_state in genesis file");

        // Create a genesis transaction to record genesis notes.
        let mut tx_builder = Transaction::genesis_builder();

        for allocation in &app_state.allocations {
            tracing::info!(?allocation, "processing allocation");

            tx_builder.add_output(allocation.note().expect("genesis allocations are valid"));

            let denom = asset::REGISTRY
                .parse_denom(&allocation.denom)
                .expect("genesis allocations must have valid denominations");

            // Accumulate the allocation amount into the supply updates for this denom.
            genesis_block
                .supply_updates
                .entry(denom.id())
                .or_insert((denom, 0))
                .1 += allocation.amount;
        }

        // We might not have any allocations of delegation tokens, but we should record the denoms.
        for genesis::ValidatorPower { validator, .. } in app_state.validators.iter() {
            let denom = validator.identity_key.delegation_token().denom();
            genesis_block
                .supply_updates
                .entry(denom.id())
                .or_insert((denom, 0));
        }

        let genesis_tx = tx_builder
            .set_chain_id(init_chain.chain_id)
            .finalize()
            .expect("can form genesis transaction");
        let verified_transaction = mark_genesis_as_verified(genesis_tx);

        // Now add the transaction and its note fragments to the pending state changes.
        genesis_block.add_transaction(verified_transaction);

        // Extract the Tendermint validators from the genesis app state
        //
        // NOTE: we ignore the validators passed to InitChain.validators, and instead expect them
        // to be provided inside the initial app genesis state (`GenesisAppState`). Returning those
        // validators in InitChain::Response tells Tendermint that they are the initial validator
        // set. See https://docs.tendermint.com/master/spec/abci/abci.html#initchain
        let validators = app_state
            .validators
            .iter()
            .map(|genesis::ValidatorPower { validator, power }| {
                tendermint::abci::types::ValidatorUpdate {
                    pub_key: validator.consensus_key,
                    power: *power,
                }
            })
            .collect();

        self.epoch_duration = app_state.epoch_duration;

        // construct the pending block and commit the initial state
        self.pending_block = Some(Arc::new(Mutex::new(genesis_block)));
        let commit = self.commit();
        let state = self.state.clone();
        let next_rate_data = self.next_rate_data.clone();
        async move {
            // Write genesis data before committing the genesis block.
            state
                .commit_genesis(&app_state)
                .await
                .expect("able to save genesis config to blobs table");

            // Now process the genesis block (note: this is a bit icky, since
            // there's some processing that happens prior to creating the commit
            // future, but its ordering doesn't matter at the moment)
            commit.await?;

            // Initialize the rate data, which is currently empty.  We hardcode
            // the epoch index 1, since we know we're in epoch 0.
            let data = state
                .rate_data(1)
                .await?
                .into_iter()
                .map(|d| (d.identity_key.clone(), d))
                .collect();
            *next_rate_data.write().unwrap() = data;

            let app_hash = state.app_hash().await.unwrap();
            Ok(Response::InitChain(response::InitChain {
                consensus_params: Some(init_chain.consensus_params),
                validators,
                app_hash: app_hash.into(),
            }))
        }
    }

    fn info(&self) -> impl Future<Output = Result<Response, BoxError>> {
        let state = self.state.clone();
        async move {
            let (last_block_height, last_block_app_hash) = match state.latest_block_info().await? {
                Some(schema::BlocksRow {
                    height, app_hash, ..
                }) => (height.try_into().unwrap(), app_hash.into()),
                None => (0u32.into(), vec![0; 32].into()),
            };

            Ok(Response::Info(response::Info {
                data: "penumbra".to_string(),
                version: ABCI_INFO_VERSION.to_string(),
                app_version: 1,
                last_block_height,
                last_block_app_hash,
            }))
        }
        .instrument(Span::current())
    }

    fn query(&self, _query: Bytes) -> response::Query {
        // TODO: implement (#22)
        Default::default()
    }

    fn begin_block(&mut self, _begin: BeginBlock) -> response::BeginBlock {
        self.pending_block = Some(Arc::new(Mutex::new(PendingBlock::new(
            self.note_commitment_tree.clone(),
            self.epoch_duration,
        ))));
        // TODO: process begin.last_commit_info to handle validator rewards, and
        // begin.byzantine_validators to handle evidence + slashing
        response::BeginBlock::default()
    }

    /// Perform checks before adding a transaction into the mempool via `CheckTx`.
    ///
    /// In the transaction validation performed before adding a transaction into the
    /// mempool, we check that:
    ///
    /// * All binding and auth sigs signatures verify (stateless),
    /// * All proofs verify (stateless and stateful),
    /// * The transaction does not reveal nullifiers already revealed in another transaction
    /// in the mempool or in the database,
    ///
    /// If a transaction does not pass these checks, we return a non-zero `CheckTx` response
    /// code, and the transaction will not be added into the mempool.
    ///
    /// We do not queue up any state changes into `PendingBlock` until `DeliverTx` where these
    /// checks are repeated.
    fn check_tx(
        &mut self,
        request: request::CheckTx,
    ) -> impl Future<Output = Result<(), anyhow::Error>> {
        let state = self.state.clone();
        let mempool_nullifiers = self.mempool_nullifiers.clone();
        let recent_anchors = self.recent_anchors.clone();
        let next_rate_data = self.next_rate_data.clone();

        async move {
            let pending_transaction =
                Transaction::try_from(request.tx.as_ref())?.verify_stateless()?;

            // Ensure we do not add any transactions with duplicate nullifiers into the mempool.
            //
            // Note that we only run this logic if this `CheckTx` request is from a new transaction
            // (i.e. `CheckTxKind::New`). If this is a recheck of an existing entry in the mempool,
            // then we don't need to add the nullifier again, as it's already in `self.mempool_nullifiers`.
            // Rechecks occur whenever a block is committed if the Tendermint `mempool.recheck` option is
            // true, which is the default option.
            if request.kind == CheckTxKind::New {
                for nullifier in pending_transaction.spent_nullifiers.clone() {
                    if mempool_nullifiers.lock().unwrap().contains(&nullifier) {
                        return Err(anyhow!(
                            "nullifer {:?} already present in mempool_nullifiers",
                            nullifier
                        ));
                    } else {
                        mempool_nullifiers.lock().unwrap().insert(nullifier);
                    }
                }
            }

            // Ensure that we do not add any transactions that have spent nullifiers in the database.
            for nullifier in pending_transaction.spent_nullifiers.clone() {
                if state
                    .nullifier(nullifier.clone())
                    .await
                    .expect("must be able to fetch nullifier")
                    .is_some()
                {
                    return Err(anyhow!(
                        "nullifer {:?} already present in database",
                        nullifier
                    ));
                };
            }

            pending_transaction
                .verify_stateful(&recent_anchors, &next_rate_data.read().unwrap())?;

            Ok(())
        }
    }

    /// Perform full transaction validation via `DeliverTx`.
    ///
    /// State changes are only applied for valid transactions. Invalid transaction are ignored.
    ///
    /// We must perform all checks again here even though they are performed in `CheckTx`, as a
    /// Byzantine node may propose a block containing double spends or other disallowed behavior,
    /// so it is not safe to assume all checks performed in `CheckTx` were done.
    fn deliver_tx(&mut self, txbytes: Bytes) -> impl Future<Output = Result<(), anyhow::Error>> {
        let state = self.state.clone();
        let recent_anchors = self.recent_anchors.clone();
        let next_rate_data = self.next_rate_data.clone();
        let pending_block_ref = self.pending_block.clone();

        async move {
            let pending_transaction =
                Transaction::try_from(txbytes.as_ref())?.verify_stateless()?;

            for nullifier in pending_transaction.spent_nullifiers.clone() {
                // verify that we're not spending a nullifier that was already spent in a previous block
                if state
                    .nullifier(nullifier.clone())
                    .await
                    .expect("must be able to fetch nullifier")
                    .is_some()
                {
                    return Err(anyhow!(
                        "nullifer {:?} already present in database",
                        nullifier
                    ));
                };
                // verify that we're not spending a nullifier that was already spent in this block
                if pending_block_ref
                    .as_ref()
                    .expect("pending_block must be Some in DeliverTx")
                    .lock()
                    .unwrap()
                    .spent_nullifiers
                    .contains(&nullifier)
                {
                    return Err(anyhow!(
                        "nullifier {:?} was already spent in this block",
                        nullifier
                    ));
                }
            }

            let verified_transaction = pending_transaction
                .verify_stateful(&recent_anchors, &next_rate_data.read().unwrap())?;

            // We accumulate data only for `VerifiedTransaction`s into `PendingBlock`.
            pending_block_ref
                .expect("pending_block must be Some in DeliverTx")
                .lock()
                .unwrap()
                .add_transaction(verified_transaction);

            metrics::increment_counter!("node_transactions_total");
            Ok(())
        }
    }

    fn end_block(&mut self, end: EndBlock) -> impl Future<Output = Result<Response, BoxError>> {
        // Clone a handle to the PendingBlock that we can move into the future
        // below.  It's important not to retain a guard after aquiring a lock,
        // since we'll be locking in an async context.
        let pending_block = self
            .pending_block
            .as_ref()
            .expect("pending_block must be Some in EndBlock")
            .clone();

        let height: u64 = end.height.try_into().expect("height should be nonnegative");
        let epoch = pending_block.lock().unwrap().set_height(height);

        let state = self.state.clone();
        async move {
            if epoch.end_height().value() == height {
                // We've finished processing the last block of `epoch`, so we've
                // crossed the epoch boundary, and (prev | current | next) are:
                let prev_epoch = epoch;
                let current_epoch = prev_epoch.next();
                let next_epoch = current_epoch.next();

                tracing::info!(
                    ?height,
                    ?prev_epoch,
                    ?current_epoch,
                    ?next_epoch,
                    "crossed epoch boundary, processing rate updates"
                );
                metrics::increment_counter!("epoch");

                // TODO (optimization): batch these queries
                let current_base_rate = state.base_rate_data(current_epoch.index).await?;
                let current_rates = state.rate_data(current_epoch.index).await?;

                // steps (foreach validator):
                // - get the total token supply for the validator's delegation tokens
                // - process the updates to the token supply:
                //   - collect all delegations occurring in previous epoch and apply them (adds to supply);
                //   - collect all undelegations started in previous epoch and apply them (reduces supply);
                // - feed the updated (current) token supply into current_rates.voting_power()
                // - persist both the current voting power and the current supply

                /// FIXME: set this less arbitrarily, and allow this to be set per-epoch
                /// 3bps -> 11% return over 365 epochs, why not
                const BASE_REWARD_RATE: u64 = 3_0000;

                let next_base_rate = current_base_rate.next(BASE_REWARD_RATE);

                // rename to curr_rate so it lines up with next_rate (same # chars)
                tracing::debug!(curr_base_rate = ?current_base_rate);
                tracing::debug!(?next_base_rate);

                let mut next_rates = Vec::new();
                let mut next_validator_statuses = Vec::new();
                for current_rate in &current_rates {
                    let identity_key = current_rate.identity_key.clone();

                    let funding_streams = state.funding_streams(identity_key.clone()).await?;

                    let next_rate = current_rate.next(&next_base_rate, funding_streams);

                    let delegation_token_supply = state
                        .asset_lookup(identity_key.delegation_token().id().encode_to_vec())
                        .await?
                        .map(|info| info.total_supply)
                        .unwrap_or(0);

                    // TODO: we should process all of the delegations and undelegations in the
                    // epoch here
                    //
                    // TODO: if a validator isn't part of the consensus set, should we ignore them
                    // and not update their rates?

                    let voting_power =
                        next_rate.voting_power(delegation_token_supply, &next_base_rate);
                    let next_status = ValidatorStatus {
                        identity_key,
                        voting_power,
                    };

                    // rename to curr_rate so it lines up with next_rate (same # chars)
                    tracing::debug!(curr_rate = ?current_rate);
                    tracing::debug!(?next_rate);
                    tracing::debug!(?next_status);

                    next_rates.push(next_rate);
                    next_validator_statuses.push(next_status);
                }

                pending_block.lock().unwrap().next_rates = Some(next_rates);
                pending_block.lock().unwrap().next_base_rate = Some(next_base_rate);
                pending_block.lock().unwrap().next_validator_statuses =
                    Some(next_validator_statuses);

                // TODO: later, set the EndBlock response to add validators
                // at the epoch boundary
            }

            // TODO: right now we are not writing the updated voting power from validator statuses
            // back to tendermint, so that we can see how the statuses are computed without risking
            // halting the testnet. in the future we want to add code here to send the next voting
            // powers back to tendermint.
            Ok(Response::EndBlock(response::EndBlock::default()))
        }
    }

    /// Commit the queued state transitions.
    fn commit(&mut self) -> impl Future<Output = Result<Response, BoxError>> {
        let pending_block_ref = self
            .pending_block
            .take()
            .expect("pending_block must be Some in Commit");

        let pending_block = Arc::try_unwrap(pending_block_ref)
            .expect("can't try_unwrap on Arc<Mutex<PendingBlock>>>")
            .into_inner()
            .expect("cannot access inner PendingBlock");

        // These nullifiers are about to be committed, so we don't need
        // to keep them in the mempool nullifier set any longer.
        for nullifier in pending_block.spent_nullifiers.iter() {
            self.mempool_nullifiers.lock().unwrap().remove(nullifier);
            metrics::increment_counter!("node_spent_nullifiers_total");
        }

        // Pull the updated note commitment tree.
        self.note_commitment_tree = pending_block.note_commitment_tree.clone();
        let anchor = self.note_commitment_tree.root2();
        self.recent_anchors.push_front(anchor);
        if self.recent_anchors.len() > NUM_RECENT_ANCHORS {
            self.recent_anchors.pop_back();
        }

        // If next_rates is set on the block, we're at the end of the epoch,
        // so update our in-memory copy.
        if let Some(ref next_rates) = pending_block.next_rates {
            *self.next_rate_data.write().unwrap() = next_rates
                .iter()
                .cloned()
                .map(|d| (d.identity_key.clone(), d))
                .collect();
        }

        let state = self.state.clone();
        async move {
            state
                .commit_block(pending_block)
                .await
                .expect("block commit should succeed");

            let app_hash = state
                .app_hash()
                .await
                .expect("must be able to fetch apphash");

            Ok(Response::Commit(response::Commit {
                data: app_hash.into(),
                retain_height: 0u32.into(),
            }))
        }
    }
}

impl Service<Request> for App {
    type Response = Response;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Response, BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.sequencer.poll_ready(cx).map(|_| Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        // Create a span for the request, then ensure that the (synchronous)
        // part of the processing is done in that span using `in_scope`.  For
        // requests that are processed asynchronously, we *also* need to use
        // `.instrument(Span::current())` to propagate the span to the future,
        // so that it will be entered every time the future is polled.
        let span = req.create_span();
        span.in_scope(|| {
            let rsp = match req {
                // handled messages
                Request::Info(_) => return self.info().instrument(Span::current()).boxed(),
                Request::Query(query) => Response::Query(self.query(query.data)),
                Request::CheckTx(check_tx) => {
                    // Process CheckTx messages sequentially.
                    // TODO: this requirement is only because we need to avoid
                    // having multiple transactions in the mempool with the same
                    // nullifiers, until we can use ABCI++ and control block
                    // proposals, at which point check_tx can run concurrently.
                    let rsp = self.check_tx(check_tx);
                    let rsp = self.sequencer.execute(rsp.instrument(Span::current()));
                    return async move {
                        let rsp = rsp.await;
                        tracing::info!(?rsp);
                        match rsp {
                            Ok(()) => Ok(Response::CheckTx(response::CheckTx::default())),
                            Err(e) => Ok(Response::CheckTx(response::CheckTx {
                                code: 1,
                                log: e.to_string(),
                                ..Default::default()
                            })),
                        }
                    }
                    .instrument(Span::current())
                    .boxed();
                }
                Request::BeginBlock(begin) => Response::BeginBlock(self.begin_block(begin)),
                Request::DeliverTx(deliver_tx) => {
                    // Process DeliverTx messages sequentially.
                    let rsp = self.deliver_tx(deliver_tx.tx);
                    let rsp = self.sequencer.execute(rsp.instrument(Span::current()));
                    return async move {
                        let rsp = rsp.await;
                        tracing::info!(?rsp);
                        match rsp {
                            Ok(()) => Ok(Response::DeliverTx(response::DeliverTx::default())),
                            Err(e) => Ok(Response::DeliverTx(response::DeliverTx {
                                code: 1,
                                log: e.to_string(),
                                ..Default::default()
                            })),
                        }
                    }
                    .instrument(Span::current())
                    .boxed();
                }
                Request::EndBlock(end) => {
                    let rsp = self.end_block(end);
                    return self
                        .sequencer
                        .execute(rsp.instrument(Span::current()))
                        .boxed();
                }
                Request::Commit => {
                    let rsp = self.commit();
                    return self
                        .sequencer
                        .execute(rsp.instrument(Span::current()))
                        .boxed();
                }

                // Called only once for network genesis, i.e. when the application block height is 0.
                Request::InitChain(init_chain) => {
                    let rsp = self.init_genesis(init_chain);
                    return self
                        .sequencer
                        .execute(rsp.instrument(Span::current()))
                        .boxed();
                }

                // unhandled messages
                Request::Flush => Response::Flush,
                Request::Echo(_) => Response::Echo(Default::default()),
                Request::ListSnapshots => Response::ListSnapshots(Default::default()),
                Request::OfferSnapshot(_) => Response::OfferSnapshot(Default::default()),
                Request::LoadSnapshotChunk(_) => Response::LoadSnapshotChunk(Default::default()),
                Request::ApplySnapshotChunk(_) => Response::ApplySnapshotChunk(Default::default()),
            };
            tracing::info!(?rsp);
            async move { Ok(rsp) }.boxed()
        })
    }
}
