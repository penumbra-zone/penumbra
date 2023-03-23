use std::pin::Pin;

use async_stream::try_stream;
use futures::{
    stream::{StreamExt, TryStreamExt},
    TryFutureExt,
};
use penumbra_chain::StateReadExt as _;
use penumbra_component::compactblock::view::StateReadExt as _;
use penumbra_component::shielded_pool::SupplyRead as _;
use penumbra_component::stake::{validator, StateReadExt as _};
use penumbra_proto::{
    client::v1alpha1::{
        oblivious_query_service_server::ObliviousQueryService, AssetListRequest, AssetListResponse,
        ChainParametersRequest, ChainParametersResponse, CompactBlockRangeRequest,
        CompactBlockRangeResponse, EpochRequest, EpochResponse, ValidatorInfoRequest,
        ValidatorInfoResponse,
    },
    DomainType,
};
use tokio::sync::mpsc;
use tonic::Status;
use tracing::{instrument, Instrument};

use crate::metrics;

/// RAII guard used to increment and decrement an active connection counter.
///
/// This ensures we appropriately decrement the counter when the guard goes out of scope.
struct CompactBlockConnectionCounter {}

impl CompactBlockConnectionCounter {
    pub fn new() -> Self {
        metrics::increment_gauge!(
            metrics::CLIENT_OBLIVIOUS_COMPACT_BLOCK_ACTIVE_CONNECTIONS,
            1.0
        );
        CompactBlockConnectionCounter {}
    }
}

impl Drop for CompactBlockConnectionCounter {
    fn drop(&mut self) {
        metrics::decrement_gauge!(
            metrics::CLIENT_OBLIVIOUS_COMPACT_BLOCK_ACTIVE_CONNECTIONS,
            1.0
        );
    }
}

use super::Info;

#[tonic::async_trait]
impl ObliviousQueryService for Info {
    type CompactBlockRangeStream = Pin<
        Box<dyn futures::Stream<Item = Result<CompactBlockRangeResponse, tonic::Status>> + Send>,
    >;

    type ValidatorInfoStream =
        Pin<Box<dyn futures::Stream<Item = Result<ValidatorInfoResponse, tonic::Status>> + Send>>;

    #[instrument(skip(self, request))]
    async fn chain_parameters(
        &self,
        request: tonic::Request<ChainParametersRequest>,
    ) -> Result<tonic::Response<ChainParametersResponse>, Status> {
        let state = self.storage.latest_snapshot();
        // We map the error here to avoid including `tonic` as a dependency
        // in the `chain` crate, to support its compilation to wasm.
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;

        let chain_params = state.get_chain_params().await.map_err(|e| {
            tonic::Status::unavailable(format!("error getting chain parameters: {e}"))
        })?;

        Ok(tonic::Response::new(ChainParametersResponse {
            chain_parameters: Some(chain_params.into()),
        }))
    }

    #[instrument(skip(self, request))]
    async fn asset_list(
        &self,
        request: tonic::Request<AssetListRequest>,
    ) -> Result<tonic::Response<AssetListResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;

