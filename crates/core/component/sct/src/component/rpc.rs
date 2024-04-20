use cnidarium::Storage;
use penumbra_proto::core::component::sct::v1::query_service_server::QueryService;
use penumbra_proto::core::component::sct::v1::{
    AnchorByHeightRequest, AnchorByHeightResponse, EpochByHeightRequest, EpochByHeightResponse,
};
use tonic::Status;
use tracing::instrument;

use super::clock::EpochRead;
use super::tree::SctRead;

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
    async fn epoch_by_height(
        &self,
        request: tonic::Request<EpochByHeightRequest>,
    ) -> Result<tonic::Response<EpochByHeightResponse>, Status> {
        let state = self.storage.latest_snapshot();

        let epoch = state
            .get_epoch_by_height(request.get_ref().height)
            .await
            .map_err(|e| tonic::Status::unknown(format!("could not get epoch for height: {e}")))?;

        Ok(tonic::Response::new(EpochByHeightResponse {
            epoch: Some(epoch.into()),
        }))
    }

    #[instrument(skip(self, request))]
    async fn anchor_by_height(
        &self,
        request: tonic::Request<AnchorByHeightRequest>,
    ) -> Result<tonic::Response<AnchorByHeightResponse>, Status> {
        let state = self.storage.latest_snapshot();

        let height = request.get_ref().height;
        let anchor = state.get_anchor_by_height(height).await.map_err(|e| {
            tonic::Status::unknown(format!("could not get anchor for height {height}: {e}"))
        })?;

        Ok(tonic::Response::new(AnchorByHeightResponse {
            anchor: anchor.map(Into::into),
        }))
    }
}
