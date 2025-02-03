use std::pin::Pin;

use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use futures::{Stream, StreamExt as _, TryStreamExt as _};
use penumbra_sdk_asset::asset;
use penumbra_sdk_keys::Address;
use penumbra_sdk_sct::component::source::SourceContext as _;
use penumbra_sdk_txhash::TransactionId;

use crate::component::state_key;

async fn get_u64(state: impl StateRead, key: &[u8]) -> u64 {
    state
        .nonverifiable_get_raw(key)
        .await
        .ok()
        .and_then(|x| Some(u64::from_le_bytes(TryInto::<[u8; 8]>::try_into(x?).ok()?)))
        .unwrap_or_default()
}

async fn put_u64(mut state: impl StateWrite, key: Vec<u8>, value: u64) {
    state.nonverifiable_put_raw(key, value.to_le_bytes().to_vec());
}

async fn get_total(state: impl StateRead, epoch_index: u64) -> u64 {
    get_u64(state, &state_key::lqt::v1::votes::total::key(epoch_index)).await
}

async fn put_total(state: impl StateWrite, epoch_index: u64, total: u64) {
    put_u64(
        state,
        state_key::lqt::v1::votes::total::key(epoch_index).to_vec(),
        total,
    )
    .await
}

async fn get_asset_total(state: impl StateRead, epoch_index: u64, asset: asset::Id) -> u64 {
    get_u64(
        state,
        &state_key::lqt::v1::votes::by_asset::total::key(epoch_index, asset),
    )
    .await
}

async fn put_asset_total(state: impl StateWrite, epoch_index: u64, asset: asset::Id, total: u64) {
    put_u64(
        state,
        state_key::lqt::v1::votes::by_asset::total::key(epoch_index, asset).to_vec(),
        total,
    )
    .await
}

fn put_ranked_asset(mut state: impl StateWrite, epoch_index: u64, power: u64, asset: asset::Id) {
    state.nonverifiable_put_raw(
        state_key::lqt::v1::votes::by_asset::ranked::key(epoch_index, power, asset).to_vec(),
        Vec::new(),
    )
}

fn delete_ranked_asset(mut state: impl StateWrite, epoch_index: u64, power: u64, asset: asset::Id) {
    state.nonverifiable_delete(
        state_key::lqt::v1::votes::by_asset::ranked::key(epoch_index, power, asset).to_vec(),
    );
}

async fn get_voter_total(
    state: impl StateRead,
    epoch_index: u64,
    asset: asset::Id,
    voter: &Address,
) -> u64 {
    get_u64(
        state,
        &state_key::lqt::v1::votes::by_voter::total::key(epoch_index, asset, voter),
    )
    .await
}

async fn put_voter_total(
    state: impl StateWrite,
    epoch_index: u64,
    asset: asset::Id,
    voter: &Address,
    total: u64,
) {
    put_u64(
        state,
        state_key::lqt::v1::votes::by_voter::total::key(epoch_index, asset, voter).to_vec(),
        total,
    )
    .await
}

fn put_ranked_voter(
    mut state: impl StateWrite,
    epoch_index: u64,
    asset: asset::Id,
    power: u64,
    voter: &Address,
    tx: TransactionId,
) {
    state.nonverifiable_put_raw(
        state_key::lqt::v1::votes::by_voter::ranked::key(epoch_index, asset, power, voter).to_vec(),
        tx.0.to_vec(),
    )
}

fn delete_ranked_voter(
    mut state: impl StateWrite,
    epoch_index: u64,
    asset: asset::Id,
    power: u64,
    voter: &Address,
) {
    state.nonverifiable_delete(
        state_key::lqt::v1::votes::by_voter::ranked::key(epoch_index, asset, power, voter).to_vec(),
    );
}

#[async_trait]
pub trait StateReadExt: StateRead + Sized {
    async fn total_votes(&self, epoch: u64) -> u64 {
        get_total(self, epoch).await
    }

    fn ranked_assets(
        &self,
        epoch: u64,
    ) -> Pin<Box<dyn Stream<Item = anyhow::Result<(u64, asset::Id)>> + Send + 'static>> {
        self.nonverifiable_prefix_raw(&state_key::lqt::v1::votes::by_asset::ranked::prefix(epoch))
            .map_ok(|(k, _)| {
                state_key::lqt::v1::votes::by_asset::ranked::parse_key(&k)
                    .expect("should be able to parse lqt::v1::votes::by_asset::ranked key")
            })
            .boxed()
    }

    fn ranked_voters(
        &self,
        epoch: u64,
        asset: asset::Id,
    ) -> Pin<Box<dyn Stream<Item = anyhow::Result<((u64, Address), TransactionId)>> + Send + 'static>>
    {
        self
        .nonverifiable_prefix_raw(&state_key::lqt::v1::votes::by_voter::ranked::prefix(
            epoch,
            asset
        ))
        .map_ok(|(k, v)| {
            let key = state_key::lqt::v1::votes::by_voter::ranked::parse_key(&k)
                .expect("should be able to parse lqt::v1::votes::by_voter::ranked key");
            let value = TransactionId(v.try_into().expect("should be able to parse transaction hash from lqt::v1::votes::by_voter::ranked_key"));
            (key, value)
        }).boxed()
    }
}

impl<T: StateRead> StateReadExt for T {}

fn add_power(total: u64, power: u64) -> u64 {
    total.checked_add(power).expect("LQT voting power overflow")
}

#[async_trait]
pub trait StateWriteExt: StateWrite + Sized {
    // Keeping this as returning a result to not have to touch other code if it changes to return an error.
    async fn tally(&mut self, epoch: u64, asset: asset::Id, power: u64, voter: &Address) {
        let tx = self.get_current_source().expect("source should be set");
        // Increment total.
        {
            let current = get_total(&self, epoch).await;
            put_total(&mut *self, epoch, add_power(current, power)).await;
        }
        // Adjust asset indices.
        {
            let current = get_asset_total(&self, epoch, asset).await;
            delete_ranked_asset(&mut *self, epoch, current, asset);
            let new = add_power(current, power);
            put_asset_total(&mut *self, epoch, asset, new).await;
            put_ranked_asset(&mut *self, epoch, new, asset);
        }
        // Adjust voter indices.
        {
            let current = get_voter_total(&self, epoch, asset, voter).await;
            delete_ranked_voter(&mut *self, epoch, asset, current, voter);
            let new = add_power(current, power);
            put_voter_total(&mut *self, epoch, asset, voter, new).await;
            put_ranked_voter(&mut self, epoch, asset, new, voter, tx);
        }
    }
}

impl<T: StateWrite> StateWriteExt for T {}
