use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

use penumbra_proto::wallet::{
    wallet_server::Wallet, Asset, AssetListRequest, AssetLookupRequest, CompactBlock,
    CompactBlockRangeRequest, TransactionByNoteRequest, TransactionDetail,
};

use crate::State;

/// The Penumbra wallet service.
pub struct WalletApp {
    state: State,
}

impl WalletApp {
    pub fn new(state: State) -> WalletApp {
        WalletApp { state }
    }
}

#[tonic::async_trait]
impl Wallet for WalletApp {
    type CompactBlockRangeStream = ReceiverStream<Result<CompactBlock, Status>>;
    type AssetListStream = ReceiverStream<Result<Asset, Status>>;

    async fn compact_block_range(
        &self,
        request: tonic::Request<CompactBlockRangeRequest>,
    ) -> Result<tonic::Response<Self::CompactBlockRangeStream>, Status> {
        let CompactBlockRangeRequest {
            start_height,
            end_height,
        } = request.into_inner();

        if end_height < start_height {
            return Err(tonic::Status::failed_precondition(
                "end height must be greater than start height",
            ));
        }

        let (tx, rx) = mpsc::channel(100);

        let state = self.state.clone();
        tokio::spawn(async move {
            for height in start_height..=end_height {
                let block = state.compact_block(height.into()).await;
                tracing::info!("sending block response: {:?}", block);
                tx.send(block.map_err(|_| tonic::Status::unavailable("database error")))
                    .await
                    .unwrap();
            }
        });

        Ok(tonic::Response::new(Self::CompactBlockRangeStream::new(rx)))
    }

    async fn transaction_by_note(
        &self,
        request: tonic::Request<TransactionByNoteRequest>,
    ) -> Result<tonic::Response<TransactionDetail>, Status> {
        let state = self.state.clone();
        let transaction = state
            .transaction_by_note(request.into_inner().cm)
            .await
            .map_err(|_| tonic::Status::not_found("transaction not found"))?;
        Ok(tonic::Response::new(transaction))
    }

    async fn asset_lookup(
        &self,
        request: tonic::Request<AssetLookupRequest>,
    ) -> Result<tonic::Response<Asset>, Status> {
        let state = self.state.clone();
        let asset = state
            .asset_lookup(request.into_inner().asset_id)
            .await
            .map_err(|_| tonic::Status::not_found("asset not found"))?;
        Ok(tonic::Response::new(asset))
    }

    async fn asset_list(
        &self,
        _request: tonic::Request<AssetListRequest>,
    ) -> Result<tonic::Response<Self::AssetListStream>, Status> {
        let state = self.state.clone();

        let (tx, rx) = mpsc::channel(100);
        tokio::spawn(async move {
            let assets = state
                .asset_list()
                .await
                .map_err(|_| tonic::Status::unavailable("database error"))
                .unwrap();
            for asset in &assets[..] {
                tracing::info!("sending asset list response: {:?}", asset);
                tx.send(Ok(asset.clone())).await.unwrap();
            }
        });

        Ok(tonic::Response::new(Self::AssetListStream::new(rx)))
    }
}
