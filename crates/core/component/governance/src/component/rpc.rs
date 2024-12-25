use std::pin::Pin;
use std::str::FromStr;

use anyhow::Context;
use async_stream::try_stream;
use cnidarium::Storage;
use futures::{StreamExt, TryStreamExt};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::core::component::governance::v1::AllTalliedDelegatorVotesForProposalRequest;
use penumbra_sdk_proto::core::component::governance::v1::AllTalliedDelegatorVotesForProposalResponse;
use penumbra_sdk_proto::core::component::governance::v1::NextProposalIdRequest;
use penumbra_sdk_proto::core::component::governance::v1::NextProposalIdResponse;
use penumbra_sdk_proto::core::component::governance::v1::VotingPowerAtProposalStartRequest;
use penumbra_sdk_proto::core::component::governance::v1::VotingPowerAtProposalStartResponse;
use penumbra_sdk_proto::{
    core::component::governance::v1::{
        query_service_server::QueryService, ProposalDataRequest, ProposalDataResponse,
        ProposalInfoRequest, ProposalInfoResponse, ProposalListRequest, ProposalListResponse,
        ProposalRateDataRequest, ProposalRateDataResponse, ValidatorVotesRequest,
        ValidatorVotesResponse,
    },
    StateReadProto,
};
use penumbra_sdk_stake::rate::RateData;
use penumbra_sdk_stake::IdentityKey;
use tonic::Status;
use tracing::instrument;