        let known_assets = state
            .known_assets()
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error getting known assets: {e}")))?;
        Ok(tonic::Response::new(AssetListResponse {
            asset_list: Some(known_assets.into()),
        }))
    }

    #[instrument(skip(self, request))]
    async fn epoch(
        &self,
        request: tonic::Request<EpochRequest>,
    ) -> Result<tonic::Response<EpochResponse>, Status> {
        let state = self.storage.latest_snapshot();

        let epoch = state
            .epoch_by_height(request.get_ref().height)
            .await
            .map_err(|e| tonic::Status::unknown(format!("could not get epoch for height: {e}")))?;

        Ok(tonic::Response::new(EpochResponse {
            epoch: Some(epoch.into()),
        }))
    }

    #[instrument(skip(self, request), fields(show_inactive = request.get_ref().show_inactive))]
    async fn validator_info(
        &self,
        request: tonic::Request<ValidatorInfoRequest>,
    ) -> Result<tonic::Response<Self::ValidatorInfoStream>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;

        let validators = state
            .validator_list()
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error listing validators: {e}")))?;

        let show_inactive = request.get_ref().show_inactive;
        let s = try_stream! {
            for v in validators {
                let info = state.validator_info(&v.identity_key)
                    .await?
                    .expect("known validator must be present");
                // Slashed and inactive validators are not shown by default.
                if !show_inactive && info.status.state != validator::State::Active {
                    continue;
                }
                yield info.to_proto();
            }
        };

        Ok(tonic::Response::new(
            s.map_ok(|info| ValidatorInfoResponse {
                validator_info: Some(info),
            })
            .map_err(|e: anyhow::Error| {
                tonic::Status::unavailable(format!("error getting validator info: {e}"))
            })
            // TODO: how do we instrument a Stream
            //.instrument(Span::current())
            .boxed(),
        ))
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
        snapshot
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;

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
        let mut state_rx = self.storage.subscribe();

        let (tx, rx) = mpsc::channel(10);
        let txerr = tx.clone();
        tokio::spawn(
            async move {
                let _guard = CompactBlockConnectionCounter::new();

                // Phase 1: Catch up from the start height.
                tracing::debug!(
                    ?end_height,
                    "catching up from start height to current end height"
                );

                // We need to send block responses in order, but fetching the
                // compact block involves disk I/O, so we want to look ahead and
                // start fetching compact blocks, rather than waiting for each
                // state query to complete sequentially.
                //
                // To do this, we spawn a task that runs ahead and queues block
                // fetches from the state.  Each block fetch is also spawned as
                // a new task, so they execute independently, and those tasks'
                // JoinHandles are sent back to this task using a bounded
                // channel.  The channel bound prevents the queueing task from
                // running too far ahead.
                let (block_fetch_tx, mut block_fetch_rx) = mpsc::channel(8);

                let storage2 = storage.clone();
                tokio::spawn(async move {
                    for height in start_height..=end_height {
                        let state3 = storage2.latest_snapshot();
                        let _ = block_fetch_tx
                            .send(tokio::spawn(
                                async move { state3.compact_block(height).await },
                            ))
                            .await;
                    }
                });

                while let Some(block_fetch) = block_fetch_rx.recv().await {
                    let block = block_fetch
                        .await??
                        .expect("compact block for in-range height must be present");
                    tx.send(Ok(block.to_proto())).await?;
                    metrics::increment_counter!(
                        metrics::CLIENT_OBLIVIOUS_COMPACT_BLOCK_SERVED_TOTAL
                    );
                }

                // If the client didn't request a keep-alive, we're done.
                if !keep_alive {
                    // Explicitly annotate the error type, so we can bubble up errors...
                    return Ok::<(), anyhow::Error>(());
                }

                // Before we can stream new compact blocks as they're created,
                // catch up on any blocks that have been created while catching up.
                let snapshot = state_rx.borrow_and_update().clone();
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
                        .await?
                        .expect("compact block for in-range height must be present");
                    tx.send(Ok(block.to_proto())).await?;
                    metrics::increment_counter!(
                        metrics::CLIENT_OBLIVIOUS_COMPACT_BLOCK_SERVED_TOTAL
                    );
                }

                // Phase 2: wait on the height notifier and stream blocks as
                // they're created.
                //
                // Because we used borrow_and_update above, we know this will
                // wait for the *next* block to be created before firing.
                loop {
                    state_rx.changed().await?;
                    let snapshot = state_rx.borrow().clone();
                    let height = snapshot.version();
                    tracing::debug!(?height, "notifying client of new block");
                    let block = snapshot
                        .compact_block(height)
                        .await?
                        .expect("compact block for in-range height must be present");
                    tx.send(Ok(block.to_proto())).await?;
                    metrics::increment_counter!(
                        metrics::CLIENT_OBLIVIOUS_COMPACT_BLOCK_SERVED_TOTAL
                    );
                }
            }
            .map_err(|e| async move {
                // ... into something that can convert them into a tonic error
                // and stuff it into a second copy of the response channel
                // to notify the client before the task exits.
                let _ = txerr
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
            tokio_stream::wrappers::ReceiverStream::new(rx)
                .map_ok(|block| CompactBlockRangeResponse {
                    compact_block: Some(block),
                })
                .boxed(),
        ))
    }
}
