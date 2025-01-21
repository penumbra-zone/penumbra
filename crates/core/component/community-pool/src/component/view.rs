use std::{collections::BTreeMap, str::FromStr};

use anyhow::Result;
use async_trait::async_trait;

use cnidarium::{StateRead, StateWrite};
use futures::{StreamExt, TryStreamExt};
use penumbra_sdk_asset::{asset, Value};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};

use crate::params::CommunityPoolParameters;

use super::state_key;

#[async_trait]
pub trait StateReadExt: StateRead {
    /// Gets the Community Pool parameters from the JMT.
    async fn get_community_pool_params(&self) -> Result<CommunityPoolParameters> {
        self.get(state_key::community_pool_params())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Missing CommunityPoolParameters"))
    }

    async fn community_pool_asset_balance(&self, asset_id: asset::Id) -> Result<Amount> {
        Ok(self
            .get(&state_key::balance_for_asset(asset_id))
            .await?
            .unwrap_or_else(|| Amount::from(0u64)))
    }

    async fn community_pool_balance(&self) -> Result<BTreeMap<asset::Id, Amount>> {
        let prefix = state_key::all_assets_balance();
        self.prefix(prefix)
            .map(|result| {
                let (key, amount) = result?;
                let asset_id = key.rsplit('/').next().expect("key is well-formed");
                let asset_id = asset::Id::from_str(asset_id)?;
                Ok((asset_id, amount))
            })
            .try_collect()
            .await
    }
}

impl<T> StateReadExt for T where T: StateRead + ?Sized {}

#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Writes the provided Community Pool parameters to the JMT.
    fn put_community_pool_params(&mut self, params: CommunityPoolParameters) {
        self.put(state_key::community_pool_params().into(), params)
    }

    async fn community_pool_deposit(&mut self, value: Value) {
        let key = state_key::balance_for_asset(value.asset_id);
        let current = self
            .get(&key)
            .await
            .expect("no deserialization errors")
            .unwrap_or_else(|| Amount::from(0u64));
        self.put(key, current + value.amount);
    }

    async fn community_pool_withdraw(&mut self, value: Value) -> Result<()> {
        let key = state_key::balance_for_asset(value.asset_id);
        let current = self.get(&key).await?.unwrap_or_else(|| Amount::from(0u64));
        if let Some(remaining) = u128::from(current).checked_sub(u128::from(value.amount)) {
            if remaining > 0 {
                self.put(key, Amount::from(remaining));
            } else {
                self.delete(key);
            }
        } else {
            anyhow::bail!(
                "insufficient balance to withdraw {} of asset ID {} from the Community Pool",
                value.amount,
                value.asset_id
            );
        }
        Ok(())
    }
}

impl<T> StateWriteExt for T where T: StateWrite + ?Sized {}
