use std::pin::Pin;

use futures::stream::{StreamExt, TryStreamExt};
use penumbra_proto::{
    self as proto,
    chain::{AssetInfo, ChainParams},
    crypto::AssetId,
    light_wallet::{
        light_wallet_server::LightWallet, ChainParamsRequest, CompactBlock,
        CompactBlockRangeRequest, ValidatorInfoRequest,
    },
    stake::ValidatorInfo,
    thin_wallet::{
        thin_wallet_server::ThinWallet, Asset, AssetListRequest, TransactionByNoteRequest,
        TransactionDetail, ValidatorRateRequest,
    },
};
use penumbra_stake::IdentityKey;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tracing::{instrument, Instrument, Span};

use crate::state;

#[tonic::async_trait]
impl LightWallet for state::Reader {
    type CompactBlockRangeStream =
        Pin<Box<dyn futures::Stream<Item = Result<CompactBlock, tonic::Status>> + Send>>;

    type ValidatorInfoStream =
        Pin<Box<dyn futures::Stream<Item = Result<ValidatorInfo, tonic::Status>> + Send>>;

    #[instrument(skip(self, _request), fields())]
    async fn chain_params(
        &self,
        _request: tonic::Request<ChainParamsRequest>,
    ) -> Result<tonic::Response<ChainParams>, Status> {
        let genesis_configuration = self
            .genesis_configuration()
            .await
            .map_err(|_| tonic::Status::unavailable("error retrieving genesis configuration"))?;

        Ok(tonic::Response::new(ChainParams {
            chain_id: genesis_configuration.chain_params.chain_id,
            epoch_duration: genesis_configuration.chain_params.epoch_duration,
        }))
    }

    #[instrument(skip(self, request), fields(show_inactive = request.get_ref().show_inactive))]
    async fn validator_info(
        &self,
        request: tonic::Request<ValidatorInfoRequest>,
    ) -> Result<tonic::Response<Self::ValidatorInfoStream>, Status> {
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
        let CompactBlockRangeRequest {
            start_height,
            end_height,
        } = request.into_inner();

        let current_height = self
            .height()
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?
            .value();

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

#[tonic::async_trait]
impl ThinWallet for state::Reader {
    type AssetListStream = ReceiverStream<Result<Asset, Status>>;

    #[instrument(skip(self, request))]
    async fn transaction_by_note(
        &self,
        request: tonic::Request<TransactionByNoteRequest>,
    ) -> Result<tonic::Response<TransactionDetail>, Status> {
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
        request: tonic::Request<AssetId>,
    ) -> Result<tonic::Response<AssetInfo>, Status> {
        let asset_id = penumbra_crypto::asset::Id::try_from(request.into_inner())
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

    #[instrument(skip(self, _request))]
    async fn asset_list(
        &self,
        _request: tonic::Request<AssetListRequest>,
    ) -> Result<tonic::Response<Self::AssetListStream>, Status> {
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

    #[instrument(skip(self, _request))]
    async fn validator_status(
        &self,
        _request: tonic::Request<proto::stake::IdentityKey>,
    ) -> Result<tonic::Response<proto::stake::ValidatorStatus>, Status> {
        todo!()
    }

    #[instrument(skip(self, request))]
    async fn validator_rate(
        &self,
        request: tonic::Request<ValidatorRateRequest>,
    ) -> Result<tonic::Response<proto::stake::RateData>, Status> {
        let request = request.into_inner();
        let rates = self
            .rate_data(request.epoch_index)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let identity_key = IdentityKey::try_from(
            request
                .identity_key
                .ok_or_else(|| tonic::Status::invalid_argument("missing identity key"))?,
        )
        .map_err(|_| tonic::Status::invalid_argument("invalid identity key"))?;

        let rate = rates
            .into_iter()
            .find(|data| data.identity_key == identity_key)
            .ok_or_else(|| tonic::Status::not_found("validator not found"))?;

        Ok(tonic::Response::new(rate.into()))
    }
}
