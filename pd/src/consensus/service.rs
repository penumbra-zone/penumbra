use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{ready, FutureExt};
use tendermint::abci::{ConsensusRequest, ConsensusResponse};
use tokio::sync::{
    mpsc::{self, error::SendError, OwnedPermit},
    oneshot,
};
use tokio_util::sync::ReusableBoxFuture;
use tower_abci::BoxError;

use super::{Message, Worker};
use crate::{state, RequestExt};

enum State {
    NoPermit,
    Waiting,
    Permit(OwnedPermit<Message>),
}

pub struct Consensus {
    queue: mpsc::Sender<Message>,
    future: ReusableBoxFuture<Result<OwnedPermit<Message>, SendError<()>>>,
    state: State,
}

impl Consensus {
    pub async fn new(state: state::Writer) -> anyhow::Result<Self> {
        let (queue_tx, queue_rx) = mpsc::channel(10);

        tokio::spawn(Worker::new(state, queue_rx).await?.run());

        Ok(Self {
            queue: queue_tx,
            state: State::NoPermit,
            future: ReusableBoxFuture::new(async { unreachable!() }),
        })
    }
}

impl Clone for Consensus {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            state: State::NoPermit,
            future: ReusableBoxFuture::new(async { unreachable!() }),
        }
    }
}

impl tower::Service<ConsensusRequest> for Consensus {
    type Response = ConsensusResponse;
    type Error = BoxError;
    type Future =
        Pin<Box<dyn Future<Output = Result<ConsensusResponse, BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.state {
            State::Permit(_) => Poll::Ready(Ok(())),
            State::NoPermit => {
                self.future.set(self.queue.clone().reserve_owned());
                self.state = State::Waiting;
                self.poll_ready(cx)
            }
            State::Waiting => {
                let permit = ready!(self.future.poll(cx))?;
                self.state = State::Permit(permit);
                Poll::Ready(Ok(()))
            }
        }
    }

    fn call(&mut self, req: ConsensusRequest) -> Self::Future {
        let permit = if let State::Permit(p) = std::mem::replace(&mut self.state, State::NoPermit) {
            p
        } else {
            panic!("called without poll_ready");
        };

        let span = req.create_span();
        let (tx, rx) = oneshot::channel();

        permit.send(Message {
            req,
            rsp_sender: tx,
            span,
        });

        async move { Ok(rx.await.expect("worker error??")) }.boxed()
    }
}
