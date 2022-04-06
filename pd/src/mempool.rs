mod message;
mod service;
mod worker;

use message::Message;
pub use service::Mempool;
use worker::Worker;

// Old code below

use std::{
    collections::BTreeSet,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use anyhow::anyhow;
use futures::FutureExt;
use penumbra_crypto::Nullifier;
use penumbra_proto::Protobuf;
use penumbra_transaction::Transaction;
use tendermint::{
    abci::{
        request::CheckTx as CheckTxRequest, response::CheckTx as CheckTxResponse, MempoolRequest,
        MempoolResponse,
    },
    block,
};
use tokio::sync::{watch, Mutex as AsyncMutex};
use tower_abci::BoxError;
use tracing::Instrument;

use crate::{state, verify::StatelessTransactionExt, RequestExt};

#[derive(Clone)]
pub struct OldMempool {
    // sidecar
    new_mempool: Mempool,
    // old code
    nullifiers: Arc<AsyncMutex<BTreeSet<Nullifier>>>,
    state: state::Reader,
    // We keep our own copy of the height watcher rather than borrowing from our
    // state::Reader so we can mutate it while tracking height updates.
    height_rx: watch::Receiver<block::Height>,
}

impl OldMempool {
    pub fn new(state: state::Reader, new_mempool: Mempool) -> Self {
        let nullifiers = Arc::new(AsyncMutex::new(Default::default()));
        let height_rx = state.height_rx().clone();
        Self {
            nullifiers,
            state,
            new_mempool,
            height_rx,
        }
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
    async fn check_tx(&self, check_tx: CheckTxRequest) -> Result<(), anyhow::Error> {
        // Verify the transaction is well-formed...
        let transaction = Transaction::decode(check_tx.tx)?;
        tracing::info!(?transaction, ?check_tx.kind);
        // ... and that it is internally consistent ...
        let transaction = transaction.verify_stateless()?;
        // ... and that it is consistent with the existing chain state.
        let block_validators = self.state.validator_info(true).await?;
        let transaction = self
            .state
            .verify_stateful(transaction, block_validators.iter())
            .await?;

        // We've verified that the transaction is consistent with the existing
        // chain state, but we want to ensure that it doesn't conflict with any
        // transactions already in the mempool, at least until we migrate to
        // ABCI++ and can control block proposal.  (At that time, we can allow
        // conflicting transactions in the mempool, but only include one of them
        // in a block.)

        // There are two kinds of transaction checks, New and Recheck.  Rechecks
        // are done on any transactions still in the mempool following a block
        // commit. Since we clear the mempool nullifier set on block commits (in
        // the poll_ready implementation), we don't need to handle that case specially.

        // We need to check-and-insert the whole batch transactionally,
        // so we need to hold the lock for the whole check.
        let mut nullifiers = self.nullifiers.lock().await;

        for nf in &transaction.spent_nullifiers {
            if nullifiers.contains(nf) {
                return Err(anyhow!("nullifier {:?} already spent in mempool", nf));
            }
        }

        for nf in transaction.spent_nullifiers {
            nullifiers.insert(nf);
        }

        Ok(())
    }
}

impl tower::Service<MempoolRequest> for OldMempool {
    type Response = MempoolResponse;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<MempoolResponse, BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Check whether a new block has arrived since our last CheckTx request.
        if self.height_rx.has_changed()? {
            // Wipe our mempool nullifier set.  Notice that this leaves any
            // *clones* of the previous version of the mempool nullifier set
            // unchanged, so any in-flight CheckTx requests will continue to use
            // the previous version.  This is so that we don't need to wait to
            // acquire a lock; it's fine because use of the old copy is more
            // restrictive than use of the new copy (which has no nullifiers in
            // it).
            self.nullifiers = Arc::new(AsyncMutex::new(Default::default()));
            // Finally, mark the new height as having been seen.
            self.height_rx.borrow_and_update();
        }
        self.new_mempool.poll_ready(cx)
    }

    fn call(&mut self, req: MempoolRequest) -> Self::Future {
        let req2 = req.clone();
        let span = req.create_span();
        let MempoolRequest::CheckTx(check_tx) = req;
        let mempool = self.clone();

        let rsp2 = self.new_mempool.call(req2);

        tokio::spawn(
            async {
                let rsp2 = rsp2.await;
                tracing::info!(?rsp2, "new mempool response");
            }
            .instrument(span.clone()),
        );

        async move {
            match mempool.check_tx(check_tx).await {
                Ok(()) => Ok(MempoolResponse::CheckTx(CheckTxResponse::default())),
                Err(e) => Ok(MempoolResponse::CheckTx(CheckTxResponse {
                    code: 1,
                    log: e.to_string(),
                    ..Default::default()
                })),
            }
        }
        .instrument(span)
        .boxed()
    }
}
