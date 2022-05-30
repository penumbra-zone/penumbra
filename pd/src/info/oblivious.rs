use std::pin::Pin;

use async_stream::try_stream;
use futures::{
    stream::{StreamExt, TryStreamExt},
    TryFutureExt,
};
use penumbra_chain::View as _;
use penumbra_proto::{
    chain::{ChainParams, CompactBlock, KnownAssets},
    client::oblivious::{
        oblivious_query_server::ObliviousQuery, AssetListRequest, ChainParamsRequest,
        CompactBlockRangeRequest, ValidatorInfoRequest,
    },
    stake::ValidatorInfo,
    Protobuf,
};
use penumbra_shielded_pool::View as _;
use penumbra_stake::{component::View as _, validator};
use tokio::sync::mpsc;
use tonic::Status;
use tracing::{instrument, Instrument};

// TODO(hdevalence): this still doesn't work, giving up for now
// We need to use the tracing-futures version of Instrument,
// because we want to instrument a Stream, and the Stream trait
// isn't in std, and the tracing::Instrument trait only works with
// (stable) std types.
// use tracing_futures::Instrument;

use super::Info;

#[tonic::async_trait]
impl ObliviousQuery for Info {
    type CompactBlockRangeStream =
        Pin<Box<dyn futures::Stream<Item = Result<CompactBlock, tonic::Status>> + Send>>;

    type ValidatorInfoStream =
        Pin<Box<dyn futures::Stream<Item = Result<ValidatorInfo, tonic::Status>> + Send>>;

    #[instrument(skip(self, request))]
    async fn chain_params(
        &self,
        request: tonic::Request<ChainParamsRequest>,
    ) -> Result<tonic::Response<ChainParams>, Status> {
        let state = self.state_tonic().await?;
        state.check_chain_id(&request.get_ref().chain_id).await?;

        let chain_params = state
            .get_chain_params()
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?;

        Ok(tonic::Response::new(chain_params.into()))
    }

    #[instrument(skip(self, request))]
    async fn asset_list(
        &self,
        request: tonic::Request<AssetListRequest>,
    ) -> Result<tonic::Response<KnownAssets>, Status> {
        let state = self.state_tonic().await?;
        state.check_chain_id(&request.get_ref().chain_id).await?;

        let known_assets = state
            .known_assets()
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?;
        Ok(tonic::Response::new(known_assets.into()))
    }

    #[instrument(skip(self, request), fields(show_inactive = request.get_ref().show_inactive))]
    async fn validator_info(
        &self,
        request: tonic::Request<ValidatorInfoRequest>,
    ) -> Result<tonic::Response<Self::ValidatorInfoStream>, Status> {
        let state = self.state_tonic().await?;
        state.check_chain_id(&request.get_ref().chain_id).await?;

        let validators = state
            .validator_list()
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?;

        let show_inactive = request.get_ref().show_inactive;
        let s = try_stream! {
            for identity_key in validators {
                let info = state.validator_info(&identity_key)
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
            s.map_err(|_: anyhow::Error| tonic::Status::unavailable("database error"))
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
        let state = self.state_tonic().await?;
        state.check_chain_id(&request.get_ref().chain_id).await?;

        let CompactBlockRangeRequest {
            start_height,
            end_height,
            keep_alive,
            ..
        } = request.into_inner();

        let current_height = state
            .get_block_height()
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?;

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
        let mut height_rx = self.height_rx.clone();

        let (tx, rx) = mpsc::channel(10);
        let txerr = tx.clone();
        tokio::spawn(
            async move {
                // Phase 1: Catch up from the start height.
                tracing::debug!(
                    ?end_height,
                    "catching up from start height to current end height"
                );
                for height in start_height..=end_height {
                    let block = state
                        .compact_block(height)
                        .await?
                        .expect("compact block for in-range height must be present");
                    tx.send(Ok(block.to_proto())).await?;
                }

                // If the client didn't request a keep-alive, we're done.
                if !keep_alive {
                    // Explicitly annotate the error type, so we can bubble up errors...
                    return Ok::<(), anyhow::Error>(());
                }

                // Before we can stream new compact blocks as they're created,
                // catch up on any blocks that have been created while catching up.
                let cur_height = height_rx.borrow_and_update().value();
                let state = storage.state().await?;
                tracing::debug!(
                    cur_height,
                    "finished request, client requested keep-alive, continuing to stream blocks"
                );

                // We want to send all blocks *after* end_height (which we already sent)
                // up to and including cur_height (which we won't send in the loop below).
                // This range could be empty.
                for height in (end_height + 1)..=cur_height {
                    tracing::debug!(?height, "sending block in phase 2 catch-up");
                    let block = state
                        .compact_block(height)
                        .await?
                        .expect("compact block for in-range height must be present");
                    tx.send(Ok(block.to_proto())).await?;
                }

                // Phase 2: wait on the height notifier and stream blocks as
                // they're created.
                //
                // Because we used borrow_and_update above, we know this will
                // wait for the *next* block to be created before firing.
                loop {
                    height_rx.changed().await?;
                    let height = height_rx.borrow().value();
                    tracing::debug!(?height, "notifying client of new block");
                    let state = storage.state().await?;
                    let block = state
                        .compact_block(height)
                        .await?
                        .expect("compact block for in-range height must be present");
                    tx.send(Ok(block.to_proto())).await?;
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
            tokio_stream::wrappers::ReceiverStream::new(rx).boxed(),
        ))
    }
}
