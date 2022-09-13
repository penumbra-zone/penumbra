use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::FutureExt;
use penumbra_storage::Storage;
use tendermint::{
    abci::{ConsensusRequest, ConsensusResponse},
    block,
};
use tokio::sync::{mpsc, oneshot, watch};
use tokio_util::sync::PollSender;
use tower_abci::BoxError;
use tracing::error_span;

use super::{Message, Worker};
use crate::RequestExt;

#[derive(Clone)]
pub struct Consensus {
    queue: PollSender<Message>,
}

impl Consensus {
    pub async fn new(storage: Storage) -> anyhow::Result<(Self, watch::Receiver<block::Height>)> {
        let (queue_tx, queue_rx) = mpsc::channel(10);
        let initial_height = match storage.latest_version().await? {
            Some(version) => version.try_into().unwrap(),
            _ => 0u32.into(),
        };
        let (height_tx, height_rx) = watch::channel(initial_height);

        tokio::task::Builder::new()
            .name("consensus::Worker")
            .spawn(Worker::new(storage, queue_rx, height_tx).await?.run())
            .expect("failed to spawn consensus worker");

        Ok((
            Self {
                queue: PollSender::new(queue_tx),
            },
            height_rx,
        ))
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
        // Check if the worker has terminated. We do this again in `call`
        // because the worker may have terminated *after* `poll_ready` reserved
        // a send permit.
        if self.queue.is_closed() {
            return async move {
                Err(anyhow::anyhow!("consensus worker terminated or panicked").into())
            }
            .boxed();
        }

        let span = req.create_span();
        let span = error_span!(parent: &span, "app", role = "consensus");
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
