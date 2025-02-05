use cnidarium::Storage;
use penumbra_sdk_proto::core::component::funding::v1::{
    self as pb, funding_service_server::FundingService,
};
use penumbra_sdk_sct::{component::clock::EpochRead, Nullifier};

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
    async fn lqt_check_nullifier(
        &self,
        request: tonic::Request<pb::LqtCheckNullifierRequest>,
    ) -> Result<tonic::Response<pb::LqtCheckNullifierResponse>, tonic::Status> {
        // Retrieve latest state snapshot.
        let state = self.storage.latest_snapshot();

        let req_inner = request.into_inner();
        let nullifier_proto = req_inner
            .nullifier
            .ok_or_else(|| tonic::Status::invalid_argument("missing nullifier"))?;

        // Proto to domain type conversion.
        let nullifier: Nullifier = nullifier_proto
            .try_into()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid nullifier: {e}")))?;

        // If `epoch_index` is omitted (defaults to zero in protobuf), query the current epoch;
        // Otherwise use the provided epoch.
        let epoch_index = if req_inner.epoch_index == 0 {
            let current_epoch = state.get_current_epoch().await.map_err(|e| {
                tonic::Status::internal(format!("failed to retrieve current epoch: {e}"))
            })?;
            current_epoch.index
        } else {
            req_inner.epoch_index
        };

        // Perform a state nullifier lookup.
        let maybe_tx_id = state
            .get_lqt_spent_nullifier_by_epoch(nullifier, epoch_index)
            .await;

        if let Some(tx_id) = maybe_tx_id {
            // Nullifier was spent and user has already voted.
            Ok(tonic::Response::new(pb::LqtCheckNullifierResponse {
                transaction: Some(tx_id.into()),
                already_voted: true,
                epoch_index,
            }))
        } else {
            // Nullifier was not spent and user has not voted yet.
            Ok(tonic::Response::new(pb::LqtCheckNullifierResponse {
                transaction: None,
                already_voted: false,
                epoch_index,
            }))
        }
    }
}
