use std::pin::Pin;

use anyhow::bail;
use cnidarium::Storage;
use futures::{StreamExt, TryFutureExt, TryStreamExt};
use penumbra_sdk_proto::core::component::compact_block::v1::{
    query_service_server::QueryService, CompactBlock, CompactBlockRangeRequest,
    CompactBlockRangeResponse, CompactBlockRequest, CompactBlockResponse,
};
use penumbra_sdk_sct::component::clock::EpochRead;
use tokio::sync::mpsc;
use tonic::Status;
use tracing::{instrument, Instrument};

use super::{metrics, StateReadExt};

// TODO: Hide this and only expose a Router?
pub struct Server {
    storage: Storage,
}

impl Server {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl QueryService for Server {
    type CompactBlockRangeStream = Pin<
        Box<dyn futures::Stream<Item = Result<CompactBlockRangeResponse, tonic::Status>> + Send>,
    >;

    async fn compact_block(
        &self,
        request: tonic::Request<CompactBlockRequest>,
    ) -> Result<tonic::Response<CompactBlockResponse>, Status> {
        let snapshot = self.storage.latest_snapshot();

        let height = request.get_ref().height;
        let compact_block = snapshot
            .compact_block(height)
            .await
            .map_err(|e| tonic::Status::internal(format!("error fetching block: {e:#}")))?
            .ok_or_else(|| tonic::Status::not_found(format!("compact block {height} not found")))?;

        Ok(tonic::Response::new(CompactBlockResponse {
            compact_block: Some(compact_block.into()),
        }))
    }

