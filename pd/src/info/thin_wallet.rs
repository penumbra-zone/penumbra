use penumbra_proto::{
    self as proto,
    chain::AssetInfo,
    thin_wallet::{
        thin_wallet_server::ThinWallet, Asset, AssetListRequest, AssetLookupRequest,
        TransactionByNoteRequest, TransactionDetail, ValidatorStatusRequest,
    },
};

use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tracing::instrument;

use crate::components::{app::View as _, shielded_pool::View as _, staking::View as _};
use crate::WriteOverlayExt;

struct WalletOverlay<T: WriteOverlayExt>(T);

#[tonic::async_trait]
impl<T: 'static + WriteOverlayExt> ThinWallet for WalletOverlay<T> {
    type AssetListStream = ReceiverStream<Result<Asset, Status>>;

    #[instrument(skip(self, request))]
    async fn transaction_by_note(
        &self,
        request: tonic::Request<TransactionByNoteRequest>,
    ) -> Result<tonic::Response<TransactionDetail>, Status> {
        Ok(self
            .0
            .get_transaction_by_note(request)
            .await
            .map_err(|_| tonic::Status::not_found("transaction not found"))?)
    }

    #[instrument(skip(self, request))]
    async fn asset_lookup(
        &self,
        request: tonic::Request<AssetLookupRequest>,
    ) -> Result<tonic::Response<AssetInfo>, Status> {
        Ok(self
            .0
            .get_asset_info(request)
            .await
            .map_err(|_| tonic::Status::not_found("asset not found"))?)
    }

    #[instrument(skip(self, request))]
    async fn asset_list(
        &self,
        request: tonic::Request<AssetListRequest>,
    ) -> Result<tonic::Response<Self::AssetListStream>, Status> {
        Ok(self
            .0
            .get_asset_list(request)
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?)
    }

    #[instrument(skip(self, request))]
    async fn validator_status(
        &self,
        request: tonic::Request<ValidatorStatusRequest>,
    ) -> Result<tonic::Response<proto::stake::ValidatorStatus>, Status> {
        let x = self
            .0
            .get_validator_status(request)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .unwrap();

        Ok(tonic::Response::new(x.into()))
    }

    #[instrument(skip(self, request))]
    async fn next_validator_rate(
        &self,
        request: tonic::Request<proto::stake::IdentityKey>,
    ) -> Result<tonic::Response<proto::stake::RateData>, Status> {
        let identity_key = request
            .into_inner()
            .try_into()
            .map_err(|_| tonic::Status::invalid_argument("invalid identity key"))?;

        let rate_data = self
            .0
            .next_validator_rate(&identity_key)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .unwrap();

        Ok(tonic::Response::new(rate_data.into()))
    }
}
