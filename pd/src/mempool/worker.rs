use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;

use penumbra_proto::Protobuf;
use penumbra_storage2::{StateNotification, Storage};
use penumbra_transaction::Transaction;

use tokio::sync::{mpsc, watch};
use tracing::{instrument, Instrument};

use super::Message;
use crate::App;

pub struct Worker {
    queue: mpsc::Receiver<Message>,
    storage: Storage,
    app: App,
    state_rx: watch::Receiver<StateNotification>,
}

impl Worker {
    #[instrument(skip(storage, queue), name = "mempool::Worker::new")]
    pub async fn new(storage: Storage, queue: mpsc::Receiver<Message>) -> Result<Self> {
        let app = App::new(storage.state());
        let state_rx = storage.subscribe();

        Ok(Self {
            queue,
            storage,
            app,
            state_rx,
        })
    }

    /// Currently, we perform all stateless and stateful checks sequentially in
    /// the mempool worker.  A possibly more performant design would be to only
    /// perform the stateful checks in the worker, and have a frontend service
    /// that performs the stateless checks.  However, this probably isn't
    /// important to do until we know that it's a bottleneck.
    async fn check_and_execute_tx(&mut self, tx_bytes: Bytes) -> Result<()> {
        // TODO: should this wrapper fn exist?
        let tx = Arc::new(Transaction::decode(tx_bytes.as_ref())?);
        self.app.deliver_tx(tx).await?;
        Ok(())
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                // Use a biased select to poll for height changes *before* polling for messages.
                biased;
                // Check whether the height has changed, which requires us to throw away our
                // ephemeral mempool state, and create a new one based on the new state.
                change = self.state_rx.changed() => {
                    if let Ok(()) = change {
                        let state = self.state_rx.borrow().into_state();
                        tracing::info!(height = ?state.version(), "resetting ephemeral mempool state");
                        self.app = App::new(state);
                    } else {
                        // TODO: what triggers this, now that the channel is owned by the
                        // shared Storage instance, rather than the consensus worker?
                        tracing::info!("state notification channel closed, shutting down");
                        // old: The consensus worker shut down, we should too.
                        return Ok(());
                    }
                }
                message = self.queue.recv() => {
                    if let Some(Message {
                        tx_bytes,
                        rsp_sender,
                        span,
                    }) = message {
                        let _ = rsp_sender.send(
                            self.check_and_execute_tx(tx_bytes)
                                .instrument(span)
                                .await
                        );
                    } else {
                        // The queue is closed, so we're done.
                        return Ok(());
                    }
                }
            }
        }
    }
}
