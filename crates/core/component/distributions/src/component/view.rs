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

    fn get_staking_token_issuance_for_epoch(&self) -> Option<Amount> {
        self.object_get(&state_key::staking_token_issuance_for_epoch())
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
}
impl<T: StateWrite + ?Sized> StateWriteExt for T {}
