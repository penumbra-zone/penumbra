use cnidarium::Storage;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::core::component::distributions::v1::{
    self as pb, distributions_service_server::DistributionsService,
};
use penumbra_sdk_sct::component::clock::EpochRead;

use crate::component::StateReadExt;

pub struct Server {
    storage: Storage,
}

impl Server {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl DistributionsService for Server {
    async fn current_lqt_pool_size(
        &self,
        _request: tonic::Request<pb::CurrentLqtPoolSizeRequest>,
    ) -> Result<tonic::Response<pb::CurrentLqtPoolSizeResponse>, tonic::Status> {
        // Retrieve latest state snapshot.
        let state = self.storage.latest_snapshot();

        let current_block_height = state.get_block_height().await.map_err(|e| {
            tonic::Status::internal(format!("failed to get current block height: {}", e))
        })?;
        let current_epoch = state
            .get_current_epoch()
            .await
            .map_err(|e| tonic::Status::internal(format!("failed to get current epoch: {}", e)))?;
        let epoch_length = current_block_height
            .checked_sub(current_epoch.start_height)
            .unwrap_or_else(|| panic!("epoch start height is greater than current block height (epoch_start={}, current_height={}", current_epoch.start_height, current_block_height));

        let lqt_block_reward_rate = state
            .get_distributions_params()
            .await
            .map_err(|e| {
                tonic::Status::internal(format!("failed to get distributions parameters: {}", e))
            })?
            .liquidity_tournament_incentive_per_block as u64;

        let current_lqt_pool_size = lqt_block_reward_rate
            .checked_mul(epoch_length as u64)
            .expect("infallible unless issuance is pathological");

        Ok(tonic::Response::new(pb::CurrentLqtPoolSizeResponse {
            epoch_index: current_epoch.index,
            pool_size: Some(Amount::from(current_lqt_pool_size).into()),
        }))
    }

    async fn lqt_pool_size_by_epoch(
        &self,
        request: tonic::Request<pb::LqtPoolSizeByEpochRequest>,
    ) -> Result<tonic::Response<pb::LqtPoolSizeByEpochResponse>, tonic::Status> {
        // Retrieve latest state snapshot.
        let state = self.storage.latest_snapshot();
        let epoch_index = request.into_inner().epoch;
        let amount = state
            .get_lqt_reward_issuance_for_epoch(epoch_index)
            .await
            .ok_or_else(|| {
                tonic::Status::not_found(format!(
                    "failed to retrieve LQT issuance for epoch {} from non-verifiable storage",
                    epoch_index,
                ))
            })?;

        Ok(tonic::Response::new(pb::LqtPoolSizeByEpochResponse {
            epoch_index,
            pool_size: Some(amount.into()),
        }))
    }
}
