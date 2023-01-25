use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::Context as _;
use futures::FutureExt;
use penumbra_storage::Storage;
use tendermint::abci::{
    request::CheckTx as CheckTxReq, request::CheckTxKind, response::CheckTx as CheckTxRsp,
    MempoolRequest, MempoolResponse,
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::PollSender;
use tower_abci::BoxError;
use tracing::{error_span, Instrument};

use super::{Message, Worker};
use crate::metrics;
use crate::RequestExt;

#[derive(Clone)]
pub struct Mempool {
    queue: PollSender<Message>,
}

impl Mempool {
    pub async fn new(storage: Storage) -> anyhow::Result<Self> {
        let (queue_tx, queue_rx) = mpsc::channel(10);

        tokio::task::Builder::new()
            .name("mempool::Worker")
            .spawn(Worker::new(storage, queue_rx).await?.run())
            .expect("failed to spawn mempool worker");

        Ok(Self {
            queue: PollSender::new(queue_tx),
        })
    }
}

impl tower_service::Service<MempoolRequest> for Mempool {
    type Response = MempoolResponse;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<MempoolResponse, BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.queue.poll_reserve(cx).map_err(Into::into)
    }

    fn call(&mut self, req: MempoolRequest) -> Self::Future {
        // Check if the worker has terminated. We do this again in `call`
        // because the worker may have terminated *after* `poll_ready` reserved
        // a send permit.
        if self.queue.is_closed() {
            return async move {
                Err(anyhow::anyhow!("mempool worker terminated or panicked").into())
            }
            .boxed();
        }
        let span = req.create_span();
        let span = error_span!(parent: &span, "app", role = "mempool");
        let (tx, rx) = oneshot::channel();

        let MempoolRequest::CheckTx(CheckTxReq {
            tx: tx_bytes, kind, ..
        }) = req;

        self.queue
            .send_item(Message {
                tx_bytes,
                rsp_sender: tx,
                span: span.clone(),
            })
            .expect("called without `poll_ready`");

        async move {
            let kind_str = match kind {
                CheckTxKind::New => "new",
                CheckTxKind::Recheck => "recheck",
            };

            match rx.await.context("mempool worker terminated or panicked")? {
                Ok(()) => {
                    tracing::info!("tx accepted");
                    metrics::increment_counter!(
                        metrics::MEMPOOL_CHECKTX_TOTAL,
                        "kind" => kind_str,
                        "code" => "0"
                    );
                    Ok(MempoolResponse::CheckTx(CheckTxRsp::default()))
                }
                Err(e) => {
                    tracing::info!(?e, "tx rejected");
                    metrics::increment_counter!(
                        metrics::MEMPOOL_CHECKTX_TOTAL,
                        "kind" => kind_str,
                        "code" => "1"
                    );
                    Ok(MempoolResponse::CheckTx(CheckTxRsp {
                        code: 1,
                        // Use the alternate format specifier to include the chain of error causes.
                        log: format!("{:#}", e),
                        ..Default::default()
                    }))
                }
            }
        }
        .instrument(span)
        .boxed()
    }
}
