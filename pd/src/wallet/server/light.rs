use std::pin::Pin;

use futures::stream::{StreamExt, TryStreamExt};
use penumbra_proto::{
    chain::ChainParams,
    light_wallet::{
        light_wallet_server::LightWallet, ChainParamsRequest, CompactBlock,
        CompactBlockRangeRequest, ValidatorInfoRequest,
    },
    stake::ValidatorInfo,
};
use tonic::Status;
use tracing::instrument;

use crate::state;

#[tonic::async_trait]
impl LightWallet for state::Reader {
    type CompactBlockRangeStream =
        Pin<Box<dyn futures::Stream<Item = Result<CompactBlock, tonic::Status>> + Send>>;

    type ValidatorInfoStream =
        Pin<Box<dyn futures::Stream<Item = Result<ValidatorInfo, tonic::Status>> + Send>>;

    #[instrument(skip(self, request), fields())]
    async fn chain_params(
        &self,
        request: tonic::Request<ChainParamsRequest>,
    ) -> Result<tonic::Response<ChainParams>, Status> {
        self.check_chain_id(&request.get_ref().chain_id)?;

        let genesis_configuration = self
            .genesis_configuration()
            .await
            .map_err(|_| tonic::Status::unavailable("error retrieving genesis configuration"))?;

        Ok(tonic::Response::new(
            genesis_configuration.chain_params.into(),
        ))
    }

    #[instrument(skip(self, request), fields(show_inactive = request.get_ref().show_inactive))]
    async fn validator_info(
        &self,
        request: tonic::Request<ValidatorInfoRequest>,
    ) -> Result<tonic::Response<Self::ValidatorInfoStream>, Status> {
        self.check_chain_id(&request.get_ref().chain_id)?;

        let validator_info = self
            .validator_info(request.into_inner().show_inactive)
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?;

        Ok(tonic::Response::new(
            futures::stream::iter(validator_info.into_iter().map(|info| Ok(info.into()))).boxed(),
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
        self.check_chain_id(&request.get_ref().chain_id)?;

        let CompactBlockRangeRequest {
            start_height,
            end_height,
            ..
        } = request.into_inner();

        let current_height = self.height().value();

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
            .compact_blocks(
                start_height.try_into().unwrap(),
                end_height.try_into().unwrap(),
            )
            .map_err(|e| tonic::Status::internal(e.to_string()));

        Ok(tonic::Response::new(stream.boxed()))
    }
}
