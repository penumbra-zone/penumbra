use std::{
    collections::BTreeSet,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures::future::FutureExt;
use rand_core::OsRng;
use tendermint::abci::{
    request::{self, BeginBlock, EndBlock},
    response, Request, Response,
};
use tokio::sync::oneshot;
use tower::Service;
use tower_abci::BoxError;
use tracing::instrument;

use penumbra_crypto::{
    merkle::TreeExt,
    merkle::{self, NoteCommitmentTree},
    note, Action, Nullifier, Transaction,
};

use crate::{db::schema, genesis::GenesisNotes, PendingBlock, State};

const ABCI_INFO_VERSION: &str = env!("VERGEN_GIT_SEMVER");

/// The Penumbra ABCI application.
#[derive(Debug)]
pub struct App {
    state: State,

    /// Written to the database after every block commit.
    note_commitment_tree: merkle::BridgeTree<note::Commitment, { merkle::DEPTH as u8 }>,

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
    mempool_nullifiers: BTreeSet<Nullifier>,

    /// Contains all queued state changes for the duration of a block.  This is
    /// set to Some at the beginning of BeginBlock and consumed (and reset to
    /// None) in Commit.
    pending_block: Option<PendingBlock>,

    /// Used to allow asynchronous requests to be processed sequentially.
    completion_tracker: CompletionTracker,
}

impl App {
    /// Create the application with the given DB state.
    #[instrument(skip(state))]
    pub async fn new(state: State) -> Result<Self, anyhow::Error> {
        let note_commitment_tree = state.note_commitment_tree().await?;

        Ok(Self {
            state,
            note_commitment_tree,
            mempool_nullifiers: Default::default(),
            pending_block: None,
            completion_tracker: Default::default(),
        })
    }

    #[instrument(skip(self, init_chain))]
    fn init_genesis(
        &mut self,
        init_chain: request::InitChain,
    ) -> impl Future<Output = Result<Response, BoxError>> {
        tracing::info!(?init_chain);
        let mut genesis_block = PendingBlock::new(NoteCommitmentTree::new(0));
        genesis_block.set_height(0);

        if init_chain.app_state_bytes.is_empty() {
            tracing::warn!("we have no Penumbra-specific state");
        } else {
            // Note that errors cannot be handled in InitChain, the application must crash.
            let genesis: GenesisNotes = serde_json::from_slice(&init_chain.app_state_bytes)
                .expect("can parse app_state in genesis file");

            // Create genesis transaction and update database table `transactions`.
            let mut genesis_tx_builder =
                Transaction::genesis_build_with_root(self.note_commitment_tree.root2());

            for note in genesis.notes() {
                tracing::info!(?note);
                genesis_tx_builder.add_output(&mut OsRng, note);
            }
            let genesis_tx = genesis_tx_builder
                .set_chain_id(init_chain.chain_id)
                .finalize(&mut OsRng)
                .expect("can form genesis transaction");

            // Now add the transaction and its note fragments to the pending state changes.
            genesis_block.add_transaction(genesis_tx);
            tracing::info!("loaded all genesis notes");
        }

        self.pending_block = Some(genesis_block);
        let commit = self.commit();
        let state = self.state.clone();
        async move {
            commit.await?;
            let app_hash = state.app_hash().await.unwrap();
            Ok(Response::InitChain(response::InitChain {
                consensus_params: Some(init_chain.consensus_params),
                validators: init_chain.validators,
                app_hash: app_hash.into(),
            }))
        }
    }

    #[instrument(skip(self))]
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
    }

    #[instrument(skip(self))]
    fn query(&self, query: Bytes) -> response::Query {
        // TODO: implement (#23)
        Default::default()
    }

    #[instrument(skip(self))]
    fn begin_block(&mut self, begin: BeginBlock) -> response::BeginBlock {
        self.pending_block = Some(PendingBlock::new(self.note_commitment_tree.clone()));
        response::BeginBlock::default()
    }

    #[instrument(skip(self))]
    fn deliver_tx(&mut self, tx: Bytes) -> response::DeliverTx {
        // TODO: implement (#135)

        // This should accumulate data from `tx` into `self.pending_block`

        Default::default()
    }

    #[instrument(skip(self))]
    fn end_block(&mut self, end: EndBlock) -> response::EndBlock {
        self.pending_block
            .as_mut()
            .expect("pending_block must be Some in EndBlock processing")
            .set_height(end.height);

        // TODO: here's where we process validator changes
        response::EndBlock::default()
    }

    /// Commit the queued state transitions.
    #[instrument(skip(self))]
    fn commit(&mut self) -> impl Future<Output = Result<Response, BoxError>> {
        let pending_block = self
            .pending_block
            .take()
            .expect("pending_block must be Some when commit is called");

        // These nullifiers are about to be committed, so we don't need
        // to keep them in the mempool nullifier set any longer.
        for nullifier in pending_block.spent_nullifiers.iter() {
            self.mempool_nullifiers.remove(nullifier);
        }

        // Pull the updated note commitment tree.
        self.note_commitment_tree = pending_block.note_commitment_tree.clone();

        let finished_signal = self.completion_tracker.start();
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

            // Signal that we're ready to resume processing further requests.
            let _ = finished_signal.send(());

            Ok(Response::Commit(response::Commit {
                data: app_hash.into(),
                retain_height: 0u32.into(),
            }))
        }
    }

    /// Verifies a transaction and if it verifies, updates the node state.
    ///
    /// TODO: split into stateless and stateful parts.
    pub fn verify_transaction(&mut self, transaction: Transaction) -> bool {
        // 1. Check binding signature.
        if !transaction.verify_binding_sig() {
            return false;
        }

        // 2. Check all spend auth signatures using provided spend auth keys
        // and check all proofs verify. If any action does not verify, the entire
        // transaction has failed.
        let mut nullifiers_to_add = BTreeSet::<Nullifier>::new();
        let mut note_commitments_to_add = Vec::<note::Commitment>::new();

        for action in transaction.transaction_body().actions {
            match action {
                Action::Output(inner) => {
                    if !inner.body.proof.verify(
                        inner.body.value_commitment,
                        inner.body.note_commitment,
                        inner.body.ephemeral_key,
                    ) {
                        return false;
                    }

                    // Queue up the state changes.
                    note_commitments_to_add.push(inner.body.note_commitment);
                }
                Action::Spend(inner) => {
                    if !inner.verify_auth_sig() {
                        return false;
                    }

                    if !inner.body.proof.verify(
                        self.note_commitment_tree.root2(),
                        inner.body.value_commitment,
                        inner.body.nullifier.clone(),
                        inner.body.rk,
                    ) {
                        return false;
                    }

                    // Check nullifier is not already in the nullifier set OR
                    // has been revealed already in this transaction.
                    /*
                    if self.nullifier_set.contains(&inner.body.nullifier.clone())
                        || nullifiers_to_add.contains(&inner.body.nullifier.clone())
                    {
                        return false;
                    }
                    */

                    // Queue up the state changes.
                    nullifiers_to_add.insert(inner.body.nullifier.clone());
                }
            }
        }

        true
    }
}

