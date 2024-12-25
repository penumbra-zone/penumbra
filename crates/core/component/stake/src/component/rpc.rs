use std::pin::Pin;

use cnidarium::Storage;
use futures::StreamExt;
use penumbra_sdk_proto::{
    core::component::stake::v1::{
        query_service_server::QueryService, CurrentValidatorRateRequest,
        CurrentValidatorRateResponse, GetValidatorInfoRequest, GetValidatorInfoResponse,
        ValidatorInfoRequest, ValidatorInfoResponse, ValidatorPenaltyRequest,
        ValidatorPenaltyResponse, ValidatorStatusRequest, ValidatorStatusResponse,
        ValidatorUptimeRequest, ValidatorUptimeResponse,
    },
    DomainType,
};
use tap::{TapFallible, TapOptional};
use tonic::Status;
use tracing::{error_span, instrument, Instrument, Span};

use super::{validator_handler::ValidatorDataRead, ConsensusIndexRead, SlashingData};
use crate::validator::{Info, State};

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
    async fn get_validator_info(
        &self,
        request: tonic::Request<GetValidatorInfoRequest>,
    ) -> Result<tonic::Response<GetValidatorInfoResponse>, tonic::Status> {
        let state = self.storage.latest_snapshot();
        let GetValidatorInfoRequest { identity_key } = request.into_inner();

        // Take the identity key from the inbound request.
        let identity_key = identity_key
            .ok_or_else(|| Status::invalid_argument("an identity key must be provided"))?
            .try_into()
            .tap_err(|error| tracing::debug!(?error, "request contained an invalid identity key"))
            .map_err(|_| Status::invalid_argument("invalid identity key"))?;

        // Look up the information for the validator with the given identity key.
        let info = state
            .get_validator_info(&identity_key)
            .await
            .tap_err(|error| tracing::error!(?error, %identity_key, "failed to get validator info"))
            .map_err(|_| Status::invalid_argument("failed to get validator info"))?
            .tap_none(|| tracing::debug!(%identity_key, "validator info was not found"))
            .ok_or_else(|| Status::not_found("validator info was not found"))?;

        // Construct the outbound response.
        let resp = GetValidatorInfoResponse {
            validator_info: Some(info.to_proto()),
        };

        Ok(tonic::Response::new(resp))
    }

    type ValidatorInfoStream =
        Pin<Box<dyn futures::Stream<Item = Result<ValidatorInfoResponse, tonic::Status>> + Send>>;

    #[instrument(skip(self, request), fields(show_inactive = request.get_ref().show_inactive))]
    async fn validator_info(
        &self,
        request: tonic::Request<ValidatorInfoRequest>,
    ) -> Result<tonic::Response<Self::ValidatorInfoStream>, Status> {
        use futures::TryStreamExt;

        // Get the latest snapshot from the backing storage, and determine whether or not the
        // response should include inactive validator definitions.
        let snapshot = self.storage.latest_snapshot();
        let ValidatorInfoRequest { show_inactive } = request.into_inner();

        // Returns `true` if we should include a validator in the outbound response.
        let filter_inactive = move |info: &Info| {
            let should = match info.status.state {
                State::Active => true,
                _ if show_inactive => true, // Include other validators if the request asked us to.
                _ => false,                 // Otherwise, skip this entry.
            };
            futures::future::ready(should)
        };

        // Converts information about a validator into a RPC response.
        let to_resp = |info: Info| {
            let validator_info = Some(info.to_proto());
            ValidatorInfoResponse { validator_info }
        };

        // Creates a span that follows from the current tracing context.
        let make_span = |identity_key| -> Span {
            let span = error_span!("fetching validator information", %identity_key);
            let current = Span::current();
            span.follows_from(current);
            span
        };

        // Get a stream of identity keys corresponding to validators in the consensus set.
        let consensus_set = snapshot
            .consensus_set_stream()
            .map_err(|e| format!("error getting consensus set: {e}"))
            .map_err(Status::unavailable)?;

        // Adapt the stream of identity keys into a stream of validator information.
        // Define a span indicating that the spawned future follows from the current context.
        let validators = async_stream::try_stream! {
            for await identity_key in consensus_set {
                let identity_key = identity_key?;
                let span = make_span(identity_key);
                yield snapshot
                    .get_validator_info(&identity_key)
                    .instrument(span)
                    .await?
                    .expect("known validator must be present");
            }
        };

        // Construct the outbound response.
        let stream = validators
            .try_filter(filter_inactive)
            .map_ok(to_resp)
            .map_err(|e: anyhow::Error| format!("error getting validator info: {e}"))
            .map_err(Status::unavailable)
            .into_stream()
            .boxed();

        Ok(tonic::Response::new(stream))
    }

    #[instrument(skip(self, request))]
    async fn validator_status(
        &self,
        request: tonic::Request<ValidatorStatusRequest>,
    ) -> Result<tonic::Response<ValidatorStatusResponse>, Status> {
        let state = self.storage.latest_snapshot();

        let id = request
            .into_inner()
            .identity_key
            .ok_or_else(|| Status::invalid_argument("missing identity key"))?
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid identity key"))?;

        let status = state
            .get_validator_status(&id)
            .await
            .map_err(|e| Status::unavailable(format!("error getting validator status: {e}")))?
            .ok_or_else(|| Status::not_found("validator not found"))?;

        Ok(tonic::Response::new(ValidatorStatusResponse {
            status: Some(status.into()),
        }))
    }

    #[instrument(skip(self, request))]
    async fn validator_penalty(
        &self,
        request: tonic::Request<ValidatorPenaltyRequest>,
    ) -> Result<tonic::Response<ValidatorPenaltyResponse>, Status> {
        let state = self.storage.latest_snapshot();
        let request = request.into_inner();
        let id = request
            .identity_key
            .ok_or_else(|| Status::invalid_argument("missing identity key"))?
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid identity key"))?;

        let penalty = state
            .compounded_penalty_over_range(&id, request.start_epoch_index, request.end_epoch_index)
            .await
            .map_err(|e| Status::unavailable(format!("error getting validator penalty: {e}")))?;

        Ok(tonic::Response::new(ValidatorPenaltyResponse {
            penalty: Some(penalty.into()),
        }))
    }

    #[instrument(skip(self, request))]
    async fn current_validator_rate(
        &self,
        request: tonic::Request<CurrentValidatorRateRequest>,
    ) -> Result<tonic::Response<CurrentValidatorRateResponse>, Status> {
        let state = self.storage.latest_snapshot();
        let identity_key = request
            .into_inner()
            .identity_key
            .ok_or_else(|| tonic::Status::invalid_argument("empty message"))?
            .try_into()
            .map_err(|_| tonic::Status::invalid_argument("invalid identity key"))?;

        let rate_data = state
            .get_validator_rate(&identity_key)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        match rate_data {
            Some(r) => Ok(tonic::Response::new(CurrentValidatorRateResponse {
                data: Some(r.into()),
            })),
            None => Err(Status::not_found("current validator rate not found")),
        }
    }

    #[instrument(skip(self, request))]
    async fn validator_uptime(
        &self,
        request: tonic::Request<ValidatorUptimeRequest>,
    ) -> Result<tonic::Response<ValidatorUptimeResponse>, Status> {
        let state = self.storage.latest_snapshot();
        let identity_key = request
            .into_inner()
            .identity_key
            .ok_or_else(|| tonic::Status::invalid_argument("empty message"))?
            .try_into()
            .map_err(|_| tonic::Status::invalid_argument("invalid identity key"))?;

        let uptime_data = state
            .get_validator_uptime(&identity_key)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        match uptime_data {
            Some(u) => Ok(tonic::Response::new(ValidatorUptimeResponse {
                uptime: Some(u.into()),
            })),
            None => Err(Status::not_found("validator uptime not found")),
        }
    }
}
