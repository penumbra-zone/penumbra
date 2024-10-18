use async_trait::async_trait;

use ibc_proto::ibc::core::client::v1::query_server::Query as ClientQuery;
use ibc_proto::ibc::core::client::v1::{
    ConsensusStateWithHeight, IdentifiedClientState, QueryClientParamsRequest,
    QueryClientParamsResponse, QueryClientStateRequest, QueryClientStateResponse,
    QueryClientStatesRequest, QueryClientStatesResponse, QueryClientStatusRequest,
    QueryClientStatusResponse, QueryConsensusStateHeightsRequest,
    QueryConsensusStateHeightsResponse, QueryConsensusStateRequest, QueryConsensusStateResponse,
    QueryConsensusStatesRequest, QueryConsensusStatesResponse, QueryUpgradedClientStateRequest,
    QueryUpgradedClientStateResponse, QueryUpgradedConsensusStateRequest,
    QueryUpgradedConsensusStateResponse,
};
use prost::Message;

use ibc_types::core::client::ClientId;
use ibc_types::core::client::Height;
use ibc_types::path::ClientConsensusStatePath;
use ibc_types::path::ClientStatePath;
use ibc_types::DomainType;

use std::str::FromStr;
use tonic::{Response, Status};

use crate::component::{ClientStateReadExt, HostInterface};
use crate::prefix::MerklePrefixExt;
use crate::IBC_COMMITMENT_PREFIX;

use super::utils::determine_snapshot_from_metadata;
use super::IbcQuery;

