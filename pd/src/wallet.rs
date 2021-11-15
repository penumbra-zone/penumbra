use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

use penumbra_proto::wallet::{
    wallet_server::Wallet, CompactBlock, CompactBlockRangeRequest, TransactionByNoteRequest,
    TransactionDetail,
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
        _request: tonic::Request<TransactionByNoteRequest>,
    ) -> Result<tonic::Response<TransactionDetail>, Status> {
        Err(tonic::Status::unimplemented(
            "how should this relate to tendermint rpc?",
        ))
    }
}
