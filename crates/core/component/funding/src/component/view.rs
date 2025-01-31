use crate::{component::state_key, params::FundingParameters};
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};

#[async_trait]
pub trait StateReadExt: StateRead {
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
        self.put(state_key::funding_parameters().into(), params)
    }
}
impl<T: StateWrite + ?Sized> StateWriteExt for T {}
