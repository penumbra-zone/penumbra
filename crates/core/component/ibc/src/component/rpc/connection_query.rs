use async_trait::async_trait;

use cnidarium_component::ChainStateReadExt;
use ibc_proto::ibc::core::client::v1::Height;
use ibc_proto::ibc::core::connection::v1::query_server::Query as ConnectionQuery;
use ibc_proto::ibc::core::connection::v1::{
    ConnectionEnd, QueryClientConnectionsRequest, QueryClientConnectionsResponse,
    QueryConnectionClientStateRequest, QueryConnectionClientStateResponse,
    QueryConnectionConsensusStateRequest, QueryConnectionConsensusStateResponse,
    QueryConnectionRequest, QueryConnectionResponse, QueryConnectionsRequest,
    QueryConnectionsResponse,
};

use ibc_types::core::connection::{ConnectionId, IdentifiedConnectionEnd};
use ibc_types::path::ConnectionPath;
use ibc_types::DomainType;
use prost::Message;
use std::str::FromStr;

use crate::component::rpc::{Snapshot, Storage};
use crate::component::ConnectionStateReadExt;
use crate::prefix::MerklePrefixExt;
use crate::IBC_COMMITMENT_PREFIX;

use super::IbcQuery;

#[async_trait]
impl<C: ChainStateReadExt + Snapshot + 'static, S: Storage<C>> ConnectionQuery for IbcQuery<C, S> {
    /// Connection queries an IBC connection end.
    async fn connection(
        &self,
        request: tonic::Request<QueryConnectionRequest>,
    ) -> std::result::Result<tonic::Response<QueryConnectionResponse>, tonic::Status> {
        tracing::debug!("querying connection {:?}", request);
        let snapshot = self.0.latest_snapshot();
        let connection_id = &ConnectionId::from_str(&request.get_ref().connection_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid connection id: {e}")))?;

        let (conn, proof) = snapshot
            .get_with_proof(
                IBC_COMMITMENT_PREFIX
                    .apply_string(ConnectionPath::new(connection_id).to_string())
                    .as_bytes()
                    .to_vec(),
            )
            .await
            .map(|res| {
                let conn = res
                    .0
                    .map(|conn_bytes| ConnectionEnd::decode(conn_bytes.as_ref()))
                    .transpose();

                (conn, res.1)
            })
            .map_err(|e| tonic::Status::aborted(format!("couldn't get connection: {e}")))?;

        let conn =
            conn.map_err(|e| tonic::Status::aborted(format!("couldn't decode connection: {e}")))?;

        let height = Height {
            revision_number: 0,
            revision_height: snapshot.version(),
        };

        let res = QueryConnectionResponse {
            connection: conn,
            proof: proof.encode_to_vec(),
            proof_height: Some(height),
        };

        Ok(tonic::Response::new(res))
    }

    async fn connection_params(
        &self,
        _request: tonic::Request<
            ibc_proto::ibc::core::connection::v1::QueryConnectionParamsRequest,
        >,
    ) -> std::result::Result<
        tonic::Response<ibc_proto::ibc::core::connection::v1::QueryConnectionParamsResponse>,
        tonic::Status,
    > {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    /// Connections queries all the IBC connections of a chain.
    async fn connections(
        &self,
        _request: tonic::Request<QueryConnectionsRequest>,
    ) -> std::result::Result<tonic::Response<QueryConnectionsResponse>, tonic::Status> {
        let snapshot = self.0.latest_snapshot();
        let height = snapshot.version();

        let connection_counter = snapshot
            .get_connection_counter()
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get connection counter: {e}")))?;

        let mut connections = vec![];
        for conn_idx in 0..connection_counter.0 {
            let conn_id = ConnectionId(format!("connection-{}", conn_idx));
            let connection = snapshot
                .get_connection(&conn_id)
                .await
                .map_err(|e| {
                    tonic::Status::aborted(format!("couldn't get connection {conn_id}: {e}"))
                })?
                .ok_or("unable to get connection")
                .map_err(|e| {
                    tonic::Status::aborted(format!("couldn't get connection {conn_id}: {e}"))
                })?;
            let id_conn = IdentifiedConnectionEnd {
                connection_id: conn_id,
                connection_end: connection,
            };
            connections.push(id_conn.into());
        }

        let height = Height {
            revision_number: 0,
            revision_height: height,
        };

        let res = QueryConnectionsResponse {
            connections,
            pagination: None,
            height: Some(height),
        };

        Ok(tonic::Response::new(res))
    }
    /// ClientConnections queries the connection paths associated with a client
    /// state.
    async fn client_connections(
        &self,
        _request: tonic::Request<QueryClientConnectionsRequest>,
    ) -> std::result::Result<tonic::Response<QueryClientConnectionsResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// ConnectionClientState queries the client state associated with the
    /// connection.
    async fn connection_client_state(
        &self,
        _request: tonic::Request<QueryConnectionClientStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryConnectionClientStateResponse>, tonic::Status>
    {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// ConnectionConsensusState queries the consensus state associated with the
    /// connection.
    async fn connection_consensus_state(
        &self,
        _request: tonic::Request<QueryConnectionConsensusStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryConnectionConsensusStateResponse>, tonic::Status>
    {
        Err(tonic::Status::unimplemented("not implemented"))
    }
}
