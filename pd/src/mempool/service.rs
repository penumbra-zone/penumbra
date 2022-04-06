use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::FutureExt;
use tendermint::{
    abci::{
        request::CheckTx as CheckTxReq, response::CheckTx as CheckTxRsp, MempoolRequest,
        MempoolResponse,
    },
    block,
};
use tokio::sync::{mpsc, oneshot, watch};
use tokio_util::sync::PollSender;
use tower_abci::BoxError;

use super::{Message, Worker};
use crate::{RequestExt, Storage};

#[derive(Clone)]
pub struct Mempool {
    queue: PollSender<Message>,
}

impl Mempool {
    pub async fn new(
        storage: Storage,
        height_rx: watch::Receiver<block::Height>,
    ) -> anyhow::Result<Self> {
        let (queue_tx, queue_rx) = mpsc::channel(10);

        tokio::spawn(Worker::new(storage, queue_rx, height_rx).await?.run());

        Ok(Self {
            queue: PollSender::new(queue_tx),
        })
    }
}

impl tower::Service<MempoolRequest> for Mempool {
    type Response = MempoolResponse;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<MempoolResponse, BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.queue.poll_reserve(cx).map_err(Into::into)
    }

    fn call(&mut self, req: MempoolRequest) -> Self::Future {
        let span = req.create_span();
        let (tx, rx) = oneshot::channel();

        let MempoolRequest::CheckTx(CheckTxReq { tx: tx_bytes, .. }) = req;

        self.queue
            .send_item(Message {
                tx_bytes,
                rsp_sender: tx,
                span,
            })
            .expect("called without `poll_ready`");

        async move {
            match rx.await.expect("worker error??") {
                Ok(()) => Ok(MempoolResponse::CheckTx(CheckTxRsp::default())),
                Err(e) => Ok(MempoolResponse::CheckTx(CheckTxRsp {
                    code: 1,
                    log: e.to_string(),
                    ..Default::default()
                })),
            }
        }
        .boxed()
    }
}
