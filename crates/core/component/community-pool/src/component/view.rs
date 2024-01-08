use std::{collections::BTreeMap, str::FromStr};

use anyhow::Result;
use async_trait::async_trait;

use cnidarium::{StateRead, StateWrite};
use futures::{StreamExt, TryStreamExt};
use penumbra_asset::{asset, Value};
use penumbra_num::Amount;
use penumbra_proto::{StateReadProto, StateWriteProto};

use crate::params::CommunityPoolParameters;

use super::state_key;

#[async_trait]
pub trait StateReadExt: StateRead {
    /// Indicates if the Community Pool parameters have been updated in this block.
    fn community_pool_params_updated(&self) -> bool {
        self.object_get::<()>(state_key::community_pool_params_updated())
            .is_some()
    }

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
        // Note that the Community Pool params have been updated:
        self.object_put(state_key::community_pool_params_updated(), ());

        // Change the Community Pool parameters:
        self.put(state_key::community_pool_params().into(), params)
    }

    async fn community_pool_deposit(&mut self, value: Value) -> Result<()> {
        let key = state_key::balance_for_asset(value.asset_id);
        let current = self.get(&key).await?.unwrap_or_else(|| Amount::from(0u64));
        self.put(key, current + value.amount);
        Ok(())
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
