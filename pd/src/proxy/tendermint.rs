use penumbra_proto::tendermint_proxy::tendermint_query_server::TendermintQuery;
use penumbra_proto::tendermint_proxy::{
    AbciQueryRequest, AbciQueryResponse, GetBlockByHeightRequest, GetBlockByHeightResponse,
    GetLatestBlockRequest, GetLatestBlockResponse, GetLatestValidatorSetRequest,
    GetLatestValidatorSetResponse, GetNodeInfoRequest, GetNodeInfoResponse, GetSyncingRequest,
    GetSyncingResponse, GetValidatorSetByHeightRequest, GetValidatorSetByHeightResponse,
};
use tendermint_rpc::Client;
use tonic;

use tendermint_rpc;

#[derive(Clone)]
pub struct TendermintProxyStub {
    pub tendermint_instance: String,
}

impl Default for TendermintProxyStub {
    fn default() -> Self {
        Self {
            tendermint_instance: "127.0.0.1:26656".to_string(),
        }
    }
}

#[tonic::async_trait]
impl TendermintQuery for TendermintProxyStub {
    async fn get_node_info(
        &self,
        request: tonic::Request<GetNodeInfoRequest>,
    ) -> Result<tonic::Response<GetNodeInfoResponse>, tonic::Status> {
        let mut client =
            tendermint_rpc::HttpClient::new(self.tendermint_instance.as_str()).unwrap();
        let resp = client.status().await.unwrap();

        // Find a way to get from RPC types to proto types.
        // Enquiring with tendermint-rs devs, there was no clean way to do this in the code
        // I have reviewed. Might be necessary to either:
        //      - implement `TryFrom` / `TryInto` upstream
        //      - get the upstream repo to consolidate types

        // Ok(tonic::Response::new(resp.into()));
        unimplemented!()
    }

    #[doc = " GetSyncing queries node syncing."]
    async fn get_syncing(
        &self,
        request: tonic::Request<GetSyncingRequest>,
    ) -> Result<tonic::Response<GetSyncingResponse>, tonic::Status> {
        unimplemented!()
    }

    #[doc = " GetLatestBlock returns the latest block."]
    async fn get_latest_block(
        &self,
        request: tonic::Request<GetLatestBlockRequest>,
    ) -> Result<tonic::Response<GetLatestBlockResponse>, tonic::Status> {
        unimplemented!()
    }

    #[doc = " GetBlockByHeight queries block for given height."]
    async fn get_block_by_height(
        &self,
        request: tonic::Request<GetBlockByHeightRequest>,
    ) -> Result<tonic::Response<GetBlockByHeightResponse>, tonic::Status> {
        unimplemented!()
    }

    #[doc = " GetLatestValidatorSet queries latest validator-set."]
    async fn get_latest_validator_set(
        &self,
        request: tonic::Request<GetLatestValidatorSetRequest>,
    ) -> Result<tonic::Response<GetLatestValidatorSetResponse>, tonic::Status> {
        unimplemented!()
    }

    #[doc = " GetValidatorSetByHeight queries validator-set at a given height."]
    async fn get_validator_set_by_height(
        &self,
        request: tonic::Request<GetValidatorSetByHeightRequest>,
    ) -> Result<tonic::Response<GetValidatorSetByHeightResponse>, tonic::Status> {
        unimplemented!()
    }

    #[doc = " ABCIQuery defines a query handler that supports ABCI queries directly to the"]
    #[doc = " application, bypassing Tendermint completely. The ABCI query must contain"]
    #[doc = " a valid and supported path, including app, custom, p2p, and store."]
    #[doc = ""]
    #[doc = " Since: cosmos-sdk 0.46"]
    async fn abci_query(
        &self,
        request: tonic::Request<AbciQueryRequest>,
    ) -> Result<tonic::Response<AbciQueryResponse>, tonic::Status> {
        unimplemented!()
    }
}
