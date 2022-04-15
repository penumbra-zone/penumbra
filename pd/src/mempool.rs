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

use futures::FutureExt;
use penumbra_crypto::Nullifier;

use tendermint::abci::{
    request::CheckTx as CheckTxRequest, response::CheckTx as CheckTxResponse, MempoolRequest,
    MempoolResponse,
};
use tokio::sync::Mutex as AsyncMutex;
use tower_abci::BoxError;
use tracing::Instrument;

use crate::RequestExt;

#[derive(Clone)]
pub struct OldMempool {
    // sidecar
    new_mempool: Mempool,
    // old code
    nullifiers: Arc<AsyncMutex<BTreeSet<Nullifier>>>,
}

impl OldMempool {
    pub fn new(new_mempool: Mempool) -> Self {
        let nullifiers = Arc::new(AsyncMutex::new(Default::default()));
        Self {
            nullifiers,
            new_mempool,
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
    async fn check_tx(&self, _check_tx: CheckTxRequest) -> Result<(), anyhow::Error> {
        Err(anyhow::anyhow!("deprecated"))
    }
}

impl tower::Service<MempoolRequest> for OldMempool {
    type Response = MempoolResponse;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<MempoolResponse, BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
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
