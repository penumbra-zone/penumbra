//! RPC query service for token factory.

use cnidarium::Storage;
use penumbra_sdk_proto::core::component::token_factory::v1::{
    query_service_server::QueryService, AllTokensRequest, AllTokensResponse, TokenMetadataRequest,
    TokenMetadataResponse,
};
use tonic::Status;
use tracing::instrument;

use crate::TokenFactoryId;

use super::StateReadExt;

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
    async fn token_metadata(
        &self,
        request: tonic::Request<TokenMetadataRequest>,
    ) -> Result<tonic::Response<TokenMetadataResponse>, Status> {
        let state = self.storage.latest_snapshot();
        let request = request.into_inner();

        let token_id_bytes: [u8; 32] = request
            .token_id
            .try_into()
            .map_err(|_| Status::invalid_argument("token_id must be exactly 32 bytes"))?;

        let id = TokenFactoryId::from(token_id_bytes);

        let metadata = state
            .token_factory_metadata(&id)
            .await
            .map_err(|e| Status::internal(format!("failed to query metadata: {e}")))?;

        Ok(tonic::Response::new(TokenMetadataResponse {
            metadata: metadata.map(Into::into),
        }))
    }

    #[instrument(skip(self, _request))]
    async fn all_tokens(
        &self,
        _request: tonic::Request<AllTokensRequest>,
    ) -> Result<tonic::Response<AllTokensResponse>, Status> {
        // TODO: implement pagination over all tokens
        // For now, return empty - listing all tokens requires iterating
        // over the metadata prefix which is expensive
        Ok(tonic::Response::new(AllTokensResponse {
            tokens: vec![],
            next_cursor: vec![],
        }))
    }
}
