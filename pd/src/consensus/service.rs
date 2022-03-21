use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::FutureExt;
use tendermint::abci::{ConsensusRequest, ConsensusResponse};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::PollSender;
use tower_abci::BoxError;

use super::{Message, Worker};
use crate::{state, RequestExt, Storage};

#[derive(Clone)]
pub struct Consensus {
    queue: PollSender<Message>,
}

impl Consensus {
    pub async fn new(state: state::Writer, storage: Storage) -> anyhow::Result<Self> {
        let (queue_tx, queue_rx) = mpsc::channel(10);

        tokio::spawn(Worker::new(state, storage, queue_rx).await?.run());

        Ok(Self {
            queue: PollSender::new(queue_tx),
        })
    }
}

impl tower::Service<ConsensusRequest> for Consensus {
    type Response = ConsensusResponse;
    type Error = BoxError;
    type Future =
        Pin<Box<dyn Future<Output = Result<ConsensusResponse, BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.queue.poll_reserve(cx).map_err(Into::into)
    }

    fn call(&mut self, req: ConsensusRequest) -> Self::Future {
        let span = req.create_span();
        let (tx, rx) = oneshot::channel();

        self.queue
            .send_item(Message {
                req,
                rsp_sender: tx,
                span,
            })
            .expect("called without `poll_ready`");

        async move { Ok(rx.await.expect("worker error??")) }.boxed()
    }
}
