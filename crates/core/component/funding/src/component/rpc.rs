use cnidarium::Storage;
use penumbra_sdk_proto::core::component::funding::v1::{
    self as pb, funding_service_server::FundingService,
};
use penumbra_sdk_sct::Nullifier;

use super::liquidity_tournament::nullifier::NullifierRead;

pub struct Server {
    storage: Storage,
}

impl Server {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl FundingService for Server {
    async fn lqt_current_epoch_voted(
        &self,
        request: tonic::Request<pb::LqtCurrentEpochVotedRequest>,
    ) -> Result<tonic::Response<pb::LqtCurrentEpochVotedResponse>, tonic::Status> {
        // Retrieve latest state snapshot.
        let state = self.storage.latest_snapshot();

        let req_nullifier = request.into_inner();
        let nullifier_proto = req_nullifier
            .nullifier
            .ok_or_else(|| tonic::Status::invalid_argument("missing nullifier"))?;

        // Proto to domain type conversion.
        let nullifier: Nullifier = nullifier_proto
            .try_into()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid nullifier: {e}")))?;

        // Perform a state nullifier lookup.
        let maybe_tx_id = state.get_lqt_spent_nullifier(nullifier).await;

        if let Some(tx_id) = maybe_tx_id {
            // Nullifier was spent and user has already voted.
            Ok(tonic::Response::new(pb::LqtCurrentEpochVotedResponse {
                tx_id: Some(tx_id.into()),
                already_voted: true,
            }))
        } else {
            // Nullifier was not spent and user has not voted yet.
            Ok(tonic::Response::new(pb::LqtCurrentEpochVotedResponse {
                tx_id: None,
                already_voted: false,
            }))
        }
    }
}
