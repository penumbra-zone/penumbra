use std::pin::Pin;

use cnidarium::Storage;
use penumbra_sdk_asset::asset::{self};
use penumbra_sdk_proto::core::component::shielded_pool::v1::{
    query_service_server::QueryService, AssetMetadataByIdRequest, AssetMetadataByIdResponse,
    AssetMetadataByIdsRequest, AssetMetadataByIdsResponse,
};

use tonic::Status;
use tracing::instrument;

use super::AssetRegistryRead;

mod bank_query;
mod transfer_query;

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
    type AssetMetadataByIdsStream = Pin<
        Box<dyn futures::Stream<Item = Result<AssetMetadataByIdsResponse, tonic::Status>> + Send>,
    >;

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

        let denom_metadata = state.denom_metadata_by_asset(&id).await;

        let rsp = match denom_metadata {
            Some(denom_metadata) => {
                tracing::debug!(?id, ?denom_metadata, "found denom metadata");
                AssetMetadataByIdResponse {
                    denom_metadata: Some(denom_metadata.into()),
                }
            }
            None => {
                tracing::debug!(?id, "unknown asset id");
                Default::default()
            }
        };

        Ok(tonic::Response::new(rsp))
    }

    async fn asset_metadata_by_ids(
        &self,
        _request: tonic::Request<AssetMetadataByIdsRequest>,
    ) -> Result<tonic::Response<Self::AssetMetadataByIdsStream>, tonic::Status> {
        unimplemented!("asset_metadata_by_ids not yet implemented")
    }
}
