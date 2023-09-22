use async_trait::async_trait;

use ibc_proto::ibc::core::client::v1::query_server::Query as ClientQuery;
use ibc_proto::ibc::core::client::v1::{
    IdentifiedClientState, QueryClientParamsRequest, QueryClientParamsResponse,
    QueryClientStateRequest, QueryClientStateResponse, QueryClientStatesRequest,
    QueryClientStatesResponse, QueryClientStatusRequest, QueryClientStatusResponse,
    QueryConsensusStateHeightsRequest, QueryConsensusStateHeightsResponse,
    QueryConsensusStateRequest, QueryConsensusStateResponse, QueryConsensusStatesRequest,
    QueryConsensusStatesResponse, QueryUpgradedClientStateRequest,
    QueryUpgradedClientStateResponse, QueryUpgradedConsensusStateRequest,
    QueryUpgradedConsensusStateResponse,
};

use ibc_types::core::client::ClientId;

use std::str::FromStr;
use tonic::{Response, Status};

use crate::component::ClientStateReadExt;

use super::IbcQuery;

#[async_trait]
impl ClientQuery for IbcQuery {
    async fn client_state(
        &self,
        _request: tonic::Request<QueryClientStateRequest>,
    ) -> std::result::Result<Response<QueryClientStateResponse>, Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// ClientStates queries all the IBC light clients of a chain.
    async fn client_states(
        &self,
        _request: tonic::Request<QueryClientStatesRequest>,
    ) -> std::result::Result<tonic::Response<QueryClientStatesResponse>, tonic::Status> {
        let snapshot = self.0.latest_snapshot();

        let client_counter = snapshot
            .client_counter()
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get client counter: {e}")))?
            .0;

        let mut client_states = vec![];
        for client_idx in 0..client_counter {
            // NOTE: currently, we only look up tendermint clients, because we only support tendermint clients.
            let client_id = ClientId::from_str(format!("07-tendermint-{}", client_idx).as_str())
                .map_err(|e| tonic::Status::aborted(format!("invalid client id: {e}")))?;
            let client_state = snapshot.get_client_state(&client_id).await;
            let id_client = IdentifiedClientState {
                client_id: client_id.to_string(),
                client_state: client_state.ok().map(|state| state.into()), // send None if we couldn't find the client state
            };
            client_states.push(id_client);
        }

        let res = QueryClientStatesResponse {
            client_states,
            pagination: None,
        };

        Ok(tonic::Response::new(res))
    }
    /// ConsensusState queries a consensus state associated with a client state at
    /// a given height.
    async fn consensus_state(
        &self,
        _request: tonic::Request<QueryConsensusStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryConsensusStateResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// ConsensusStates queries all the consensus state associated with a given
    /// client.
    async fn consensus_states(
        &self,
        _request: tonic::Request<QueryConsensusStatesRequest>,
    ) -> std::result::Result<tonic::Response<QueryConsensusStatesResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// ConsensusStateHeights queries the height of every consensus states associated with a given client.
    async fn consensus_state_heights(
        &self,
        _request: tonic::Request<QueryConsensusStateHeightsRequest>,
    ) -> std::result::Result<tonic::Response<QueryConsensusStateHeightsResponse>, tonic::Status>
    {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// Status queries the status of an IBC client.
    async fn client_status(
        &self,
        _request: tonic::Request<QueryClientStatusRequest>,
    ) -> std::result::Result<tonic::Response<QueryClientStatusResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// ClientParams queries all parameters of the ibc client.
    async fn client_params(
        &self,
        _request: tonic::Request<QueryClientParamsRequest>,
    ) -> std::result::Result<tonic::Response<QueryClientParamsResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// UpgradedClientState queries an Upgraded IBC light client.
    async fn upgraded_client_state(
        &self,
        _request: tonic::Request<QueryUpgradedClientStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryUpgradedClientStateResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// UpgradedConsensusState queries an Upgraded IBC consensus state.
    async fn upgraded_consensus_state(
        &self,
        _request: tonic::Request<QueryUpgradedConsensusStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryUpgradedConsensusStateResponse>, tonic::Status>
    {
        Err(tonic::Status::unimplemented("not implemented"))
    }
}
