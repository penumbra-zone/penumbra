use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::FutureExt;
use tendermint::{
    abci::{ConsensusRequest, ConsensusResponse},
    block,
};
use tokio::sync::{mpsc, oneshot, watch};
use tokio_util::sync::PollSender;
use tower_abci::BoxError;

use super::{Message, Worker};
use crate::{state, RequestExt, Storage};

#[derive(Clone)]
pub struct Consensus {
    queue: PollSender<Message>,
}

impl Consensus {
    pub async fn new(
        state: state::Writer,
        storage: Storage,
    ) -> anyhow::Result<(Self, watch::Receiver<block::Height>)> {
        let (queue_tx, queue_rx) = mpsc::channel(10);
        let initial_height = match storage.latest_version().await? {
            Some(version) => version.try_into().unwrap(),
            _ => 0u32.into(),
        };
        let (height_tx, height_rx) = watch::channel(initial_height);

        tokio::spawn(
            Worker::new(state, storage, queue_rx, height_tx)
                .await?
                .run(),
        );

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
