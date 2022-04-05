use penumbra_proto::{
    chain::{ChainParams, CompactBlock},
    light_wallet::{
        light_wallet_server::LightWallet, ChainParamsRequest, CompactBlockRangeRequest,
        ValidatorInfoRequest,
    },
    stake::ValidatorInfo,
};
use std::pin::Pin;

use tonic::Status;
use tracing::instrument;

use crate::components::{app::View as _, shielded_pool::View as _, staking::View as _};
use crate::WriteOverlayExt;

struct WalletOverlay<T: WriteOverlayExt>(T);

#[tonic::async_trait]
impl<T: 'static + WriteOverlayExt> LightWallet for WalletOverlay<T> {
    type CompactBlockRangeStream =
        Pin<Box<dyn futures::Stream<Item = Result<CompactBlock, tonic::Status>> + Send>>;

    type ValidatorInfoStream =
        Pin<Box<dyn futures::Stream<Item = Result<ValidatorInfo, tonic::Status>> + Send>>;

    #[instrument(skip(self, request))]
    async fn chain_params(
        &self,
        request: tonic::Request<ChainParamsRequest>,
    ) -> Result<tonic::Response<ChainParams>, Status> {
        let chain_params = self
            .0
            .get_chain_params()
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?;

        Ok(tonic::Response::new(chain_params.into()))
    }

    #[instrument(skip(self, request))]
    async fn validator_info(
        &self,
        request: tonic::Request<ValidatorInfoRequest>,
    ) -> Result<tonic::Response<Self::ValidatorInfoStream>, Status> {
        let info = self
            .0
            .get_validator_info(request)
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?;

        Ok(tonic::Response::new(info.into()))
    }

    #[instrument(skip(self, request))]
    async fn compact_block_range(
        &self,
        request: tonic::Request<CompactBlockRangeRequest>,
    ) -> Result<tonic::Response<Self::CompactBlockRangeStream>, Status> {
        let block_range = self
            .0
            .get_compact_block_range(request)
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?;

        Ok(tonic::Response::new(block_range.into()))
    }
}