use crate::state_key;
use crate::Tally;
use crate::Vote;

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
        let proposal_id = request.into_inner().proposal_id;

        let start_block_height = state
            .proposal_voting_start(proposal_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or_else(|| tonic::Status::not_found(format!("proposal {proposal_id} not found")))?;

        let start_position = state
            .proposal_voting_start_position(proposal_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or_else(|| tonic::Status::not_found(format!("proposal {proposal_id} not found")))?;

        Ok(tonic::Response::new(ProposalInfoResponse {
            start_block_height,
            start_position: start_position.into(),
        }))
    }

    #[instrument(skip(self, _request))]
    async fn next_proposal_id(
        &self,
        _request: tonic::Request<NextProposalIdRequest>,
    ) -> Result<tonic::Response<NextProposalIdResponse>, Status> {
        let state = self.storage.latest_snapshot();

        let next_proposal_id: u64 = state
            .get_proto(state_key::next_proposal_id())
            .await
            .map_err(|e| tonic::Status::internal(format!("unable to fetch next proposal id: {e}")))?
            .ok_or_else(|| tonic::Status::not_found("there are no proposals yet".to_string()))?;

        Ok(tonic::Response::new(NextProposalIdResponse {
            next_proposal_id,
        }))
    }

    #[instrument(skip(self, request))]
    async fn proposal_data(
        &self,
        request: tonic::Request<ProposalDataRequest>,
    ) -> Result<tonic::Response<ProposalDataResponse>, Status> {
        let state = self.storage.latest_snapshot();
        let proposal_id = request.into_inner().proposal_id;

        let start_block_height = state
            .proposal_voting_start(proposal_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or_else(|| tonic::Status::not_found(format!("proposal {proposal_id} not found")))?;

        let end_block_height = state
            .proposal_voting_end(proposal_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or_else(|| tonic::Status::not_found(format!("proposal {proposal_id} not found")))?;

        let start_position = state
            .proposal_voting_start_position(proposal_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or_else(|| tonic::Status::not_found(format!("proposal {proposal_id} not found")))?;

        let proposal = state
            .proposal_definition(proposal_id)
            .await
            .map_err(|e| tonic::Status::internal(format!("unable to fetch proposal: {e}")))?
            .ok_or_else(|| {
                tonic::Status::not_found(format!("proposal {} not found", proposal_id))
            })?;

        let proposal_state = state
            .proposal_state(proposal_id)
            .await
            .map_err(|e| tonic::Status::internal(format!("unable to fetch proposal state: {e}")))?
            .ok_or_else(|| {
                tonic::Status::not_found(format!("proposal {} state not found", proposal_id))
            })?;

        let proposal_deposit_amount: Amount = state
            .get(&state_key::proposal_deposit_amount(proposal_id))
            .await
            .map_err(|e| {
                tonic::Status::internal(format!("unable to fetch proposal deposit amount: {e}"))
            })?
            .ok_or_else(|| {
                tonic::Status::not_found(format!(
                    "deposit amount for proposal {} was not found",
                    proposal_id
                ))
            })?;

        Ok(tonic::Response::new(ProposalDataResponse {
            start_block_height,
            end_block_height,
            start_position: start_position.into(),
            state: Some(proposal_state.into()),
            proposal: Some(proposal.into()),
            proposal_deposit_amount: Some(proposal_deposit_amount.into()),
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

        let proposal_id_list: Vec<u64> = if request.into_inner().inactive {
            let next = state.next_proposal_id().await.map_err(|e| {
                tonic::Status::internal(format!("unable to get next proposal id: {e}"))
            })?;

            (0..next).collect()
        } else {
            state
                .unfinished_proposals()
                .await
                .map_err(|e| {
                    tonic::Status::internal(format!("unable to fetch unfinished proposals: {e}"))
                })?
                .into_iter()
                .collect::<Vec<_>>()
        };

        let s = try_stream! {
            for proposal_id in proposal_id_list {
            let proposal = state
                .proposal_definition(proposal_id)
                .await
                .map_err(|e| tonic::Status::internal(format!("unable to fetch proposal: {e}")))?
                .ok_or_else(|| {
                    tonic::Status::not_found(format!("proposal {} not found", proposal_id))
                })?;

            let proposal_state = state
                .proposal_state(proposal_id)
                .await
                .map_err(|e| tonic::Status::internal(format!("unable to fetch proposal state: {e}")))?
                .ok_or_else(|| {
                    tonic::Status::not_found(format!("proposal {} state not found", proposal_id))
                })?;

            let proposal_voting_start_position = state
                .proposal_voting_start_position(proposal_id)
                .await
                .map_err(|e| {
                    tonic::Status::internal(format!(
                        "unable to fetch proposal voting start position: {e}"
                    ))
                })?
                .ok_or_else(|| {
                    tonic::Status::not_found(format!(
                        "voting start position for proposal {} not found",
                        proposal_id
                    ))
                })?;

            let start_block_height = state
                .proposal_voting_start(proposal_id)
                .await
                .map_err(|e| {
                    tonic::Status::internal(format!(
                        "unable to fetch proposal voting start block: {e}"
                    ))
                })?
                .ok_or_else(|| {
                    tonic::Status::not_found(format!(
                        "voting start block for proposal {} not found",
                        proposal_id
                    ))
                })?;

            let end_block_height = state
                .proposal_voting_end(proposal_id)
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?
                .ok_or_else(|| tonic::Status::not_found(format!("proposal {proposal_id} not found")))?;

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

        let proposal_id = request.into_inner().proposal_id;

        let s = state
            .prefix::<Vote>(&state_key::all_validator_votes_for_proposal(proposal_id))
            .and_then(|r| async move {
                Ok((
                    IdentityKey::from_str(r.0.rsplit('/').next().context("invalid key")?)?,
                    r.1,
                ))
            })
            .map_ok(|i: (IdentityKey, Vote)| ValidatorVotesResponse {
                vote: Some(i.1.into()),
                identity_key: Some(i.0.into()),
            });

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

    #[instrument(skip(self, request))]
    async fn voting_power_at_proposal_start(
        &self,
        request: tonic::Request<VotingPowerAtProposalStartRequest>,
    ) -> Result<tonic::Response<VotingPowerAtProposalStartResponse>, Status> {
        let state = self.storage.latest_snapshot();
        let request = request.into_inner();
        let proposal_id = request.proposal_id;
        if let Some(identity_key) = request.identity_key {
            // If the query is for a specific validator, return their voting power at the start of
            // the proposal
            let identity_key = identity_key.try_into().map_err(|_| {
                tonic::Status::invalid_argument(
                    "identity key in request was bad protobuf".to_string(),
                )
            })?;

            let voting_power = state
                .get_proto::<u64>(&state_key::voting_power_at_proposal_start(
                    proposal_id,
                    identity_key,
                ))
                .await
                .map_err(|e| tonic::Status::internal(format!("error accessing storage: {}", e)))?;

            if voting_power.is_none() {
                return Err(tonic::Status::not_found(format!(
                    "validator did not exist at proposal creation: {}",
                    identity_key
                )));
            }

            Ok(tonic::Response::new(VotingPowerAtProposalStartResponse {
                voting_power: voting_power.expect("voting power should be set"),
            }))
        } else {
            // If the query is for the total voting power at the start of the proposal, return that
            let total_voting_power = state
                .total_voting_power_at_proposal_start(proposal_id)
                .await
                .map_err(|e| tonic::Status::internal(format!("error accessing storage: {}", e)))?;

            Ok(tonic::Response::new(VotingPowerAtProposalStartResponse {
                voting_power: total_voting_power,
            }))
        }
    }

    type AllTalliedDelegatorVotesForProposalStream = Pin<
        Box<
            dyn futures::Stream<
                    Item = Result<AllTalliedDelegatorVotesForProposalResponse, tonic::Status>,
                > + Send,
        >,
    >;

    #[instrument(skip(self, request))]
    async fn all_tallied_delegator_votes_for_proposal(
        &self,
        request: tonic::Request<AllTalliedDelegatorVotesForProposalRequest>,
    ) -> Result<tonic::Response<Self::AllTalliedDelegatorVotesForProposalStream>, Status> {
        let state = self.storage.latest_snapshot();
        let proposal_id = request.into_inner().proposal_id;

        let s = state.prefix::<Tally>(&state_key::all_tallied_delegator_votes_for_proposal(
            proposal_id,
        ));
        Ok(tonic::Response::new(
            s.and_then(|r| async move {
                Ok((
                    IdentityKey::from_str(r.0.rsplit('/').next().context("invalid key")?)?,
                    r.1,
                ))
            })
            .map_err(|e| {
                tonic::Status::internal(format!("unable to retrieve tallied delegator votes: {e}"))
            })
            .map_ok(
                |i: (IdentityKey, Tally)| AllTalliedDelegatorVotesForProposalResponse {
                    tally: Some(i.1.into()),
                    identity_key: Some(i.0.into()),
                },
            )
            // TODO: how do we instrument a Stream
            //.instrument(Span::current())
            .boxed(),
        ))
    }
}
