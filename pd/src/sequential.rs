use std::{
    future::Future,
    task::{Context, Poll},
};

use tokio::sync::oneshot;

/// Allows executing futures in sequence, ensuring that each one is fully
/// resolved before beginning processing of the next one.
///
/// This allows a service to ensure that a request is processed before
/// processing any further requests.
#[derive(Debug)]
pub struct Sequencer {
    // it would be cleaner to use an Option, but we have to box the oneshot
    // future because it won't be Unpin and Service::poll_ready doesn't require
    // a pinned receiver, so tracking the waiting state in a separate bool allows
    // reallocating a new boxed future every time.
    waiting: bool,
    completion: tokio_util::sync::ReusableBoxFuture<Result<(), oneshot::error::RecvError>>,
}

impl Sequencer {
    /// Execute the given future on a new task.
    ///
    /// Spans attached to the future *returned* by `execute` do not propagate to
    /// the executed future, so the input `fut` should be `.instrument`ed if
    /// that's desired.
    ///
    /// This function must only be called after `self.poll_ready()` returns
    /// `Poll::Ready`.  After it is called, `self.poll_ready()` will not return
    /// `Poll::Ready` until the future completes.
    pub fn execute<O: Send + 'static>(
        &mut self,
        fut: impl Future<Output = O> + Send + 'static,
    ) -> impl Future<Output = O> + Send + 'static {
        assert!(!self.waiting);
        let (tx, rx) = oneshot::channel();
        self.waiting = true;
        self.completion.set(rx);

        async move {
            // Spawn a new task to ensure the future is driven to completion.
            // (depending on the future, it may not ever complete, but not for
            // lack of polling...)
            let output = tokio::spawn(fut).await.unwrap();
            // Signal completion after the future resolves.
            let _we_dont_care_if_the_sequencer_was_dropped = tx.send(());
            output
        }
    }

    pub fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if !self.waiting {
            return Poll::Ready(());
        }

        match self.completion.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => {
                self.waiting = false;
                Poll::Ready(())
            }
            Poll::Ready(Err(_)) => {
                tracing::error!("response future of sequentially-processed request was dropped before completion, likely a bug");
                self.waiting = false;
                Poll::Ready(())
            }
        }
    }
}

impl Default for Sequencer {
    fn default() -> Self {
        Self {
            completion: tokio_util::sync::ReusableBoxFuture::new(async { Ok(()) }),
            waiting: false,
        }
    }
}
