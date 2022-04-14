use std::pin::Pin;

use futures::stream::StreamExt;
use penumbra_proto::{
    chain::{ChainParams, CompactBlock, KnownAssets},
    light_client::{
        light_protocol_server::LightProtocol, AssetListRequest, ChainParamsRequest,
        CompactBlockRangeRequest, ValidatorInfoRequest,
    },
    stake::ValidatorInfo,
};

use tonic::Status;
use tracing::instrument;

use crate::components::{app::View, staking::View as _};

struct WalletOverlay<T: View>(T);

#[tonic::async_trait]
impl<T> LightProtocol for WalletOverlay<T>
where
    T: View,
{
    type CompactBlockRangeStream =
        Pin<Box<dyn futures::Stream<Item = Result<CompactBlock, tonic::Status>> + Send>>;

    type ValidatorInfoStream =
        Pin<Box<dyn futures::Stream<Item = Result<ValidatorInfo, tonic::Status>> + Send>>;

    #[instrument(skip(self, request), fields())]
    async fn chain_params(
        &self,
        request: tonic::Request<ChainParamsRequest>,
    ) -> Result<tonic::Response<ChainParams>, Status> {
        self.0.check_chain_id(&request.get_ref().chain_id).await?;

        let genesis_configuration =
            self.0.genesis_configuration().await.map_err(|_| {
                tonic::Status::unavailable("error retrieving genesis configuration")
            })?;

        Ok(tonic::Response::new(
            genesis_configuration.chain_params.into(),
        ))
    }

    #[instrument(skip(self, request), fields(show_inactive = request.get_ref().show_inactive))]
    async fn validator_info(
        &self,
        request: tonic::Request<ValidatorInfoRequest>,
    ) -> Result<tonic::Response<Self::ValidatorInfoStream>, Status> {
        self.0.check_chain_id(&request.get_ref().chain_id).await?;

        let validator_info = self
            .validator_info(request)
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?;

        Ok(tonic::Response::new(
            futures::stream::iter(&validator_info.into_iter().map(|info| Ok(info.into()))).boxed(),
        ))
    }

    #[instrument(skip(self, request))]
    async fn asset_list(
        &self,
        request: tonic::Request<AssetListRequest>,
    ) -> Result<tonic::Response<KnownAssets>, Status> {
        self.0.check_chain_id(&request.get_ref().chain_id).await?;

        tracing::debug!("processing request");

        let assets = self
            .asset_list(request)
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))
            .unwrap()
            .into_inner()
            .assets;

        Ok(tonic::Response::new(KnownAssets { assets }))
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

        let current_height = self.0.height().value();

        // Treat end_height = 0 as end_height = current_height so that if the
        // end_height is unspecified in the proto, it will be treated as a
        // request to sync up to the current height.
        let end_height = if end_height == 0 {
            current_height
        } else {
            std::cmp::min(end_height, current_height)
        };

        // It's useful to record the end height since we adjusted it,
        // but the start height is already recorded in the span.
        tracing::info!(
            end_height,
            num_blocks = end_height.saturating_sub(start_height),
            "starting compact_block_range response"
        );

        let stream = self
            .0
            .compact_block_range(
                start_height.try_into().unwrap(),
                end_height.try_into().unwrap(),
            )
            .map_err(|e| tonic::Status::internal(e.to_string()));

        Ok(tonic::Response::new(stream.boxed()))
    }
}
