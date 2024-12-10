use async_trait::async_trait;

use ibc_proto::ibc::core::client::v1::{Height, IdentifiedClientState};
use ibc_proto::ibc::core::connection::v1::query_server::Query as ConnectionQuery;
use ibc_proto::ibc::core::connection::v1::{
    ConnectionEnd, QueryClientConnectionsRequest, QueryClientConnectionsResponse,
    QueryConnectionClientStateRequest, QueryConnectionClientStateResponse,
    QueryConnectionConsensusStateRequest, QueryConnectionConsensusStateResponse,
    QueryConnectionRequest, QueryConnectionResponse, QueryConnectionsRequest,
    QueryConnectionsResponse,
};

use ibc_types::core::client::ClientId;
use ibc_types::core::connection::{ClientPaths, ConnectionId, IdentifiedConnectionEnd};
use ibc_types::path::{
    ClientConnectionPath, ClientConsensusStatePath, ClientStatePath, ConnectionPath,
};
use ibc_types::DomainType;
use prost::Message;
use std::str::FromStr;

use crate::component::rpc::utils::determine_snapshot_from_metadata;
use crate::component::{ConnectionStateReadExt, HostInterface};
use crate::prefix::MerklePrefixExt;
use crate::IBC_COMMITMENT_PREFIX;

use super::IbcQuery;