// Wrapper that allows the service to ensure that the current request's response
// future will complete before processing any further requests.
#[derive(Debug)]
struct CompletionTracker {
    // it would be cleaner to use an Option, but we have to box the oneshot
    // future because it won't be Unpin and Service::poll_ready doesn't require
    // a pinned receiver, so tracking the waiting state in a separate bool allows
    // reallocating a new boxed future every time.
    waiting: bool,
    future: tokio_util::sync::ReusableBoxFuture<Result<(), oneshot::error::RecvError>>,
}

impl CompletionTracker {
    /// Returns a oneshot::Sender used to signal completion of the tracked request.
    pub fn start(&mut self) -> oneshot::Sender<()> {
        assert!(!self.waiting);
        let (tx, rx) = oneshot::channel();
        self.waiting = true;
        self.future.set(rx);
        tx
    }

    pub fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if !self.waiting {
            return Poll::Ready(());
        }

        match self.future.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => {
                self.waiting = false;
                Poll::Ready(())
            }
            Poll::Ready(Err(_)) => {
                tracing::error!("response future of sequentially-processed request was dropped before completion, likely a bug");
                self.waiting = false;
                Poll::Ready(())
            }
        }
    }
}

impl Default for CompletionTracker {
    fn default() -> Self {
        Self {
            future: tokio_util::sync::ReusableBoxFuture::new(async { Ok(()) }),
            waiting: false,
        }
    }
}

impl Service<Request> for App {
    type Response = Response;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Response, BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.completion_tracker.poll_ready(cx).map(|_| Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        tracing::info!(?req);

        let rsp = match req {
            // handled messages
            Request::Info(_) => return self.info().boxed(),
            Request::Query(query) => Response::Query(self.query(query.data)),

            // TODO
            Request::CheckTx(_) => Response::CheckTx(Default::default()),

            Request::BeginBlock(begin) => Response::BeginBlock(self.begin_block(begin)),
            Request::DeliverTx(deliver_tx) => Response::DeliverTx(self.deliver_tx(deliver_tx.tx)),
            Request::EndBlock(end) => Response::EndBlock(self.end_block(end)),
            Request::Commit => return self.commit().boxed(),

            // Called only once for network genesis, i.e. when the application block height is 0.
            Request::InitChain(init_chain) => return self.init_genesis(init_chain).boxed(),

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
    }
}

/*
// TODO: restore this test after writing a state facade (?)
//   OR: don't write a state facade, and test the actual code using test pg states
#[cfg(test)]
mod tests {
    use super::*;

    use ark_ff::Zero;
    use rand_core::OsRng;

    use penumbra_crypto::{keys::SpendKey, memo::MemoPlaintext, Fq, Note, Value};

    #[test]
    fn test_transaction_verification_fails_for_dummy_merkle_tree() {

        let mut app = App::default();

        let mut rng = OsRng;
        let sk_sender = SpendKey::generate(&mut rng);
        let fvk_sender = sk_sender.full_viewing_key();
        let ovk_sender = fvk_sender.outgoing();

        let sk_recipient = SpendKey::generate(&mut rng);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let merkle_root = merkle::Root(Fq::zero());
        let mut merkle_siblings = Vec::new();
        for _i in 0..merkle::DEPTH {
            merkle_siblings.push(note::Commitment(Fq::zero()))
        }
        let dummy_merkle_path: merkle::Path = (merkle::DEPTH, merkle_siblings);

        let value_to_send = Value {
            amount: 10,
            asset_id: b"pen".as_ref().into(),
        };
        let dummy_note = Note::new(
            *dest.diversifier(),
            *dest.transmission_key(),
            value_to_send,
            Fq::zero(),
        )
        .expect("transmission key is valid");

        let transaction = Transaction::build_with_root(merkle_root)
            .set_fee(20)
            .set_chain_id("Pen".to_string())
            .add_output(
                &mut rng,
                &dest,
                value_to_send,
                MemoPlaintext::default(),
                ovk_sender,
            )
            .add_spend(&mut rng, sk_sender, dummy_merkle_path, dummy_note, 0.into())
            .finalize(&mut rng);

        // The merkle path is invalid, so this transaction should not verify.
        assert!(!app.verify_transaction(transaction.unwrap()));
    }
}
    */
