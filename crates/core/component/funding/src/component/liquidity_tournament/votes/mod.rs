use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sdk_asset::asset;
use penumbra_sdk_keys::Address;

use crate::component::state_key;

#[async_trait]
pub trait StateWriteExt: StateWrite {
    // Keeping this as returning a result to not have to touch other code if it changes to return an error.
    async fn tally(
        &mut self,
        epoch: u64,
        asset: asset::Id,
        power: u64,
        voter: &Address,
    ) -> anyhow::Result<()> {
        self.nonverifiable_put_raw(
            state_key::lqt::v1::votes::receipt(epoch, asset, power, voter).to_vec(),
            Vec::default(),
        );
        Ok(())
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
