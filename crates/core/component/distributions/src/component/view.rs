use async_trait::async_trait;

use crate::{component::state_key, params::DistributionsParameters};
use anyhow::Result;
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};

#[async_trait]
pub trait StateReadExt: StateRead {
    /// Gets the distributions module chain parameters from the JMT.
    async fn get_distributions_params(&self) -> Result<DistributionsParameters> {
        self.get(state_key::distributions_parameters())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Missing DistributionsParameters"))
    }

    // Get the total amount of staking tokens issued for this epoch.
    fn get_staking_token_issuance_for_epoch(&self) -> Option<Amount> {
        self.object_get(&state_key::staking_token_issuance_for_epoch())
    }

    // Get the total amount of LQT rewards issued for this epoch.
    async fn get_lqt_reward_issuance_for_epoch(&self, epoch_index: u64) -> Option<Amount> {
        let key = state_key::lqt::v1::budget::for_epoch(epoch_index);

        self.nonverifiable_get(&key).await.unwrap_or_else(|_| {
            tracing::error!("LQT issuance does not exist for epoch");
            None
        })
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}
#[async_trait]

pub trait StateWriteExt: StateWrite + StateReadExt {
    /// Set the total amount of staking tokens issued for this epoch.
    fn set_staking_token_issuance_for_epoch(&mut self, issuance: Amount) {
        self.object_put(state_key::staking_token_issuance_for_epoch(), issuance);
    }

    /// Set the Distributions parameters in the JMT.
    fn put_distributions_params(&mut self, params: DistributionsParameters) {
        self.put(state_key::distributions_parameters().into(), params)
    }

    /// Set the total amount of LQT rewards issued for this epoch.
    fn set_lqt_reward_issuance_for_epoch(&mut self, epoch_index: u64, issuance: Amount) {
        self.nonverifiable_put(
            state_key::lqt::v1::budget::for_epoch(epoch_index).into(),
            issuance,
        );
    }
}
impl<T: StateWrite + ?Sized> StateWriteExt for T {}
