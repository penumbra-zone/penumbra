use {
    crate::app::StateReadExt as _,
    cnidarium::Storage,
    penumbra_sdk_proto::core::app::v1::{
        query_service_server::QueryService, AppParametersRequest, AppParametersResponse,
        TransactionsByHeightRequest, TransactionsByHeightResponse,
    },
    tonic::Status,
    tracing::instrument,
};

pub(super) struct AppQueryServer {
    storage: Storage,
}

impl AppQueryServer {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl QueryService for AppQueryServer {
    #[instrument(skip(self, request))]
    async fn transactions_by_height(
        &self,
        request: tonic::Request<TransactionsByHeightRequest>,
    ) -> Result<tonic::Response<TransactionsByHeightResponse>, Status> {
        let state = self.storage.latest_snapshot();
        let request_inner = request.into_inner();
        let block_height = request_inner.block_height;

        let tx_response = state
            .transactions_by_height(block_height)
            .await
            .map_err(|e| tonic::Status::internal(format!("transaction response bad: {e}")))?;

        Ok(tonic::Response::new(tx_response))
    }

    #[instrument(skip(self, _request))]
    async fn app_parameters(
        &self,
        _request: tonic::Request<AppParametersRequest>,
    ) -> Result<tonic::Response<AppParametersResponse>, Status> {
        let state = self.storage.latest_snapshot();
        // We map the error here to avoid including `tonic` as a dependency
        // in the `chain` crate, to support its compilation to wasm.

        let app_parameters = state.get_app_params().await.map_err(|e| {
            tonic::Status::unavailable(format!("error getting app parameters: {e}"))
        })?;

        Ok(tonic::Response::new(AppParametersResponse {
            app_parameters: Some(app_parameters.into()),
        }))
    }
}
