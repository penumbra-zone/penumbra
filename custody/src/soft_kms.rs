//! A basic software key management system that stores keys in memory but
//! presents as an asynchronous signer.

use penumbra_proto::custody::v1alpha1::{self as pb, AuthorizeResponse};
use penumbra_transaction::AuthorizationData;
use rand_core::OsRng;
use tonic::{async_trait, Request, Response, Status};

use crate::{policy::Policy, AuthorizeRequest};

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

    /// Attempt to authorize the requested [`TransactionPlan`](penumbra_transaction::plan::TransactionPlan).
    #[tracing::instrument(skip(self, request), name = "softhsm_sign")]
    pub fn sign(&self, request: &AuthorizeRequest) -> anyhow::Result<AuthorizationData> {
        tracing::debug!(?request.plan);

        for policy in &self.config.auth_policy {
            policy.check(request)?;
        }

        Ok(request.plan.authorize(OsRng, &self.config.spend_key))
    }
}

#[async_trait]
impl pb::custody_protocol_service_server::CustodyProtocolService for SoftKms {
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
            .map_err(|e| Status::unauthenticated(format!("{:#}", e)))?;

        let authorization_response = AuthorizeResponse {
            data: Some(authorization_data.into()),
        };

        Ok(Response::new(authorization_response))
    }
}
