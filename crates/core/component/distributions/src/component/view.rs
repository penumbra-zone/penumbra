use async_trait::async_trait;

use crate::{component::state_key, params::DistributionsParameters};
use anyhow::Result;
use penumbra_num::Amount;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};

#[async_trait]
pub trait StateReadExt: StateRead {
    /// Indicates if the Distributions parameters have been updated in this block.
    fn distributions_params_updated(&self) -> bool {
        self.object_get::<()>(state_key::distributions_parameters_updated())
            .is_some()
    }

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
        // Note that the fee params have been updated:
        self.object_put(state_key::distributions_parameters_updated(), ());
        self.put(state_key::distributions_parameters().into(), params)
    }
}
impl<T: StateWrite + ?Sized> StateWriteExt for T {}
