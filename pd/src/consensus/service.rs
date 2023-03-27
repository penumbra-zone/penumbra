use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::FutureExt;
use penumbra_storage::Storage;
use tendermint::abci::{ConsensusRequest, ConsensusResponse};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::PollSender;
use tower_abci::BoxError;

use super::{Message, Worker};

#[derive(Clone)]
pub struct Consensus {
    queue: PollSender<Message>,
}

impl Consensus {
    pub async fn new(storage: Storage) -> anyhow::Result<Self> {
        let (queue_tx, queue_rx) = mpsc::channel(10);

        tokio::task::Builder::new()
            .name("consensus::Worker")
            .spawn(Worker::new(storage, queue_rx).await?.run())
            .expect("failed to spawn consensus worker");

        Ok(Self {
            queue: PollSender::new(queue_tx),
        })
    }
}

impl tower_service::Service<ConsensusRequest> for Consensus {
    type Response = ConsensusResponse;
    type Error = BoxError;
    type Future =
        Pin<Box<dyn Future<Output = Result<ConsensusResponse, BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.queue.poll_reserve(cx).map_err(Into::into)
    }

    fn call(&mut self, req: ConsensusRequest) -> Self::Future {
        // Check if the worker has terminated. We do this again in `call`
        // because the worker may have terminated *after* `poll_ready` reserved
        // a send permit.
        if self.queue.is_closed() {
            return async move {
                Err(anyhow::anyhow!("consensus worker terminated or panicked").into())
            }
            .boxed();
        }

        let span = tracing::Span::current();
        //let span = error_span!(parent: &span, "app", role = "consensus");
        let (tx, rx) = oneshot::channel();

        self.queue
            .send_item(Message {
                req,
                rsp_sender: tx,
                span,
            })
            .expect("called without `poll_ready`");

        async move {
            rx.await
                .map_err(|_| anyhow::anyhow!("consensus worker terminated or panicked").into())
        }
        .boxed()
    }
}
