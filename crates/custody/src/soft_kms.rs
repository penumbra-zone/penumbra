//! A basic software key management system that stores keys in memory but
//! presents as an asynchronous signer.

use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_sdk_proto::{
    core::component::{
        governance::v1::ValidatorVoteBody as ProtoValidatorVoteBody,
        stake::v1::Validator as ProtoValidator,
    },
    custody::v1::{self as pb, AuthorizeResponse},
    Message as _,
};
use penumbra_sdk_transaction::AuthorizationData;
use rand_core::OsRng;
use tonic::{async_trait, Request, Response, Status};

use crate::{
    policy::Policy, AuthorizeRequest, AuthorizeValidatorDefinitionRequest,
    AuthorizeValidatorVoteRequest,
};

mod config;

pub use config::Config;

/// A basic software key management system that stores keys in memory but
/// presents as an asynchronous signer.
pub struct SoftKms {
    config: Config,
}

impl SoftKms {
    /// Initialize with the given [`Config`].
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Attempt to authorize the requested [`TransactionPlan`](penumbra_sdk_transaction::TransactionPlan).
    #[tracing::instrument(skip(self, request), name = "softhsm_sign")]
    pub fn sign(&self, request: &AuthorizeRequest) -> anyhow::Result<AuthorizationData> {
        tracing::debug!(?request.plan);

        for policy in &self.config.auth_policy {
            policy.check_transaction(request)?;
        }

        Ok(request.plan.authorize(OsRng, &self.config.spend_key)?)
    }

    /// Attempt to authorize the requested validator definition.
    #[tracing::instrument(skip(self, request), name = "softhsm_sign_validator_definition")]
    pub fn sign_validator_definition(
        &self,
        request: &AuthorizeValidatorDefinitionRequest,
    ) -> anyhow::Result<Signature<SpendAuth>> {
        tracing::debug!(?request.validator_definition);

        for policy in &self.config.auth_policy {
            policy.check_validator_definition(request)?;
        }

        let protobuf_serialized: ProtoValidator = request.validator_definition.clone().into();
        let validator_definition_bytes = protobuf_serialized.encode_to_vec();

        Ok(self
            .config
            .spend_key
            .spend_auth_key()
            .sign(OsRng, &validator_definition_bytes))
    }

    /// Attempt to authorize the requested validator vote.
    #[tracing::instrument(skip(self, request), name = "softhsm_sign_validator_vote")]
    pub fn sign_validator_vote(
        &self,
        request: &AuthorizeValidatorVoteRequest,
    ) -> anyhow::Result<Signature<SpendAuth>> {
        tracing::debug!(?request.validator_vote);

        for policy in &self.config.auth_policy {
            policy.check_validator_vote(request)?;
        }

        let protobuf_serialized: ProtoValidatorVoteBody = request.validator_vote.clone().into();
        let validator_vote_bytes = protobuf_serialized.encode_to_vec();

        Ok(self
            .config
            .spend_key
            .spend_auth_key()
            .sign(OsRng, &validator_vote_bytes))
    }
}

#[async_trait]
impl pb::custody_service_server::CustodyService for SoftKms {
    async fn authorize(
        &self,
        request: Request<pb::AuthorizeRequest>,
    ) -> Result<Response<AuthorizeResponse>, Status> {
        let request = request
            .into_inner()
            .try_into()
            .map_err(|e: anyhow::Error| Status::invalid_argument(e.to_string()))?;

        let authorization_data = self
            .sign(&request)
            .map_err(|e| Status::unauthenticated(format!("{e:#}")))?;

        let authorization_response = AuthorizeResponse {
            data: Some(authorization_data.into()),
        };

        Ok(Response::new(authorization_response))
    }

    async fn authorize_validator_definition(
        &self,
        request: Request<pb::AuthorizeValidatorDefinitionRequest>,
    ) -> Result<Response<pb::AuthorizeValidatorDefinitionResponse>, Status> {
        let request = request
            .into_inner()
            .try_into()
            .map_err(|e: anyhow::Error| Status::invalid_argument(e.to_string()))?;

        let validator_definition_auth = self
            .sign_validator_definition(&request)
            .map_err(|e| Status::unauthenticated(format!("{e:#}")))?;

        let authorization_response = pb::AuthorizeValidatorDefinitionResponse {
            validator_definition_auth: Some(validator_definition_auth.into()),
        };

        Ok(Response::new(authorization_response))
    }

    async fn authorize_validator_vote(
        &self,
        request: Request<pb::AuthorizeValidatorVoteRequest>,
    ) -> Result<Response<pb::AuthorizeValidatorVoteResponse>, Status> {
        let request = request
            .into_inner()
            .try_into()
            .map_err(|e: anyhow::Error| Status::invalid_argument(e.to_string()))?;

        let validator_vote_auth = self
            .sign_validator_vote(&request)
            .map_err(|e| Status::unauthenticated(format!("{e:#}")))?;

        let authorization_response = pb::AuthorizeValidatorVoteResponse {
            validator_vote_auth: Some(validator_vote_auth.into()),
        };

        Ok(Response::new(authorization_response))
    }

    async fn export_full_viewing_key(
        &self,
        _request: Request<pb::ExportFullViewingKeyRequest>,
    ) -> Result<Response<pb::ExportFullViewingKeyResponse>, Status> {
        Ok(Response::new(pb::ExportFullViewingKeyResponse {
            full_viewing_key: Some(self.config.spend_key.full_viewing_key().clone().into()),
        }))
    }

    async fn confirm_address(
        &self,
        request: Request<pb::ConfirmAddressRequest>,
    ) -> Result<Response<pb::ConfirmAddressResponse>, Status> {
        let address_index = request
            .into_inner()
            .address_index
            .ok_or_else(|| {
                Status::invalid_argument("missing address index in confirm address request")
            })?
            .try_into()
            .map_err(|e| {
                Status::invalid_argument(format!(
                    "invalid address index in confirm address request: {e:#}"
                ))
            })?;

        let (address, _dtk) = self
            .config
            .spend_key
            .full_viewing_key()
            .payment_address(address_index);

        Ok(Response::new(pb::ConfirmAddressResponse {
            address: Some(address.into()),
        }))
    }
}
