use cnidarium::Storage;
use penumbra_proto::core::component::sct::v1::query_service_server::QueryService;
use penumbra_proto::core::component::sct::v1::{EpochByHeightRequest, EpochByHeightResponse};
use tonic::Status;
use tracing::instrument;

use super::clock::EpochRead;

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
}
