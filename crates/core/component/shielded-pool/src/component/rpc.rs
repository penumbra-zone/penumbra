use cnidarium::Storage;
use penumbra_asset::asset;
use penumbra_proto::core::component::shielded_pool::v1::{
    query_service_server::QueryService, AssetMetadataByIdRequest, AssetMetadataByIdResponse,
};

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
    async fn asset_metadata_by_id(
        &self,
        request: tonic::Request<AssetMetadataByIdRequest>,
    ) -> Result<tonic::Response<AssetMetadataByIdResponse>, Status> {
        let state = self.storage.latest_snapshot();

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
                AssetMetadataByIdResponse {
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
