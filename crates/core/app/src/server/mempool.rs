use anyhow::Result;

use cnidarium::{Snapshot, Storage};

use tendermint::v0_37::abci::{
    request::CheckTx as CheckTxReq, request::CheckTxKind, response::CheckTx as CheckTxRsp,
    MempoolRequest as Request, MempoolResponse as Response,
};
use tokio::sync::{mpsc, watch};
use tower_actor::Message;
use tracing::Instrument;

use crate::{app::App, metrics};

/// A mempool service that applies transaction checks against an isolated application fork.
pub struct Mempool {
    queue: mpsc::Receiver<Message<Request, Response, tower::BoxError>>,
    snapshot: Snapshot,
    rx_snapshot: watch::Receiver<Snapshot>,
}

impl Mempool {
    pub fn new(
        storage: Storage,
        queue: mpsc::Receiver<Message<Request, Response, tower::BoxError>>,
    ) -> Self {
        let snapshot = storage.latest_snapshot();
        let snapshot_rx = storage.subscribe();

        Self {
            queue,
            snapshot,
            rx_snapshot: snapshot_rx,
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

        let mut app = App::new(self.snapshot.clone());

        match app.deliver_tx_bytes(tx_bytes.as_ref()).await {
            Ok(events) => {
                let elapsed = start.elapsed();
                tracing::info!(?elapsed, "tx accepted");
                metrics::counter!(metrics::MEMPOOL_CHECKTX_TOTAL, "kind" => kind_str, "code" => "0").increment(1);
                Ok(Response::CheckTx(CheckTxRsp {
                    events,
                    ..Default::default()
                }))
            }
            Err(e) => {
                let elapsed = start.elapsed();
                tracing::info!(?e, ?elapsed, "tx rejected");
                metrics::counter!(metrics::MEMPOOL_CHECKTX_TOTAL, "kind" => kind_str, "code" => "1").increment(1);
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
                change = self.rx_snapshot.changed() => {
                    if let Ok(()) = change {
                        let snapshot = self.rx_snapshot.borrow().clone();
                        tracing::debug!(height = ?snapshot.version(), "mempool has rewired to use the latest snapshot");
                        self.snapshot = snapshot;
                    } else {
                        tracing::info!("state notification channel closed, shutting down");
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
