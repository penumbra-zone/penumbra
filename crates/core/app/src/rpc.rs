use penumbra_chain::component::StateReadExt as _;
use penumbra_proto::core::app::v1alpha1::{
    query_service_server::QueryService, AppParametersRequest, AppParametersResponse,
    TransactionsByHeightRequest, TransactionsByHeightResponse,
};
use penumbra_storage::Storage;
use tonic::Status;
use tracing::instrument;

use crate::app::StateReadExt as _;

// TODO: Hide this and only expose a Router?
pub struct Server {
    storage: Storage,
}

impl Server {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl QueryService for Server {
    #[instrument(skip(self, request))]
    async fn transactions_by_height(
        &self,
        request: tonic::Request<TransactionsByHeightRequest>,
    ) -> Result<tonic::Response<TransactionsByHeightResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let request_inner = request.into_inner();
        let block_height = request_inner.block_height;

        let tx_response = state
            .transactions_by_height(block_height)
            .await
            .map_err(|e| tonic::Status::internal(format!("transaction response bad: {e}")))?;

        Ok(tonic::Response::new(tx_response))
    }

    #[instrument(skip(self, request))]
    async fn app_parameters(
        &self,
        request: tonic::Request<AppParametersRequest>,
    ) -> Result<tonic::Response<AppParametersResponse>, Status> {
        let state = self.storage.latest_snapshot();
        // We map the error here to avoid including `tonic` as a dependency
        // in the `chain` crate, to support its compilation to wasm.
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| {
                tonic::Status::unknown(format!(
                    "failed to validate chain id during app parameters lookup: {e}"
                ))
            })?;

        let app_parameters = state.get_app_params().await.map_err(|e| {
            tonic::Status::unavailable(format!("error getting app parameters: {e}"))
        })?;

        Ok(tonic::Response::new(AppParametersResponse {
            app_parameters: Some(app_parameters.into()),
        }))
    }
}
