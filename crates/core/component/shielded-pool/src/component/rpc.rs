use penumbra_asset::asset;
use penumbra_chain::component::StateReadExt as _;
use penumbra_proto::core::component::shielded_pool::v1alpha1::{
    query_service_server::QueryService, DenomMetadataByIdRequest, DenomMetadataByIdResponse,
};
use penumbra_storage::Storage;
use tonic::Status;
use tracing::instrument;

use super::SupplyRead;

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
    async fn denom_metadata_by_id(
        &self,
        request: tonic::Request<DenomMetadataByIdRequest>,
    ) -> Result<tonic::Response<DenomMetadataByIdResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;

        let request = request.into_inner();
        let id: asset::Id = request
            .asset_id
            .ok_or_else(|| Status::invalid_argument("missing asset_id"))?
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("could not parse asset_id: {e}")))?;

        let denom = state
            .denom_by_asset(&id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let rsp = match denom {
            Some(denom) => {
                tracing::debug!(?id, ?denom, "found denom");
                DenomMetadataByIdResponse {
                    denom_metadata: Some(denom.into()),
                }
            }
            None => {
                tracing::debug!(?id, "unknown asset id");
                Default::default()
            }
        };

        Ok(tonic::Response::new(rsp))
    }
}
