use std::collections::BTreeMap;

use penumbra_crypto::keys::{AccountID, SpendKey};
use penumbra_proto::custody::v1alpha1::{self as pb, AuthorizeResponse};
use penumbra_transaction::AuthorizationData;
use rand_core::OsRng;
use tonic::{async_trait, Request, Response, Status};

use crate::AuthorizeRequest;

/// A basic "SoftHSM" that stores keys in memory but presents as an asynchronous signer.
pub struct SoftHSM {
    /// Store keys in a BTreeMap so we can identify them by account ID.
    keys: BTreeMap<AccountID, SpendKey>,
}

impl SoftHSM {
    /// Initialize the SoftHSM with the given keys.
    pub fn new(keys: Vec<SpendKey>) -> Self {
        Self {
            keys: keys
                .into_iter()
                .map(|sk| (sk.full_viewing_key().hash(), sk))
                .collect(),
        }
    }

    #[tracing::instrument(skip(self, request), name = "softhsm_sign")]
    pub fn sign(&self, request: &AuthorizeRequest) -> anyhow::Result<AuthorizationData> {
        let sk = self.keys.get(&request.account_id).ok_or_else(|| {
            anyhow::anyhow!("Missing signing key for account ID {}", request.account_id)
        })?;

        tracing::debug!(?request.plan);

        Ok(request.plan.authorize(OsRng, sk))
    }
}

#[async_trait]
impl pb::custody_protocol_service_server::CustodyProtocolService for SoftHSM {
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
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let authorization_response = AuthorizeResponse {
            data: Some(authorization_data.into()),
        };

        Ok(Response::new(authorization_response))
    }
}
