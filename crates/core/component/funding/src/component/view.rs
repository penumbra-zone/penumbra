use async_trait::async_trait;

use crate::{component::state_key, params::FundingParameters};
use anyhow::Result;
use cnidarium::{StateRead, StateWrite};
use penumbra_proto::{StateReadProto, StateWriteProto};

#[async_trait]
pub trait StateReadExt: StateRead {
    /// Indicates if the funding parameters have been updated in this block.
    fn funding_params_updated(&self) -> bool {
        self.object_get::<()>(state_key::funding_parameters_updated())
            .is_some()
    }

    /// Gets the funding module chain parameters from the JMT.
    async fn get_funding_params(&self) -> Result<FundingParameters> {
        self.get(state_key::funding_parameters())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Missing FundingParameters"))
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

#[async_trait]
pub trait StateWriteExt: StateWrite + StateReadExt {
    /// Set the Funding parameters in the JMT.
    fn put_funding_params(&mut self, params: FundingParameters) {
        // Note that the fee params have been updated:
        self.object_put(state_key::funding_parameters_updated(), ());
        self.put(state_key::funding_parameters().into(), params)
    }
}
impl<T: StateWrite + ?Sized> StateWriteExt for T {}
