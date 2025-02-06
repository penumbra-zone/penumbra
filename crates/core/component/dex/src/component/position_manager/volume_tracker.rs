use cnidarium::StateWrite;
use penumbra_sdk_asset::{asset, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::StateWriteProto;
use tracing::instrument;

use crate::component::lqt::LqtRead;
use crate::event;
use crate::lp::position::{self, Position};
use crate::state_key::lqt;
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
        if prev_state.is_none() {
            tracing::debug!(?position_id, "newly opened position, skipping volume index");
            return;
        }

        // Short-circuit if the position is transitioning to a non-open state.
        // This might miss some volume updates, but is more conservative on state-flow.
        if !matches!(new_state.state, position::State::Opened) {
            tracing::debug!(
                ?position_id,
                "new state is not `Opened`, skipping volume index"
            );
            return;
        }

        let trading_pair = new_state.phi.pair.clone();

        // We want to track the **outflow** of staking tokens from the position.
        // This means that we track the amount of staking tokens that have left the position.
        // We do this by comparing the previous and new reserves of the staking token.
        // We **DO NOT** want to track the volume of the other asset denominated in staking tokens.
        let prev_state = prev_state.as_ref().expect("the previous state exists");
        let prev_balance = prev_state
            .reserves_for(*STAKING_TOKEN_ASSET_ID)
            .expect("the staking token is in the pair");

        let new_balance = new_state
            .reserves_for(*STAKING_TOKEN_ASSET_ID)
            .expect("the staking token is in the pair");

        // We track the *outflow* of the staking token.
        // "How much inventory has left the position?"
        let staking_token_outflow = prev_balance.saturating_sub(&new_balance);

        // We lookup the previous volume index entry.
        let old_volume = self.get_volume_for_position(position_id).await;
        let new_volume = old_volume.saturating_add(&staking_token_outflow);

        // Grab the ambient epoch index.
        let epoch_index = self
            .get_current_epoch()
            .await
            .expect("epoch is always set")
            .index;

        // Find the trading pair asset that is not the staking token.
        let other_asset = if trading_pair.asset_1() == *STAKING_TOKEN_ASSET_ID {
            trading_pair.asset_2()
        } else {
            trading_pair.asset_1()
        };

        self.record_proto(
            event::EventLqtPositionVolume {
                epoch_index,
                position_id: position_id.clone(),
                asset_id: other_asset,
                volume: staking_token_outflow,
                total_volume: new_volume,
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