#[async_trait]
impl<HI: HostInterface + Send + Sync + 'static> ConnectionQuery for IbcQuery<HI> {
    /// Connection queries an IBC connection end.
    #[tracing::instrument(skip(self), err, level = "debug")]
    async fn connection(
        &self,
        request: tonic::Request<QueryConnectionRequest>,
    ) -> std::result::Result<tonic::Response<QueryConnectionResponse>, tonic::Status> {
        let snapshot = match determine_snapshot_from_metadata(self.storage.clone(), request.metadata()) {
            Err(err) => return Err(tonic::Status::aborted(
                format!("could not determine the correct snapshot to open given the `\"height\"` header of the request: {err:#}")
            )),
            Ok(snapshot) => snapshot,
        };

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

        let res = QueryConnectionResponse {
            connection: conn,
            proof: proof.encode_to_vec(),
            proof_height: Some(ibc_proto::ibc::core::client::v1::Height {
                revision_height: HI::get_block_height(&snapshot)
                    .await
                    .map_err(|e| tonic::Status::aborted(format!("couldn't decode height: {e}")))?
                    + 1,
                revision_number: HI::get_revision_number(&snapshot)
                    .await
                    .map_err(|e| tonic::Status::aborted(format!("couldn't decode height: {e}")))?,
            }),
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
    #[tracing::instrument(skip(self), err, level = "debug")]
    async fn connections(
        &self,
        _request: tonic::Request<QueryConnectionsRequest>,
    ) -> std::result::Result<tonic::Response<QueryConnectionsResponse>, tonic::Status> {
        let snapshot = self.storage.latest_snapshot();
        let height = HI::get_block_height(&snapshot)
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't decode height: {e}")))?
            + 1;

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
    #[tracing::instrument(skip(self), err, level = "debug")]
    async fn client_connections(
        &self,
        request: tonic::Request<QueryClientConnectionsRequest>,
    ) -> std::result::Result<tonic::Response<QueryClientConnectionsResponse>, tonic::Status> {
        let snapshot = match determine_snapshot_from_metadata(self.storage.clone(), request.metadata()) {
            Err(err) => return Err(tonic::Status::aborted(
                format!("could not determine the correct snapshot to open given the `\"height\"` header of the request: {err:#}")
            )),
            Ok(snapshot) => snapshot,
        };
        let client_id = &ClientId::from_str(&request.get_ref().client_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid client id: {e}")))?;

        let (client_connections, proof) = snapshot
            .get_with_proof(
                IBC_COMMITMENT_PREFIX
                    .apply_string(ClientConnectionPath::new(client_id).to_string())
                    .as_bytes()
                    .to_vec(),
            )
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get client connections: {e}")))?;

        let connection_paths: Vec<String> = client_connections
            .map(|client_connections| ClientPaths::decode(client_connections.as_ref()))
            .transpose()
            .map_err(|e| {
                tonic::Status::aborted(format!("couldn't decode client connections: {e}"))
            })?
            .map(|client_paths| client_paths.paths)
            .map(|paths| paths.into_iter().map(|path| path.to_string()).collect())
            .unwrap_or_default();

        Ok(tonic::Response::new(QueryClientConnectionsResponse {
            connection_paths,
            proof: proof.encode_to_vec(),
            proof_height: Some(ibc_proto::ibc::core::client::v1::Height {
                revision_height: HI::get_block_height(&snapshot)
                    .await
                    .map_err(|e| tonic::Status::aborted(format!("couldn't decode height: {e}")))?
                    + 1,
                revision_number: HI::get_revision_number(&snapshot)
                    .await
                    .map_err(|e| tonic::Status::aborted(format!("couldn't decode height: {e}")))?,
            }),
        }))
    }
    /// ConnectionClientState queries the client state associated with the
    /// connection.
    #[tracing::instrument(skip(self), err, level = "debug")]
    async fn connection_client_state(
        &self,
        request: tonic::Request<QueryConnectionClientStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryConnectionClientStateResponse>, tonic::Status>
    {
        let snapshot = match determine_snapshot_from_metadata(self.storage.clone(), request.metadata()) {
            Err(err) => return Err(tonic::Status::aborted(
                format!("could not determine the correct snapshot to open given the `\"height\"` header of the request: {err:#}")
            )),
            Ok(snapshot) => snapshot,
        };
        let connection_id = &ConnectionId::from_str(&request.get_ref().connection_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid connection id: {e}")))?;

        let client_id = snapshot
            .get_connection(connection_id)
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get connection: {e}")))?
            .ok_or("unable to get connection")
            .map_err(|e| tonic::Status::aborted(format!("couldn't get connection: {e}")))?
            .client_id;

        let (client_state, proof) = snapshot
            .get_with_proof(
                IBC_COMMITMENT_PREFIX
                    .apply_string(ClientStatePath::new(&client_id).to_string())
                    .as_bytes()
                    .to_vec(),
            )
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get client state: {e}")))?;

        let client_state_any = client_state
            .map(|cs_bytes| ibc_proto::google::protobuf::Any::decode(cs_bytes.as_ref()))
            .transpose()
            .map_err(|e| tonic::Status::aborted(format!("couldn't decode client state: {e}")))?;

        let identified_client_state = IdentifiedClientState {
            client_id: client_id.clone().to_string(),
            client_state: client_state_any,
        };

        Ok(tonic::Response::new(QueryConnectionClientStateResponse {
            identified_client_state: Some(identified_client_state),
            proof: proof.encode_to_vec(),
            proof_height: Some(ibc_proto::ibc::core::client::v1::Height {
                revision_height: HI::get_block_height(&snapshot)
                    .await
                    .map_err(|e| tonic::Status::aborted(format!("couldn't decode height: {e}")))?
                    + 1,
                revision_number: HI::get_revision_number(&snapshot)
                    .await
                    .map_err(|e| tonic::Status::aborted(format!("couldn't decode height: {e}")))?,
            }),
        }))
    }
    /// ConnectionConsensusState queries the consensus state associated with the
    /// connection.
    #[tracing::instrument(skip(self), err, level = "debug")]
    async fn connection_consensus_state(
        &self,
        request: tonic::Request<QueryConnectionConsensusStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryConnectionConsensusStateResponse>, tonic::Status>
    {
        let snapshot = match determine_snapshot_from_metadata(self.storage.clone(), request.metadata()) {
            Err(err) => return Err(tonic::Status::aborted(
                format!("could not determine the correct snapshot to open given the `\"height\"` header of the request: {err:#}")
            )),
            Ok(snapshot) => snapshot,
        };
        let consensus_state_height = ibc_types::core::client::Height {
            revision_number: request.get_ref().revision_number,
            revision_height: request.get_ref().revision_height,
        };
        let connection_id = &ConnectionId::from_str(&request.get_ref().connection_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid connection id: {e}")))?;

        let client_id = snapshot
            .get_connection(connection_id)
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get connection: {e}")))?
            .ok_or("unable to get connection")
            .map_err(|e| tonic::Status::aborted(format!("couldn't get connection: {e}")))?
            .client_id;

        let (consensus_state, proof) = snapshot
            .get_with_proof(
                IBC_COMMITMENT_PREFIX
                    .apply_string(
                        ClientConsensusStatePath::new(&client_id, &consensus_state_height)
                            .to_string(),
                    )
                    .as_bytes()
                    .to_vec(),
            )
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get client state: {e}")))?;

        let consensus_state_any = consensus_state
            .map(|cs_bytes| ibc_proto::google::protobuf::Any::decode(cs_bytes.as_ref()))
            .transpose()
            .map_err(|e| tonic::Status::aborted(format!("couldn't decode client state: {e}")))?;

        Ok(tonic::Response::new(
            QueryConnectionConsensusStateResponse {
                consensus_state: consensus_state_any,
                client_id: client_id.to_string(),
                proof: proof.encode_to_vec(),
                proof_height: Some(ibc_proto::ibc::core::client::v1::Height {
                    revision_height: HI::get_block_height(&snapshot).await.map_err(|e| {
                        tonic::Status::aborted(format!("couldn't decode height: {e}"))
                    })? + 1,
                    revision_number: HI::get_revision_number(&snapshot).await.map_err(|e| {
                        tonic::Status::aborted(format!("couldn't decode height: {e}"))
                    })?,
                }),
            },
        ))
    }
}
