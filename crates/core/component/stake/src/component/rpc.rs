use std::pin::Pin;

use async_stream::try_stream;
use cnidarium::Storage;
use futures::{StreamExt, TryStreamExt};
use penumbra_chain::component::StateReadExt as _;
use penumbra_proto::{
    core::component::stake::v1alpha1::{
        query_service_server::QueryService, CurrentValidatorRateRequest,
        CurrentValidatorRateResponse, ValidatorInfoRequest, ValidatorInfoResponse,
        ValidatorPenaltyRequest, ValidatorPenaltyResponse, ValidatorStatusRequest,
        ValidatorStatusResponse,
    },
    DomainType,
};
use tonic::Status;
use tracing::instrument;

use super::StateReadExt;
use crate::validator;

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
    type ValidatorInfoStream =
        Pin<Box<dyn futures::Stream<Item = Result<ValidatorInfoResponse, tonic::Status>> + Send>>;

    #[instrument(skip(self, request), fields(show_inactive = request.get_ref().show_inactive))]
    async fn validator_info(
        &self,
        request: tonic::Request<ValidatorInfoRequest>,
    ) -> Result<tonic::Response<Self::ValidatorInfoStream>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| {
                tonic::Status::unknown(format!(
                    "failed to validate chain id during validator info request: {e}"
                ))
            })?;

        let validators = state
            .validator_definitions() // TODO(erwan): think through a UX for defined validators. Then we can remove `validator_list` entirely.
            .await
            .map_err(|e| tonic::Status::unavailable(format!("error listing validators: {e}")))?;

        let show_inactive = request.get_ref().show_inactive;
        let s = try_stream! {
            for v in validators {
                let info = state.validator_info(&v.identity_key)
                    .await?
                    .expect("known validator must be present");
                // Slashed and inactive validators are not shown by default.
                if !show_inactive && info.status.state != validator::State::Active {
                    continue;
                }
                yield info.to_proto();
            }
        };

        Ok(tonic::Response::new(
            s.map_ok(|info| ValidatorInfoResponse {
                validator_info: Some(info),
            })
            .map_err(|e: anyhow::Error| {
                tonic::Status::unavailable(format!("error getting validator info: {e}"))
            })
            // TODO: how do we instrument a Stream
            //.instrument(Span::current())
            .boxed(),
        ))
    }

    #[instrument(skip(self, request))]
    async fn validator_status(
        &self,
        request: tonic::Request<ValidatorStatusRequest>,
    ) -> Result<tonic::Response<ValidatorStatusResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;

        let id = request
            .into_inner()
            .identity_key
            .ok_or_else(|| Status::invalid_argument("missing identity key"))?
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid identity key"))?;

        let status = state
            .validator_status(&id)
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
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;

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
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let identity_key = request
            .into_inner()
            .identity_key
            .ok_or_else(|| tonic::Status::invalid_argument("empty message"))?
            .try_into()
            .map_err(|_| tonic::Status::invalid_argument("invalid identity key"))?;

        let rate_data = state
            .current_validator_rate(&identity_key)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        match rate_data {
            Some(r) => Ok(tonic::Response::new(CurrentValidatorRateResponse {
                data: Some(r.into()),
            })),
            None => Err(Status::not_found("current validator rate not found")),
        }
    }
}
