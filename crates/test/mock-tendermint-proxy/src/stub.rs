use {
    penumbra_sdk_proto::util::tendermint_proxy::v1::{
        tendermint_proxy_service_server::TendermintProxyService, AbciQueryRequest,
        AbciQueryResponse, BroadcastTxAsyncRequest, BroadcastTxAsyncResponse,
        BroadcastTxSyncRequest, BroadcastTxSyncResponse, GetBlockByHeightRequest,
        GetBlockByHeightResponse, GetStatusRequest, GetStatusResponse, GetTxRequest, GetTxResponse,
    },
    tonic::Status,
    tracing::instrument,
};

/// A tendermint proxy service for use in tests.
///
/// This implements [`TendermintProxyService`], but will return a [`Status::unimplemented`] error
/// for any requests it receives.
pub struct StubProxy;

#[tonic::async_trait]
impl TendermintProxyService for StubProxy {
    async fn get_tx(
        &self,
        _req: tonic::Request<GetTxRequest>,
    ) -> Result<tonic::Response<GetTxResponse>, Status> {
        Err(Status::unimplemented("get_tx"))
    }

    /// Broadcasts a transaction asynchronously.
    #[instrument(
        level = "info",
        skip_all,
        fields(req_id = tracing::field::Empty),
    )]
    async fn broadcast_tx_async(
        &self,
        _req: tonic::Request<BroadcastTxAsyncRequest>,
    ) -> Result<tonic::Response<BroadcastTxAsyncResponse>, Status> {
        Err(Status::unimplemented("broadcast_tx_async"))
    }

    // Broadcasts a transaction synchronously.
    #[instrument(
        level = "info",
        skip_all,
        fields(req_id = tracing::field::Empty),
    )]
    async fn broadcast_tx_sync(
        &self,
        _req: tonic::Request<BroadcastTxSyncRequest>,
    ) -> Result<tonic::Response<BroadcastTxSyncResponse>, Status> {
        Err(Status::unimplemented("broadcast_tx_sync"))
    }

    // Queries the current status.
    #[instrument(level = "info", skip_all)]
    async fn get_status(
        &self,
        __req: tonic::Request<GetStatusRequest>,
    ) -> Result<tonic::Response<GetStatusResponse>, Status> {
        Err(Status::unimplemented("get_status"))
    }

    #[instrument(level = "info", skip_all)]
    async fn abci_query(
        &self,
        _req: tonic::Request<AbciQueryRequest>,
    ) -> Result<tonic::Response<AbciQueryResponse>, Status> {
        Err(Status::unimplemented("abci_query"))
    }

    #[instrument(level = "info", skip_all)]
    async fn get_block_by_height(
        &self,
        _req: tonic::Request<GetBlockByHeightRequest>,
    ) -> Result<tonic::Response<GetBlockByHeightResponse>, Status> {
        Err(Status::unimplemented("get_block_by_height"))
    }
}
