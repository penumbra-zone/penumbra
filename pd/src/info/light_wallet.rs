use std::pin::Pin;

use async_stream::try_stream;
use futures::stream::{StreamExt, TryStreamExt};
use penumbra_proto::{
    chain::{ChainParams, CompactBlock, KnownAssets},
    light_wallet::{
        light_wallet_server::LightWallet, AssetListRequest, ChainParamsRequest,
        CompactBlockRangeRequest, ValidatorInfoRequest,
    },
    stake::ValidatorInfo,
    Protobuf,
};
use tonic::Status;
use tracing::instrument;

// TODO(hdevalence): this still doesn't work, giving up for now
// We need to use the tracing-futures version of Instrument,
// because we want to instrument a Stream, and the Stream trait
// isn't in std, and the tracing::Instrument trait only works with
// (stable) std types.
// use tracing_futures::Instrument;

use crate::components::{app::View as _, shielded_pool::View as _, staking::View as _};
use crate::WriteOverlayExt;

struct WalletOverlay<T: WriteOverlayExt>(T);

#[tonic::async_trait]
impl<T: WriteOverlayExt> LightWallet for WalletOverlay<T> {
    type CompactBlockRangeStream =
        Pin<Box<dyn futures::Stream<Item = Result<CompactBlock, tonic::Status>> + Send>>;

    type ValidatorInfoStream =
        Pin<Box<dyn futures::Stream<Item = Result<ValidatorInfo, tonic::Status>> + Send>>;

    #[instrument(skip(self, request))]
    async fn chain_params(
        &self,
        request: tonic::Request<ChainParamsRequest>,
    ) -> Result<tonic::Response<ChainParams>, Status> {
        self.0.check_chain_id(&request.get_ref().chain_id).await?;

        let chain_params = self
            .0
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
        self.0.check_chain_id(&request.get_ref().chain_id).await?;

        let known_assets = self
            .0
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
        self.0.check_chain_id(&request.get_ref().chain_id).await?;

        let validators = self
            .0
            .validator_list()
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?;

        let _show_inactive = request.get_ref().show_inactive;
        let store = self.0.clone();
        let s = try_stream! {
            for validator in validators {
                let info = store.validator_info(&validator)
                    .await?
                    .expect("known validator must be present");
                // TODO: filter by show_inactive
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
        ),
    )]
    async fn compact_block_range(
        &self,
        request: tonic::Request<CompactBlockRangeRequest>,
    ) -> Result<tonic::Response<Self::CompactBlockRangeStream>, Status> {
        self.0.check_chain_id(&request.get_ref().chain_id).await?;

        let CompactBlockRangeRequest {
            start_height,
            end_height,
            ..
        } = request.into_inner();

        let current_height = self
            .0
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

        let store = self.0.clone();
        let block_range = try_stream! {
            // It's useful to record the end height since we adjusted it,
            // but the start height is already recorded in the span.
            tracing::info!(
                end_height,
                num_blocks = end_height.saturating_sub(start_height),
                "starting compact_block_range response"
            );
            for height in start_height..end_height {
                let block = store.compact_block(height)
                    .await?
                    .expect("compact block for in-range height must be present");
                yield block.to_proto();
            }
        };

        Ok(tonic::Response::new(
            block_range
                .map_err(|_: anyhow::Error| tonic::Status::unavailable("database error"))
                // TODO: how to instrument a Stream?
                //.instrument(Span::current())
                .boxed(),
        ))
    }
}
