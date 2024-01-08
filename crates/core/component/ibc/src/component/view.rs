use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_proto::{StateReadProto, StateWriteProto};

use crate::params::IBCParameters;

use super::state_key;

#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Writes the provided IBC parameters to the JMT.
    fn put_ibc_params(&mut self, params: IBCParameters) {
        // Note that the IBC params have been updated:
        self.object_put(state_key::ibc_params_updated(), ());

        // Change the IBC parameters:
        self.put(state_key::ibc_params().into(), params)
    }
}

impl<T: StateWrite> StateWriteExt for T {}

#[async_trait]
pub trait StateReadExt: StateRead {
    /// Indicates if the IBC parameters have been updated in this block.
    fn ibc_params_updated(&self) -> bool {
        self.object_get::<()>(state_key::ibc_params_updated())
            .is_some()
    }

    /// Gets the IBC parameters from the JMT.
    async fn get_ibc_params(&self) -> Result<IBCParameters> {
        self.get(state_key::ibc_params())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Missing IBCParameters"))
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}
