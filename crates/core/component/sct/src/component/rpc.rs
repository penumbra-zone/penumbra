use penumbra_chain::component::StateReadExt as _;
use penumbra_proto::core::component::sct::v1alpha1::{
    query_service_server::QueryService, TransactionByNoteRequest, TransactionByNoteResponse,
};
use penumbra_storage::Storage;
use tonic::Status;
use tracing::instrument;

use super::StateReadExt;

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
    async fn transaction_by_note(
        &self,
        request: tonic::Request<TransactionByNoteRequest>,
    ) -> Result<tonic::Response<TransactionByNoteResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let cm = request
            .into_inner()
            .note_commitment
            .ok_or_else(|| Status::invalid_argument("empty message"))?
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid commitment"))?;
        let source = state
            .note_source(cm)
            .await
            .map_err(|e| Status::unavailable(format!("error getting note source: {e}")))?
            .ok_or_else(|| Status::not_found("note commitment not found"))?;
        tracing::debug!(?cm, ?source);

        Ok(tonic::Response::new(TransactionByNoteResponse {
            note_source: Some(source.into()),
        }))
    }
}
