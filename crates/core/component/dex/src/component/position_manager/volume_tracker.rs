use cnidarium::StateWrite;
use penumbra_sdk_asset::{asset, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::StateWriteProto;
use tracing::instrument;

use crate::component::lqt::LqtRead;
use crate::lp::position::{self, Position};
use crate::state_key::lqt;
use crate::{event, DirectedTradingPair};
use async_trait::async_trait;
use penumbra_sdk_proto::DomainType;
use penumbra_sdk_sct::component::clock::EpochRead;

#[async_trait]
pub(crate) trait PositionVolumeTracker: StateWrite {
    async fn update_volume_index(
        &mut self,
        position_id: &position::Id,
        prev_state: &Option<Position>,
        new_state: &Position,
    ) {
        // We only index the volume for staking token pairs.
        if !new_state.phi.matches_input(*STAKING_TOKEN_ASSET_ID) {
            return;
        }

        // Or if the position has existed before.
        let Some(prev_state) = prev_state else {
            tracing::debug!(?position_id, "newly opened position, skipping volume index");
            return;
        };

        // Short-circuit if the position is transitioning to a non-open state.
        // This might miss some volume updates, but is more conservative on state-flow.
        if !matches!(new_state.state, position::State::Opened) {
            tracing::debug!(
                ?position_id,
                "new state is not `Opened`, skipping volume index"
            );
            return;
        }

        let pair = new_state.phi.pair;
        let other_asset = if pair.asset_1 != *STAKING_TOKEN_ASSET_ID {
            pair.asset_1
        } else {
            pair.asset_2
        };
        // Get the flows with the first asset being UM, and the other asset
        let flows = new_state
            .flows(&prev_state)
            .redirect(DirectedTradingPair {
                start: *STAKING_TOKEN_ASSET_ID,
                end: other_asset,
            })
            .expect("the staking token is in the pair");
        // We want to track the **outflow** of staking tokens from the position.
        // This means that we track the amount of staking tokens that have left the position.
        // We do this by comparing the previous and new reserves of the staking token.
        // We **DO NOT** want to track the volume of the other asset denominated in staking tokens.
        // We track the *outflow* of the staking token.
        // "How much inventory has left the position?"
        let staking_token_outflow = flows.lambda_1();

        // We lookup the previous volume index entry.
        let old_volume = self.get_volume_for_position(position_id).await;
        let new_volume = old_volume.saturating_add(&staking_token_outflow);

        // Grab the ambient epoch index.
        let epoch_index = self
            .get_current_epoch()
            .await
            .expect("epoch is always set")
            .index;

        self.record_proto(
            event::EventLqtPositionVolume {
                epoch_index,
                position_id: position_id.clone(),
                asset_id: other_asset,
                volume: staking_token_outflow,
                total_volume: new_volume,
                staking_token_in: flows.delta_1(),
                asset_in: flows.delta_2(),
                staking_fees: flows.fee_1(),
                asset_fees: flows.fee_2(),
            }
            .to_proto(),
        );

        self.update_volume(
            epoch_index,
            &other_asset,
            position_id,
            old_volume,
            new_volume,
        )
    }
}

impl<T: StateWrite + ?Sized> PositionVolumeTracker for T {}

trait Inner: StateWrite {
    #[instrument(skip(self))]
    fn update_volume(
        &mut self,
        epoch_index: u64,
        asset_id: &asset::Id,
        position_id: &position::Id,
        old_volume: Amount,
        new_volume: Amount,
    ) {
        // First, update the lookup index with the new volume.
        let lookup_key = lqt::v1::lp::lookup::volume_by_position(epoch_index, position_id);
        use penumbra_sdk_proto::StateWriteProto;
        self.nonverifiable_put(lookup_key.to_vec(), new_volume);

        // Then, update the sorted index:
        let old_index_key =
            lqt::v1::lp::by_volume::key(epoch_index, asset_id, position_id, old_volume);
        // Delete the old key:
        self.nonverifiable_delete(old_index_key.to_vec());
        // Store the new one:
        let new_index_key =
            lqt::v1::lp::by_volume::key(epoch_index, asset_id, position_id, new_volume);
        self.nonverifiable_put(new_index_key.to_vec(), new_volume);
    }
}

impl<T: StateWrite + ?Sized> Inner for T {}
