use penumbra_proto::{
    self as proto,
    chain::AssetInfo,
    thin_wallet::{
        thin_wallet_server::ThinWallet, Asset, AssetListRequest, AssetLookupRequest,
        TransactionByNoteRequest, TransactionDetail, ValidatorStatusRequest,
    },
};
use penumbra_stake::IdentityKey;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tracing::{instrument, Instrument, Span};

use crate::state;

#[tonic::async_trait]
impl ThinWallet for state::Reader {
    type AssetListStream = ReceiverStream<Result<Asset, Status>>;

    #[instrument(skip(self, request))]
    async fn transaction_by_note(
        &self,
        request: tonic::Request<TransactionByNoteRequest>,
    ) -> Result<tonic::Response<TransactionDetail>, Status> {
        self.check_chain_id(&request.get_ref().chain_id)?;

        tracing::debug!(cm = ?hex::encode(&request.get_ref().cm));
        let state = self.clone();
        let transaction = state
            .transaction_by_note(request.into_inner().cm)
            .await
            .map_err(|_| tonic::Status::not_found("transaction not found"))?;
        Ok(tonic::Response::new(transaction))
    }

    #[instrument(skip(self, request))]
    async fn asset_lookup(
        &self,
        request: tonic::Request<AssetLookupRequest>,
    ) -> Result<tonic::Response<AssetInfo>, Status> {
        self.check_chain_id(&request.get_ref().chain_id)?;

        let asset_id = penumbra_crypto::asset::Id::try_from(
            request
                .into_inner()
                .asset_id
                .ok_or_else(|| tonic::Status::invalid_argument("missing asset id"))?,
        )
        .map_err(|_| tonic::Status::not_found("invalid asset ID"))?;
        tracing::debug!(?asset_id);
        let state = self.clone();
        let asset = state
            .asset_lookup(asset_id)
            .await
            .map_err(|_| tonic::Status::not_found("asset not found"))?
            .ok_or(|| anyhow::anyhow!("asset not found"))
            .map_err(|_| tonic::Status::not_found("asset not found"))?;

        Ok(tonic::Response::new(asset))
    }

    #[instrument(skip(self, request))]
    async fn asset_list(
        &self,
        request: tonic::Request<AssetListRequest>,
    ) -> Result<tonic::Response<Self::AssetListStream>, Status> {
        self.check_chain_id(&request.get_ref().chain_id)?;

        tracing::debug!("processing request");
        let state = self.clone();

        let (tx, rx) = mpsc::channel(100);
        tokio::spawn(
            async move {
                let assets = state
                    .asset_list()
                    .await
                    .map_err(|_| tonic::Status::unavailable("database error"))
                    .unwrap();
                for asset in &assets[..] {
tracing::debug!(asset_id = ?hex::encode(&asset.asset_id), asset_denom = ?asset.asset_denom, "sending asset");
                    tx.send(Ok(asset.clone())).await.unwrap();
                }
            }
            .instrument(Span::current()),
        );

        Ok(tonic::Response::new(Self::AssetListStream::new(rx)))
    }

    #[instrument(skip(self, request))]
    async fn validator_status(
        &self,
        request: tonic::Request<ValidatorStatusRequest>,
    ) -> Result<tonic::Response<proto::stake::ValidatorStatus>, Status> {
        self.check_chain_id(&request.get_ref().chain_id)?;

        todo!()
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

        let rates = self
            .next_rate_data()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let rate = rates
            .get(&identity_key)
            .ok_or_else(|| tonic::Status::not_found("validator not found"))?
            .clone();

        Ok(tonic::Response::new(rate.into()))
    }
}
