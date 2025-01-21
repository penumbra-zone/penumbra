use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};

use crate::params::IBCParameters;

use super::state_key;

#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Writes the provided IBC parameters to the JMT.
    fn put_ibc_params(&mut self, params: IBCParameters) {
        self.put(state_key::ibc_params().into(), params)
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}

#[async_trait]
pub trait StateReadExt: StateRead {
    /// Gets the IBC parameters from the JMT.
    async fn get_ibc_params(&self) -> Result<IBCParameters> {
        self.get(state_key::ibc_params())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Missing IBCParameters"))
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}