    #[instrument(
        skip(self, request),
        fields(
            start_height = request.get_ref().start_height,
            end_height = request.get_ref().end_height,
            keep_alive = request.get_ref().keep_alive,
        ),
    )]
    async fn compact_block_range(
        &self,
        request: tonic::Request<CompactBlockRangeRequest>,
    ) -> Result<tonic::Response<Self::CompactBlockRangeStream>, Status> {
        let snapshot = self.storage.latest_snapshot();
        // TODO(erwan): re-enable chain id checks
        // snapshot
        //     .check_chain_id(&request.get_ref().chain_id)
        //     .await
        //     .map_err(|e| {
        //         tonic::Status::unknown(format!(
        //             "failed to validate chain id during compact_block_range request: {e}"
        //         ))
        //     })?;

        let CompactBlockRangeRequest {
            start_height,
            end_height,
            keep_alive,
            ..
        } = request.into_inner();

        let current_height = snapshot
            .get_block_height()
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error getting block height: {e}")))?;

        // Perform housekeeping, so long-lived connections don't cause pd to leak memory.
        std::mem::drop(snapshot);

        // Treat end_height = 0 as end_height = current_height so that if the
        // end_height is unspecified in the proto, it will be treated as a
        // request to sync up to the current height.
        let end_height = if end_height == 0 {
            current_height
        } else {
            std::cmp::min(end_height, current_height)
        };

        // Clone these, so we can keep copies in the worker task we spawn
        // to handle this request.
        let storage = self.storage.clone();
        let mut rx_state_snapshot = self.storage.subscribe();

        let (tx_blocks, rx_blocks) = mpsc::channel(10);
        let tx_blocks_err = tx_blocks.clone();
        // Wrap the block sender in a guard that ensures we only send the expected next block
        let mut tx_blocks = BlockSender {
            next_height: start_height,
            inner: tx_blocks,
        };
        tokio::spawn(
            async move {
                let _guard = CompactBlockConnectionCounter::new();

                // Phase 1: Catch up from the start height.
                tracing::debug!(
                    ?end_height,
                    "catching up from start height to current end height"
                );

                // We rely on a range query to fetch compact blocks in order and
                // pipe them to the client sync stream.
                let storage2 = storage.clone();
                let latest_snapshot = storage2.latest_snapshot();
                let mut cb_stream = latest_snapshot.stream_compact_block(start_height);

                while let Some(res_compact_block) = cb_stream.next().await {
                    let compact_block = match res_compact_block {
                        Ok(compact_block) => compact_block,
                        Err(e) => {
                            bail!("error streaming compact blocks: {e}")
                        }
                    };
                    if compact_block.height > end_height {
                        break;
                    }

                    // Tracked in #2908: we previously added a timeout on `send` targeting
                    // buffered streams staying full for too long. However, in at least a few
                    // "regular usage" instances we observed client streams stopping too eagerly.
                    // In #2932, it was established that the timeout had to be at least 10s to
                    // accommodate those usecases.
                    //
                    // Although we cannot exclude that clients actually did not poll the stream for
                    // more than `9s`, this seems unlikely. We are removing the timeout mechanism
                    // altogether for now. This might negatively impact memory usage under load.
                    // Future iterations of this work should start by moving block serialization
                    // outside of the `send_op` future, and investigate if long blocking sends can
                    // happen for benign reasons (i.e not caused by the client).
                    tx_blocks.send(compact_block).await?;
                    metrics::counter!(metrics::COMPACT_BLOCK_RANGE_SERVED_TOTAL).increment(1);
                }

                // If the client didn't request a keep-alive, we're done.
                if !keep_alive {
                    // Explicitly annotate the error type, so we can bubble up errors...
                    return Ok::<(), anyhow::Error>(());
                }

                // Before we can stream new compact blocks as they're created,
                // catch up on any blocks that have been created while catching up.
                let snapshot = rx_state_snapshot.borrow_and_update().clone();
                let cur_height = snapshot.version();
                tracing::debug!(
                    cur_height,
                    "finished request, client requested keep-alive, continuing to stream blocks"
                );

                // We want to send all blocks *after* end_height (which we already sent)
                // up to and including cur_height (which we won't send in the loop below).
                // This range could be empty.
                for height in (end_height + 1)..=cur_height {
                    tracing::debug!(?height, "sending block in phase 2 catch-up");
                    let block = snapshot
                        .compact_block(height)
                        .await
                        .expect("no error fetching block")
                        .expect("compact block for in-range height must be present");
                    tx_blocks.send(block).await?;
                    metrics::counter!(metrics::COMPACT_BLOCK_RANGE_SERVED_TOTAL).increment(1);
                }

                // Ensure that we don't hold a reference to the snapshot indefinitely
                // while we hold open the connection to notify the client of new blocks.
                std::mem::drop(snapshot);

                // Phase 2: wait on the height notifier and stream blocks as
                // they're created.
                //
                // Because we used borrow_and_update above, we know this will
                // wait for the *next* block to be created before firing.
                loop {
                    rx_state_snapshot
                        .changed()
                        .await
                        .expect("channel should be open");
                    let snapshot = rx_state_snapshot.borrow().clone();
                    let height = snapshot.version();
                    tracing::debug!(?height, "notifying client of new block");
                    let block = snapshot
                        .compact_block(height)
                        .await
                        .map_err(|e| tonic::Status::internal(e.to_string()))?
                        .expect("compact block for in-range height must be present");
                    tx_blocks.send(block).await?;
                    metrics::counter!(metrics::COMPACT_BLOCK_RANGE_SERVED_TOTAL).increment(1);
                }
            }
            .map_err(|e| async move {
                // ... into something that can convert them into a tonic error
                // and stuff it into a second copy of the response channel
                // to notify the client before the task exits.
                let _ = tx_blocks_err
                    .send(Err(tonic::Status::internal(e.to_string())))
                    .await;
            })
            .instrument(tracing::Span::current()),
        );

        // TODO: eventually, we may want to register joinhandles or something
        // and be able to track how many open connections we have, drop them to
        // manage load, etc.
        //
        // for now, assume that we can do c10k or whatever and don't worry about it.
        Ok(tonic::Response::new(
            tokio_stream::wrappers::ReceiverStream::new(rx_blocks)
                .map_ok(|block| CompactBlockRangeResponse {
                    compact_block: Some(block),
                })
                .boxed(),
        ))
    }
}

/// RAII guard used to increment and decrement an active connection counter.
///
/// This ensures we appropriately decrement the counter when the guard goes out of scope.
struct CompactBlockConnectionCounter {}

impl CompactBlockConnectionCounter {
    pub fn new() -> Self {
        metrics::gauge!(metrics::COMPACT_BLOCK_RANGE_ACTIVE_CONNECTIONS).increment(1.0);
        CompactBlockConnectionCounter {}
    }
}

impl Drop for CompactBlockConnectionCounter {
    fn drop(&mut self) {
        metrics::gauge!(metrics::COMPACT_BLOCK_RANGE_ACTIVE_CONNECTIONS).decrement(1.0);
    }
}

/// Stateful wrapper for a mpsc that tracks the outbound height
struct BlockSender {
    next_height: u64,
    inner: mpsc::Sender<Result<CompactBlock, tonic::Status>>,
}

impl BlockSender {
    async fn send(&mut self, block: CompactBlock) -> anyhow::Result<()> {
        if block.height != self.next_height {
            bail!(
                "block height mismatch while sending: expected {}, got {}",
                self.next_height,
                block.height
            );
        }
        self.inner.send(Ok(block)).await?;
        self.next_height += 1;
        Ok(())
    }
}
