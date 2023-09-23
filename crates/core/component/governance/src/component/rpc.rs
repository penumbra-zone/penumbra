use std::pin::Pin;

use futures::{StreamExt, TryStreamExt};
use penumbra_chain::component::StateReadExt as _;
use penumbra_proto::{
    core::component::governance::v1alpha1::{
        query_service_server::QueryService, ProposalInfoRequest, ProposalInfoResponse,
        ProposalRateDataRequest, ProposalRateDataResponse,
    },
    StateReadProto,
};
use penumbra_stake::rate::RateData;
use penumbra_storage::Storage;
use tonic::Status;
use tracing::instrument;

use crate::state_key;

use super::StateReadExt;

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
    async fn proposal_info(
        &self,
        request: tonic::Request<ProposalInfoRequest>,
    ) -> Result<tonic::Response<ProposalInfoResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let proposal_id = request.into_inner().proposal_id;

        let start_block_height = state
            .proposal_voting_start(proposal_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or_else(|| tonic::Status::unknown(format!("proposal {proposal_id} not found")))?;

        let start_position = state
            .proposal_voting_start_position(proposal_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or_else(|| tonic::Status::unknown(format!("proposal {proposal_id} not found")))?;

        Ok(tonic::Response::new(ProposalInfoResponse {
            start_block_height,
            start_position: start_position.into(),
        }))
    }

    type ProposalRateDataStream = Pin<
        Box<dyn futures::Stream<Item = Result<ProposalRateDataResponse, tonic::Status>> + Send>,
    >;

    #[instrument(skip(self, request))]
    async fn proposal_rate_data(
        &self,
        request: tonic::Request<ProposalRateDataRequest>,
    ) -> Result<tonic::Response<Self::ProposalRateDataStream>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let proposal_id = request.into_inner().proposal_id;

        let s = state.prefix(&state_key::all_rate_data_at_proposal_start(proposal_id));
        Ok(tonic::Response::new(
            s.map_ok(|i: (String, RateData)| {
                let (_key, rate_data) = i;
                ProposalRateDataResponse {
                    rate_data: Some(rate_data.into()),
                }
            })
            .map_err(|e: anyhow::Error| {
                tonic::Status::unavailable(format!("error getting prefix value from storage: {e}"))
            })
            // TODO: how do we instrument a Stream
            //.instrument(Span::current())
            .boxed(),
        ))
    }
}