#[async_trait]
impl<HI: HostInterface + Send + Sync + 'static> ClientQuery for IbcQuery<HI> {
    async fn client_state(
        &self,
        request: tonic::Request<QueryClientStateRequest>,
    ) -> std::result::Result<Response<QueryClientStateResponse>, Status> {
        let snapshot = match determine_snapshot_from_metadata(self.storage.clone(), request.metadata()) {
            Err(err) => return Err(tonic::Status::aborted(
                format!("could not determine the correct snapshot to open given the `\"height\"` header of the request: {err:#}")
            )),
            Ok(snapshot) => snapshot,
        };

        let client_id = ClientId::from_str(&request.get_ref().client_id)
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid client id: {e}")))?;

        // Query for client_state and associated proof.
        let (cs_opt, proof) = snapshot
            .get_with_proof(
                IBC_COMMITMENT_PREFIX
                    .apply_string(ClientStatePath(client_id.clone()).to_string())
                    .as_bytes()
                    .to_vec(),
            )
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get client: {e}")))?;

        let client_state = cs_opt
            .map(|cs_opt| ibc_proto::google::protobuf::Any::decode(cs_opt.as_ref()))
            .transpose()
            .map_err(|e| tonic::Status::aborted(format!("couldn't decode client state: {e}")))?;

        let res = QueryClientStateResponse {
            client_state,
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

    /// ClientStates queries all the IBC light clients of a chain.
    async fn client_states(
        &self,
        _request: tonic::Request<QueryClientStatesRequest>,
    ) -> std::result::Result<tonic::Response<QueryClientStatesResponse>, tonic::Status> {
        let snapshot = self.storage.latest_snapshot();

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
        request: tonic::Request<QueryConsensusStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryConsensusStateResponse>, tonic::Status> {
        let snapshot = match determine_snapshot_from_metadata(self.storage.clone(), request.metadata()) {
            Err(err) => return Err(tonic::Status::aborted(
                format!("could not determine the correct snapshot to open given the `\"height\"` header of the request: {err:#}")
            )),
            Ok(snapshot) => snapshot,
        };

        let client_id = ClientId::from_str(&request.get_ref().client_id)
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid client id: {e}")))?;
        let height = if request.get_ref().latest_height {
            get_latest_verified_height(&snapshot, &client_id).await?
        } else {
            Height {
                revision_number: request.get_ref().revision_number,
                revision_height: request.get_ref().revision_height,
            }
        };

        let (cs_opt, proof) = snapshot
            .get_with_proof(
                IBC_COMMITMENT_PREFIX
                    .apply_string(ClientConsensusStatePath::new(&client_id, &height).to_string())
                    .as_bytes()
                    .to_vec(),
            )
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get consensus state: {e}")))?;

        let consensus_state = cs_opt
            .map(|cs_opt| ibc_proto::google::protobuf::Any::decode(cs_opt.as_ref()))
            .transpose()
            .map_err(|e| tonic::Status::aborted(format!("couldn't decode consensus state: {e}")))?;

        let res = QueryConsensusStateResponse {
            consensus_state,
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

    /// ConsensusStates queries all the consensus state associated with a given
    /// client.
    async fn consensus_states(
        &self,
        request: tonic::Request<QueryConsensusStatesRequest>,
    ) -> std::result::Result<tonic::Response<QueryConsensusStatesResponse>, tonic::Status> {
        let snapshot = self.storage.latest_snapshot();
        let client_id = ClientId::from_str(&request.get_ref().client_id)
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid client id: {e}")))?;

        let verified_heights = snapshot
            .get_verified_heights(&client_id)
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get verified heights: {e}")))?;
        let resp = if let Some(verified_heights) = verified_heights {
            let mut consensus_states = Vec::with_capacity(verified_heights.heights.len());
            for height in verified_heights.heights {
                let consensus_state = snapshot
                    .get_verified_consensus_state(&height, &client_id)
                    .await
                    .map_err(|e| {
                        tonic::Status::aborted(format!("couldn't get consensus state: {e}"))
                    })?;
                consensus_states.push(ConsensusStateWithHeight {
                    height: Some(height.into()),
                    consensus_state: Some(consensus_state.into()),
                });
            }
            QueryConsensusStatesResponse {
                consensus_states,
                pagination: None,
            }
        } else {
            QueryConsensusStatesResponse {
                consensus_states: vec![],
                pagination: None,
            }
        };

        Ok(tonic::Response::new(resp))
    }

    /// ConsensusStateHeights queries the height of every consensus states associated with a given client.
    async fn consensus_state_heights(
        &self,
        request: tonic::Request<QueryConsensusStateHeightsRequest>,
    ) -> std::result::Result<tonic::Response<QueryConsensusStateHeightsResponse>, tonic::Status>
    {
        let snapshot = self.storage.latest_snapshot();
        let client_id = ClientId::from_str(&request.get_ref().client_id)
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid client id: {e}")))?;

        let verified_heights = snapshot
            .get_verified_heights(&client_id)
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get verified heights: {e}")))?;
        let resp = if let Some(verified_heights) = verified_heights {
            QueryConsensusStateHeightsResponse {
                consensus_state_heights: verified_heights
                    .heights
                    .into_iter()
                    .map(|h| h.into())
                    .collect(),
                pagination: None,
            }
        } else {
            QueryConsensusStateHeightsResponse {
                consensus_state_heights: vec![],
                pagination: None,
            }
        };

        Ok(tonic::Response::new(resp))
    }

    /// Status queries the status of an IBC client.
    async fn client_status(
        &self,
        request: tonic::Request<QueryClientStatusRequest>,
    ) -> std::result::Result<tonic::Response<QueryClientStatusResponse>, tonic::Status> {
        let snapshot = self.storage.latest_snapshot();
        let client_id = ClientId::from_str(&request.get_ref().client_id)
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid client id: {e}")))?;
        let timestamp = HI::get_block_timestamp(snapshot.clone())
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get block timestamp: {e}")))?;
        let client_status = snapshot.get_client_status(&client_id, timestamp).await;
        let resp = QueryClientStatusResponse {
            status: client_status.to_string(),
        };

        Ok(tonic::Response::new(resp))
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

async fn get_latest_verified_height(
    snapshot: &cnidarium::Snapshot,
    client_id: &ClientId,
) -> Result<Height, tonic::Status> {
    let verified_heights = snapshot
        .get_verified_heights(&client_id)
        .await
        .map_err(|e| tonic::Status::aborted(format!("couldn't get verified heights: {e}")))?;

    let Some(mut verified_heights) = verified_heights else {
        return Err(tonic::Status::not_found(
            "couldn't find verified heights for client: {client_id}",
        ));
    };

    verified_heights.heights.sort();
    if verified_heights.heights.is_empty() {
        return Err(tonic::Status::not_found(
            "verified heights for client were empty: {client_id}",
        ));
    }

    Ok(verified_heights.heights.pop().expect("must be non-empty"))
}
