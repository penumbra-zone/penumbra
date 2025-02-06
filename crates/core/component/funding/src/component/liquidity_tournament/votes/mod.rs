use std::pin::Pin;

use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use futures::{Stream, StreamExt as _, TryStreamExt as _};
use penumbra_sdk_asset::asset;
use penumbra_sdk_keys::Address;
use penumbra_sdk_proto::{DomainType, StateReadProto as _, StateWriteProto as _};
use penumbra_sdk_sct::component::source::SourceContext as _;
use penumbra_sdk_stake::IdentityKey;
use penumbra_sdk_txhash::TransactionId;

use crate::component::state_key;

async fn get_total(state: impl StateRead, epoch_index: u64) -> u64 {
    state
        .nonverifiable_get_proto(&state_key::lqt::v1::votes::total::key(epoch_index))
        .await
        .expect("should be able to read u64")
        .unwrap_or_default()
}

fn put_total(mut state: impl StateWrite, epoch_index: u64, total: u64) {
    state.nonverifiable_put_proto(state_key::lqt::v1::votes::total::key(epoch_index), total)
}

async fn get_asset_total(state: impl StateRead, epoch_index: u64, asset: asset::Id) -> u64 {
    state
        .nonverifiable_get_proto(&state_key::lqt::v1::votes::by_asset::total::key(
            epoch_index,
            asset,
        ))
        .await
        .expect("should be able to read u64")
        .unwrap_or_default()
}

fn put_asset_total(mut state: impl StateWrite, epoch_index: u64, asset: asset::Id, total: u64) {
    state.nonverifiable_put_proto(
        state_key::lqt::v1::votes::by_asset::total::key(epoch_index, asset),
        total,
    )
}

fn put_ranked_asset(mut state: impl StateWrite, epoch_index: u64, power: u64, asset: asset::Id) {
    state.nonverifiable_put_raw(
        state_key::lqt::v1::votes::by_asset::ranked::key(epoch_index, power, asset),
        Vec::new(),
    )
}

fn delete_ranked_asset(mut state: impl StateWrite, epoch_index: u64, power: u64, asset: asset::Id) {
    state.nonverifiable_delete(state_key::lqt::v1::votes::by_asset::ranked::key(
        epoch_index,
        power,
        asset,
    ));
}

async fn get_voter_total(
    state: impl StateRead,
    epoch_index: u64,
    asset: asset::Id,
    voter: &Address,
    validator: &IdentityKey,
) -> u64 {
    state
        .nonverifiable_get_proto(&state_key::lqt::v1::votes::by_voter::total::key(
            epoch_index,
            asset,
            voter,
            validator,
        ))
        .await
        .expect("should be able to read u64")
        .unwrap_or_default()
}

fn put_voter_total(
    mut state: impl StateWrite,
    epoch_index: u64,
    asset: asset::Id,
    voter: &Address,
    total: u64,
    validator: &IdentityKey,
) {
    state.nonverifiable_put_proto(
        state_key::lqt::v1::votes::by_voter::total::key(epoch_index, asset, voter, validator),
        total,
    )
}

fn put_ranked_voter(
    mut state: impl StateWrite,
    epoch_index: u64,
    asset: asset::Id,
    power: u64,
    voter: &Address,
    validator: &IdentityKey,
    tx: TransactionId,
) {
    state.nonverifiable_put(
        state_key::lqt::v1::votes::by_voter::ranked::key(
            epoch_index,
            asset,
            power,
            voter,
            validator,
        ),
        tx,
    )
}

fn delete_ranked_voter(
    mut state: impl StateWrite,
    epoch_index: u64,
    asset: asset::Id,
    power: u64,
    voter: &Address,
    validator: &IdentityKey,
) {
    state.nonverifiable_delete(state_key::lqt::v1::votes::by_voter::ranked::key(
        epoch_index,
        asset,
        power,
        voter,
        validator,
    ));
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
    ) -> Pin<
        Box<
            dyn Stream<Item = anyhow::Result<((u64, Address, IdentityKey), TransactionId)>>
                + Send
                + 'static,
        >,
    > {
        self
        .nonverifiable_prefix_proto(&state_key::lqt::v1::votes::by_voter::ranked::prefix(
            epoch,
            asset
        ))
        .map_ok(|(k, v) : (_, <TransactionId as DomainType>::Proto)| {
            let key = state_key::lqt::v1::votes::by_voter::ranked::parse_key(&k)
                .expect("should be able to parse lqt::v1::votes::by_voter::ranked key");
            let value = v.try_into().expect("should be able to parse transaction hash from lqt::v1::votes::by_voter::ranked_key");
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
    async fn tally(
        &mut self,
        epoch: u64,
        asset: asset::Id,
        power: u64,
        validator: IdentityKey,
        voter: &Address,
    ) {
        let tx = self.get_current_source().expect("source should be set");
        // Increment total.
        {
            let current = get_total(&self, epoch).await;
            put_total(&mut *self, epoch, add_power(current, power))
        }
        // Adjust asset indices.
        {
            let current = get_asset_total(&self, epoch, asset).await;
            delete_ranked_asset(&mut *self, epoch, current, asset);
            let new = add_power(current, power);
            put_asset_total(&mut *self, epoch, asset, new);
            put_ranked_asset(&mut *self, epoch, new, asset);
        }
        // Adjust voter indices.
        {
            let current = get_voter_total(&self, epoch, asset, voter, &validator).await;
            delete_ranked_voter(&mut *self, epoch, asset, current, voter, &validator);
            let new = add_power(current, power);
            put_voter_total(&mut *self, epoch, asset, voter, new, &validator);
            put_ranked_voter(&mut self, epoch, asset, new, voter, &validator, tx);
        }
    }
}

impl<T: StateWrite> StateWriteExt for T {}
