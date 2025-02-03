use std::pin::Pin;

use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use futures::{Stream, StreamExt as _};
use penumbra_sdk_asset::asset;
use penumbra_sdk_keys::Address;

use crate::component::state_key;

#[allow(dead_code)]
#[async_trait]
pub trait StateReadExt: StateRead {
    async fn vote_receipts(
        &self,
        epoch: u64,
    ) -> Pin<Box<dyn Stream<Item = anyhow::Result<(asset::Id, u64, Vec<u8>)>> + Send + 'static>>
    {
        let prefix = state_key::lqt::v1::votes::prefix(epoch);
        self.nonverifiable_prefix_raw(&prefix)
            .map(|res| res.and_then(|(k, _)| state_key::lqt::v1::votes::parse_receipt(&k)))
            .boxed()
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

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
