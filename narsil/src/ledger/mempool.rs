use anyhow::Result;

use penumbra_storage::{Snapshot, Storage};
use tendermint::abci::{MempoolRequest as Request, MempoolResponse as Response};
use tendermint::v0_34::abci::{
    request::CheckTx as CheckTxReq, request::CheckTxKind, response::CheckTx as CheckTxRsp,
};
use tokio::sync::{mpsc, watch};
use tower_actor::Message;
use tracing::Instrument;

use crate::{ledger::app::App, metrics};

/// When using ABCI, we can't control block proposal directly, so we could
/// potentially end up creating blocks with mutually incompatible transactions.
/// While we'd reject one of them during execution, it's nicer to try to filter
/// them out at the mempool stage. Currently, the way we do this is by having
/// the mempool worker maintain an ephemeral fork of the entire execution state,
/// and execute incoming transactions against the fork.  This prevents
/// conflicting transactions in the local mempool, since we'll update the fork,
/// then reject the second transaction against the forked state. When we learn a
/// new state has been committed, we discard and recreate the ephemeral fork.
///
/// After switching to ABCI++, we can eliminate this mechanism and just build
/// blocks we want.
pub struct Mempool {
    queue: mpsc::Receiver<Message<Request, Response, tower::BoxError>>,
    app: App,
    snapshot_rx: watch::Receiver<Snapshot>,
}

impl Mempool {
    pub fn new(
        storage: Storage,
        queue: mpsc::Receiver<Message<Request, Response, tower::BoxError>>,
    ) -> Self {
        let app = App::new(storage.latest_snapshot());
        let snapshot_rx = storage.subscribe();

        Self {
            queue,
            app,
            snapshot_rx,
        }
    }

    pub async fn check_tx(&mut self, req: Request) -> Result<Response, tower::BoxError> {
        let Request::CheckTx(CheckTxReq {
            tx: tx_bytes, kind, ..
        }) = req;

        let start = tokio::time::Instant::now();
        let kind_str = match kind {
            CheckTxKind::New => "new",
            CheckTxKind::Recheck => "recheck",
        };

        match self.app.deliver_tx_bytes(tx_bytes.as_ref()).await {
            Ok(events) => {
                let elapsed = start.elapsed();
                tracing::info!(?elapsed, "tx accepted");
                metrics::increment_counter!(
                    metrics::MEMPOOL_CHECKTX_TOTAL,
                    "kind" => kind_str,
                    "code" => "0"
                );
                Ok(Response::CheckTx(CheckTxRsp {
                    events,
                    ..Default::default()
                }))
            }
            Err(e) => {
                let elapsed = start.elapsed();
                tracing::info!(?e, ?elapsed, "tx rejected");
                metrics::increment_counter!(
                    metrics::MEMPOOL_CHECKTX_TOTAL,
                    "kind" => kind_str,
                    "code" => "1"
                );
                Ok(Response::CheckTx(CheckTxRsp {
                    code: 1.into(),
                    // Use the alternate format specifier to include the chain of error causes.
                    log: format!("{e:#}"),
                    ..Default::default()
                }))
            }
        }
    }

    pub async fn run(mut self) -> Result<(), tower::BoxError> {
        loop {
            tokio::select! {
                // Use a biased select to poll for height changes *before* polling for messages.
                biased;
                // Check whether the height has changed, which requires us to throw away our
                // ephemeral mempool state, and create a new one based on the new state.
                change = self.snapshot_rx.changed() => {
                    if let Ok(()) = change {
                        let snapshot = self.snapshot_rx.borrow().clone();
                        tracing::debug!(height = ?snapshot.version(), "resetting ephemeral mempool state");
                        self.app = App::new(snapshot);
                    } else {
                        // TODO: what triggers this, now that the channel is owned by the
                        // shared Storage instance, rather than the consensus worker?
                        tracing::info!("state notification channel closed, shutting down");
                        // old: The consensus worker shut down, we should too.
                        return Ok(());
                    }
                }
                message = self.queue.recv() => {
                    if let Some(Message {req, rsp_sender, span }) = message {
                        let _ = rsp_sender.send(self.check_tx(req).instrument(span).await);
                    } else {
                        // The queue is closed, so we're done.
                        return Ok(());
                    }
                }
            }
        }
    }
}
