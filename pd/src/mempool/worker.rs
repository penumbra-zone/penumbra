use anyhow::Result;

use penumbra_storage::{Snapshot, Storage};

use tokio::sync::{mpsc, watch};
use tracing::{instrument, Instrument};

use super::Message;
use crate::App;

/// When using ABCI, we can't control block proposal directly, so we could
/// potentially end up creating blocks with mutually incompatible transactions.
/// While we'd reject one of them during execution, it's nicer to try to filter
/// them out at the mempool stage. Currently, the way we do this is by having
/// the mempool worker maintain an ephemeral fork of the entire execution state,
/// and execute incoming transactions against the fork.  This prevents
/// conflicting transactions in the local mempool, since we'll update the fork,
/// then reject the second transaction against the forked state. When we learn a
/// new state has been committed, we discard and recreate the ephemeral fork.
///
/// After switching to ABCI++, we can eliminate this mechanism and just build
/// blocks we want.
pub struct Worker {
    queue: mpsc::Receiver<Message>,
    app: App,
    snapshot_rx: watch::Receiver<Snapshot>,
}

impl Worker {
    #[instrument(skip(storage, queue), name = "mempool::Worker::new")]
    pub async fn new(storage: Storage, queue: mpsc::Receiver<Message>) -> Result<Self> {
        let app = App::new(storage.latest_snapshot());
        let snapshot_rx = storage.subscribe();

        Ok(Self {
            queue,
            app,
            snapshot_rx,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                // Use a biased select to poll for height changes *before* polling for messages.
                biased;
                // Check whether the height has changed, which requires us to throw away our
                // ephemeral mempool state, and create a new one based on the new state.
                change = self.snapshot_rx.changed() => {
                    if let Ok(()) = change {
                        let snapshot = self.snapshot_rx.borrow().clone();
                        tracing::debug!(height = ?snapshot.version(), "resetting ephemeral mempool state");
                        self.app = App::new(snapshot);
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
                        let rsp = self.app.deliver_tx_bytes(tx_bytes.as_ref()).instrument(span).await.map(|_| ());
                        let _ = rsp_sender.send(rsp);
                    } else {
                        // The queue is closed, so we're done.
                        return Ok(());
                    }
                }
            }
        }
    }
}
