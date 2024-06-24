use anyhow::Result;

use cnidarium::Storage;

use tendermint::v0_37::abci::{
    request::CheckTx as CheckTxReq, request::CheckTxKind, response::CheckTx as CheckTxRsp,
    MempoolRequest as Request, MempoolResponse as Response,
};
use tokio::sync::mpsc;
use tower_actor::Message;
use tracing::Instrument;

use crate::{app::App, metrics};

/// A mempool service that applies transaction checks against an isolated application fork.
pub struct Mempool {
    queue: mpsc::Receiver<Message<Request, Response, tower::BoxError>>,
    storage: Storage,
}

impl Mempool {
    pub fn new(
        storage: Storage,
        queue: mpsc::Receiver<Message<Request, Response, tower::BoxError>>,
    ) -> Self {
        Self { queue, storage }
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

        let mut app = App::new(self.storage.latest_snapshot());

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
        tracing::info!("mempool service started");
        while let Some(Message {
            req,
            rsp_sender,
            span,
            // We could perform `CheckTx` asynchronously, and poll many
            // entries from the queue:
            // See https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.Receiver.html#method.recv_many
        }) = self.queue.recv().await
        {
            let result = self.check_tx(req).instrument(span).await;
            let _ = rsp_sender.send(result);
        }
        tracing::info!("mempool service stopped");
        Ok(())
    }
}
