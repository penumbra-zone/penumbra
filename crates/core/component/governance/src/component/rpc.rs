use std::pin::Pin;

use anyhow::Context;
use anyhow::Error;
use async_stream::try_stream;
use async_stream::AsyncStream;
use core::future::Future;
use futures::{StreamExt, TryStreamExt};
use penumbra_chain::component::StateReadExt as _;
use penumbra_proto::{
    core::component::governance::v1alpha1::{
        query_service_server::QueryService, ProposalDataRequest, ProposalDataResponse,
        ProposalInfoRequest, ProposalInfoResponse, ProposalListRequest, ProposalListResponse,
        ProposalRateDataRequest, ProposalRateDataResponse, ValidatorVotesRequest,
        ValidatorVotesResponse,
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

    #[instrument(skip(self, request))]
    async fn proposal_data(
        &self,
        request: tonic::Request<ProposalDataRequest>,
    ) -> Result<tonic::Response<ProposalDataResponse>, Status> {
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

        let end_block_height = state
            .proposal_voting_end(proposal_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or_else(|| tonic::Status::unknown(format!("proposal {proposal_id} not found")))?;

        let start_position = state
            .proposal_voting_start_position(proposal_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or_else(|| tonic::Status::unknown(format!("proposal {proposal_id} not found")))?;

        let proposal = state
            .proposal_definition(proposal_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("unable to fetch proposal: {e}")))?
            .ok_or_else(|| tonic::Status::unknown(format!("proposal {} not found", proposal_id)))?;

        let proposal_state = state
            .proposal_state(proposal_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("unable to fetch proposal state: {e}")))?
            .ok_or_else(|| {
                tonic::Status::unknown(format!("proposal {} state not found", proposal_id))
            })?;

        Ok(tonic::Response::new(ProposalDataResponse {
            start_block_height,
            end_block_height,
            start_position: start_position.into(),
            state: Some(proposal_state.into()),
            proposal: Some(proposal.into()),
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

    type ProposalListStream =
        Pin<Box<dyn futures::Stream<Item = Result<ProposalListResponse, tonic::Status>> + Send>>;

    #[instrument(skip(self, request))]
    async fn proposal_list(
        &self,
        request: tonic::Request<ProposalListRequest>,
    ) -> Result<tonic::Response<Self::ProposalListStream>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;

        let proposal_id_list: Vec<u64> = if request.into_inner().inactive {
            let next = state.next_proposal_id().await.map_err(|e| {
                tonic::Status::unknown(format!("unable to get next proposal id: {e}"))
            })?;

            (0..next).collect()
        } else {
            state
                .unfinished_proposals()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(format!("unable to fetch unfinished proposals: {e}"))
                })?
                .into_iter()
                .collect::<Vec<_>>()
        };

        let s = try_stream! {
            for proposal_id in proposal_id_list {
            let proposal = state
                .proposal_definition(proposal_id)
                .await
                .map_err(|e| tonic::Status::unknown(format!("unable to fetch proposal: {e}")))?
                .ok_or_else(|| {
                    tonic::Status::unknown(format!("proposal {} not found", proposal_id))
                })?;

            let proposal_state = state
                .proposal_state(proposal_id)
                .await
                .map_err(|e| tonic::Status::unknown(format!("unable to fetch proposal state: {e}")))?
                .ok_or_else(|| {
                    tonic::Status::unknown(format!("proposal {} state not found", proposal_id))
                })?;

            let proposal_voting_start_position = state
                .proposal_voting_start_position(proposal_id)
                .await
                .map_err(|e| {
                    tonic::Status::unknown(format!(
                        "unable to fetch proposal voting start position: {e}"
                    ))
                })?
                .ok_or_else(|| {
                    tonic::Status::unknown(format!(
                        "voting start position for proposal {} not found",
                        proposal_id
                    ))
                })?;

            let start_block_height = state
                .proposal_voting_start(proposal_id)
                .await
                .map_err(|e| {
                    tonic::Status::unknown(format!(
                        "unable to fetch proposal voting start block: {e}"
                    ))
                })?
                .ok_or_else(|| {
                    tonic::Status::unknown(format!(
                        "voting start block for proposal {} not found",
                        proposal_id
                    ))
                })?;

            let end_block_height = state
                .proposal_voting_end(proposal_id)
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?
                .ok_or_else(|| tonic::Status::unknown(format!("proposal {proposal_id} not found")))?;

            yield ProposalListResponse {
                proposal: Some(proposal.into()),
                start_block_height,
                end_block_height,
                start_position: proposal_voting_start_position.into(),
                state: Some(proposal_state.into()),
            }
        }};

        Ok(tonic::Response::new(
            s.map_err(|e: anyhow::Error| {
                tonic::Status::unavailable(format!(
                    "error getting position value from storage: {e}"
                ))
            })
            // TODO: how do we instrument a Stream
            //.instrument(Span::current())
            .boxed(),
        ))
    }

    type ValidatorVotesStream =
        Pin<Box<dyn futures::Stream<Item = Result<ValidatorVotesResponse, tonic::Status>> + Send>>;

    #[instrument(skip(self, request))]
    async fn validator_votes(
        &self,
        request: tonic::Request<ValidatorVotesRequest>,
    ) -> Result<tonic::Response<Self::ValidatorVotesStream>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;

        let proposal_id = request.into_inner().proposal_id;

        // TODO: the validator_votes method makes and consumes a stream, then we
        // reconstruct a stream here -- we should instead just pass the results
        // from storage in a streaming manner here, but the type checker got tricky.
        let validator_votes = state
            .validator_votes(proposal_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("unable to fetch validator votes: {e}")))?;

        let s = try_stream! {
            for (identity_key, vote) in validator_votes {
                yield ValidatorVotesResponse {
                    identity_key: Some(identity_key.into()),
                    vote: Some(vote.into())
                }
            }
        };

        Ok(tonic::Response::new(
            s.map_err(|e: anyhow::Error| {
                tonic::Status::unavailable(format!(
                    "error getting validator votes from storage: {e}"
                ))
            })
            // TODO: how do we instrument a Stream
            //.instrument(Span::current())
            .boxed(),
        ))
    }
}
