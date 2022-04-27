use anyhow::Result;
use bytes::Bytes;

use penumbra_proto::Protobuf;
use penumbra_transaction::Transaction;
use tendermint::block;
use tokio::sync::{mpsc, watch};
use tracing::Instrument;
use penumbra_storage::Storage;

use super::Message;
use crate::{App, Component};

pub struct Worker {
    queue: mpsc::Receiver<Message>,
    storage: Storage,
    app: App,
    height_rx: watch::Receiver<block::Height>,
}

impl Worker {
    pub async fn new(
        storage: Storage,
        queue: mpsc::Receiver<Message>,
        height_rx: watch::Receiver<block::Height>,
    ) -> Result<Self> {
        let app = App::new(storage.overlay().await?).await;

        Ok(Self {
            queue,
            storage,
            app,
            height_rx,
        })
    }

    /// Currently, we perform all stateless and stateful checks sequentially in
    /// the mempool worker.  A possibly more performant design would be to only
    /// perform the stateful checks in the worker, and have a frontend service
    /// that performs the stateless checks.  However, this probably isn't
    /// important to do until we know that it's a bottleneck.
    async fn check_and_execute_tx(&mut self, tx_bytes: Bytes) -> Result<()> {
        let tx = Transaction::decode(tx_bytes.as_ref())?;
        App::check_tx_stateless(&tx)?;
        self.app.check_tx_stateful(&tx).await?;
        self.app.execute_tx(&tx).await;
        Ok(())
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                // Use a biased select to poll for height changes *before* polling for messages.
                biased;
                // Check whether the height has changed, which requires us to throw away our
                // ephemeral mempool state, and create a new one based on the new state.
                change = self.height_rx.changed() => {
                    if let Ok(()) = change {
                        let height = self.height_rx.borrow().value();
                        tracing::info!(?height, "resetting ephemeral mempool state");
                        self.app = App::new(self.storage.overlay().await?).await;
                    } else {
                        tracing::info!("consensus worker shut down, shutting down mempool worker");
                        // The consensus worker shut down, we should too.
                        return Ok(());
                    }
                }
                message = self.queue.recv() => {
                    if let Some(Message {
                        tx_bytes,
                        rsp_sender,
                        span,
                    }) = message {
                        // ... and then execute it if it was valid.
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
