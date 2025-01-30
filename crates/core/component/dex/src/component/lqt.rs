use crate::lp::position;
use crate::state_key::lqt;
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateRead;
use futures::StreamExt;
use penumbra_sdk_asset::asset;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::StateReadProto;
use penumbra_sdk_sct::component::clock::EpochRead;
use std::pin::Pin;

/// Provides public read access to LQT data.
#[async_trait]
pub trait LqtRead: StateRead {
    /// Returns the cumulative volume of staking token for a trading pair.
    /// This is the sum of the outflows of the staking token from all positions in the pair.
    ///
    /// Default to zero if no volume is found.
    async fn get_volume_for_pair(&self, asset: asset::Id) -> Amount {
        let epoch = self.get_current_epoch().await.expect("epoch is always set");
        let key = lqt::v1::pair::lookup::volume_by_pair(epoch.index, asset);
        let value = self.nonverifiable_get(&key).await.unwrap_or_default();
        value.unwrap_or_default()
    }

    /// Returns the cumulative volume of staking token for a given position id.
    /// This is the sum of the outflows of the staking token from the position.
    ///
    /// Default to zero if no volume is found.
    async fn get_volume_for_position(&self, position_id: &position::Id) -> Amount {
        let epoch = self.get_current_epoch().await.expect("epoch is always set");
        let key = lqt::v1::lp::lookup::volume_by_position(epoch.index, position_id);
        let value = self.nonverifiable_get(&key).await.unwrap_or_default();
        value.unwrap_or_default()
    }

    /// Returns a stream of position ids sorted by descending volume.
    /// The volume is the sum of the outflows of the staking token from the position.
    fn positions_by_volume_stream(
        &self,
        epoch_index: u64,
        asset_id: asset::Id,
    ) -> Result<
        Pin<
            Box<
                dyn futures::Stream<Item = Result<(asset::Id, position::Id, Amount)>>
                    + Send
                    + 'static,
            >,
        >,
    > {
        let key = lqt::v1::lp::by_volume::prefix_with_asset(epoch_index, &asset_id);
        Ok(self
            .nonverifiable_prefix_raw(&key)
            .map(|res| {
                res.map(|(raw_entry, _)| {
                    let (asset, volume, position_id) =
                        lqt::v1::lp::by_volume::parse_key(&raw_entry).expect("internal invariant failed: failed to parse state key for lqt::v1::lp::by_volume");
                    (asset, position_id, volume)
                })
            })
            .boxed())
    }
}

impl<T: StateRead + ?Sized> LqtRead for T {}
