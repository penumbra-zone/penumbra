use crate::TendermintProxy;
use penumbra_sdk_proto::{
    util::tendermint_proxy::v1::{
        tendermint_proxy_service_server::TendermintProxyService, AbciQueryRequest,
        AbciQueryResponse, BroadcastTxAsyncRequest, BroadcastTxAsyncResponse,
        BroadcastTxSyncRequest, BroadcastTxSyncResponse, GetBlockByHeightRequest,
        GetBlockByHeightResponse, GetStatusRequest, GetStatusResponse, GetTxRequest, GetTxResponse,
    },
    DomainType,
};
use penumbra_sdk_transaction::Transaction;
use tap::TapFallible;
use tendermint::{abci::Code, block::Height};
use tendermint_rpc::{Client, HttpClient};
use tonic::Status;
use tracing::instrument;

#[tonic::async_trait]
impl TendermintProxyService for TendermintProxy {
    // Note: the conversions that take place in here could be moved to
    // from/try_from impls, but they're not used anywhere else, so it's
    // unimportant right now, and would require additional wrappers
    // since none of the structs are defined in our crates :(
    // TODO: move those to proto/src/protobuf.rs

    /// Fetches a transaction by hash.
    ///
    /// Returns a [`GetTxResponse`] information about the requested transaction.
    #[instrument(level = "info", skip_all)]
    async fn get_tx(
        &self,
        req: tonic::Request<GetTxRequest>,
    ) -> Result<tonic::Response<GetTxResponse>, Status> {
        // Create an HTTP client, connecting to tendermint.
        let client = HttpClient::new(self.tendermint_url.as_ref()).map_err(|e| {
            Status::unavailable(format!("error creating tendermint http client: {e:#?}"))
        })?;

        // Parse the inbound transaction hash from the client request.
        let GetTxRequest { hash, prove } = req.into_inner();
        let hash = hash
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("invalid transaction hash: {e:#?}")))?;

        // Send the request to Tendermint.
        let rsp = client
            .tx(hash, prove)
            .await
            .map(GetTxResponse::from)
            .map_err(|e| Status::unavailable(format!("error getting tx: {e}")))?;

        // Before forwarding along the response, verify that the transaction can be
        // successfully decoded into our domain type.
        Transaction::decode(rsp.tx.as_ref())
            .map_err(|e| Status::unavailable(format!("error decoding tx: {e}")))?;

        Ok(tonic::Response::new(rsp))
    }

    /// Broadcasts a transaction asynchronously.
    #[instrument(
        level = "info",
        skip_all,
        fields(req_id = tracing::field::Empty),
    )]
    async fn broadcast_tx_async(
        &self,
        req: tonic::Request<BroadcastTxAsyncRequest>,
    ) -> Result<tonic::Response<BroadcastTxAsyncResponse>, Status> {
        // Create an HTTP client, connecting to tendermint.
        let client = HttpClient::new(self.tendermint_url.as_ref()).map_err(|e| {
            Status::unavailable(format!("error creating tendermint http client: {e:#?}"))
        })?;

        // Process the inbound request, recording the request ID in the tracing span.
        let BroadcastTxAsyncRequest { req_id, params } = req.into_inner();
        tracing::Span::current().record("req_id", req_id);

        // Broadcast the transaction parameters.
        client
            .broadcast_tx_async(params)
            .await
            .map(BroadcastTxAsyncResponse::from)
            .map(tonic::Response::new)
            .map_err(|e| Status::unavailable(format!("error broadcasting tx async: {e}")))
    }

    // Broadcasts a transaction synchronously.
    #[instrument(
        level = "info",
        skip_all,
        fields(req_id = tracing::field::Empty),
    )]
    async fn broadcast_tx_sync(
        &self,
        req: tonic::Request<BroadcastTxSyncRequest>,
    ) -> Result<tonic::Response<BroadcastTxSyncResponse>, Status> {
        // Create an HTTP client, connecting to tendermint.
        let client = HttpClient::new(self.tendermint_url.as_ref()).map_err(|e| {
            Status::unavailable(format!("error creating tendermint http client: {e:#?}"))
        })?;

        // Process the inbound request, recording the request ID in the tracing span.
        let BroadcastTxSyncRequest { req_id, params } = req.into_inner();
        tracing::Span::current().record("req_id", req_id);

        // Broadcast the transaction parameters.
        client
            .broadcast_tx_sync(params)
            .await
            .map(BroadcastTxSyncResponse::from)
            .map(tonic::Response::new)
            .map_err(|e| tonic::Status::unavailable(format!("error broadcasting tx sync: {e}")))
            .tap_ok(|res| tracing::debug!("{:?}", res))
    }

    // Queries the current status.
    #[instrument(level = "info", skip_all)]
    async fn get_status(
        &self,
        _req: tonic::Request<GetStatusRequest>,
    ) -> Result<tonic::Response<GetStatusResponse>, Status> {
        // generic bounds on HttpClient::new are not well-constructed, so we have to
        // render the URL as a String, then borrow it, then re-parse the borrowed &str
        let client = HttpClient::new(self.tendermint_url.as_ref()).map_err(|e| {
            tonic::Status::unavailable(format!("error creating tendermint http client: {e:#?}"))
        })?;

        // Send the status request.
        client
            .status()
            .await
            .map(GetStatusResponse::from)
            .map(tonic::Response::new)
            .map_err(|e| tonic::Status::unavailable(format!("error querying status: {e}")))
    }

    #[instrument(level = "info", skip_all)]
    async fn abci_query(
        &self,
        req: tonic::Request<AbciQueryRequest>,
    ) -> Result<tonic::Response<AbciQueryResponse>, Status> {
        // generic bounds on HttpClient::new are not well-constructed, so we have to
        // render the URL as a String, then borrow it, then re-parse the borrowed &str
        let client = HttpClient::new(self.tendermint_url.to_string().as_ref()).map_err(|e| {
            tonic::Status::unavailable(format!("error creating tendermint http client: {e:#?}"))
        })?;

        // Parse the inbound request, confirm that the height provided is valid.
        // TODO: how does path validation work on tendermint-rs@29
        let AbciQueryRequest {
            data,
            path,
            height,
            prove,
        } = req.into_inner();
        let height: Height = height
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid height"))?;

        // Send the ABCI query to Tendermint.
        let rsp = client
            .abci_query(Some(path), data, Some(height), prove)
            .await
            .map_err(|e| Status::unavailable(format!("error querying abci: {e}")))
            // Confirm that the response code is 0, or return an error response.
            .and_then(|rsp| match rsp.code {
                Code::Ok => Ok(rsp),
                tendermint::abci::Code::Err(e) => {
                    Err(Status::unavailable(format!("error querying abci: {e}")))
                }
            })?;

        AbciQueryResponse::try_from(rsp)
            .map(tonic::Response::new)
            .map_err(|error| Status::internal(format!("{error}")))
    }

    #[instrument(level = "info", skip_all)]
    async fn get_block_by_height(
        &self,
        req: tonic::Request<GetBlockByHeightRequest>,
    ) -> Result<tonic::Response<GetBlockByHeightResponse>, Status> {
        // generic bounds on HttpClient::new are not well-constructed, so we have to
        // render the URL as a String, then borrow it, then re-parse the borrowed &str
        let client = HttpClient::new(self.tendermint_url.to_string().as_ref()).map_err(|e| {
            tonic::Status::unavailable(format!("error creating tendermint http client: {e:#?}"))
        })?;

        // Parse the height from the inbound client request.
        let GetBlockByHeightRequest { height } = req.into_inner();
        let height =
            tendermint::block::Height::try_from(height).expect("height should be less than 2^63");

        // Fetch the block and forward Tendermint's response back to the client.
        client
            .block(height)
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error querying abci: {e}")))
            .and_then(|b| {
                match GetBlockByHeightResponse::try_from(b) {
                    Ok(b) => Ok(b),
                    Err(e) => {
                        tracing::warn!(?height, error = ?e, "proxy: error deserializing GetBlockByHeightResponse");
                        Err(tonic::Status::internal("error deserializing GetBlockByHeightResponse"))
                    }
                }
            })
            .map(tonic::Response::new)
    }
}
