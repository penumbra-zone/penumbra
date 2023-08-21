use async_trait::async_trait;
use ibc_proto::ibc::core::client::v1::query_server::Query;
use ibc_proto::ibc::core::client::v1::{
    QueryClientParamsRequest, QueryClientParamsResponse, QueryClientStateRequest,
    QueryClientStateResponse, QueryClientStatesRequest, QueryClientStatesResponse,
    QueryClientStatusRequest, QueryClientStatusResponse, QueryConsensusStateHeightsRequest,
    QueryConsensusStateHeightsResponse, QueryConsensusStateRequest, QueryConsensusStateResponse,
    QueryConsensusStatesRequest, QueryConsensusStatesResponse, QueryUpgradedClientStateRequest,
    QueryUpgradedClientStateResponse, QueryUpgradedConsensusStateRequest,
    QueryUpgradedConsensusStateResponse,
};
use tonic::{Response, Status};
use tower::ServiceExt;

#[derive(Clone)]
pub struct IbcQuery();

#[async_trait]
impl Query for IbcQuery {
    async fn client_state(
        &self,
        request: tonic::Request<QueryClientStateRequest>,
    ) -> std::result::Result<Response<QueryClientStateResponse>, Status> {
        todo!()
    }
    /// ClientStates queries all the IBC light clients of a chain.
    async fn client_states(
        &self,
        request: tonic::Request<QueryClientStatesRequest>,
    ) -> std::result::Result<tonic::Response<QueryClientStatesResponse>, tonic::Status> {
        todo!()
    }
    /// ConsensusState queries a consensus state associated with a client state at
    /// a given height.
    async fn consensus_state(
        &self,
        request: tonic::Request<QueryConsensusStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryConsensusStateResponse>, tonic::Status> {
        todo!()
    }
    /// ConsensusStates queries all the consensus state associated with a given
    /// client.
    async fn consensus_states(
        &self,
        request: tonic::Request<QueryConsensusStatesRequest>,
    ) -> std::result::Result<tonic::Response<QueryConsensusStatesResponse>, tonic::Status> {
        todo!()
    }
    /// ConsensusStateHeights queries the height of every consensus states associated with a given client.
    async fn consensus_state_heights(
        &self,
        request: tonic::Request<QueryConsensusStateHeightsRequest>,
    ) -> std::result::Result<tonic::Response<QueryConsensusStateHeightsResponse>, tonic::Status>
    {
        todo!()
    }
    /// Status queries the status of an IBC client.
    async fn client_status(
        &self,
        request: tonic::Request<QueryClientStatusRequest>,
    ) -> std::result::Result<tonic::Response<QueryClientStatusResponse>, tonic::Status> {
        todo!()
    }
    /// ClientParams queries all parameters of the ibc client.
    async fn client_params(
        &self,
        request: tonic::Request<QueryClientParamsRequest>,
    ) -> std::result::Result<tonic::Response<QueryClientParamsResponse>, tonic::Status> {
        todo!()
    }
    /// UpgradedClientState queries an Upgraded IBC light client.
    async fn upgraded_client_state(
        &self,
        request: tonic::Request<QueryUpgradedClientStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryUpgradedClientStateResponse>, tonic::Status> {
        todo!()
    }
    /// UpgradedConsensusState queries an Upgraded IBC consensus state.
    async fn upgraded_consensus_state(
        &self,
        request: tonic::Request<QueryUpgradedConsensusStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryUpgradedConsensusStateResponse>, tonic::Status>
    {
        todo!()
    }
}
