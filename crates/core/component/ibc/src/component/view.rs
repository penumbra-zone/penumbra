use anyhow::Result;
use async_trait::async_trait;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};

use crate::params::IBCParameters;

use super::state_key;

#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Writes the provided IBC parameters to the JMT.
    fn put_ibc_params(&mut self, params: IBCParameters) {
        // TODO: this needs to be handled on a per-component basis or possibly removed from the compact block
        // entirely, currently disabled, see https://github.com/penumbra-zone/penumbra/issues/3107
        // Note to the shielded pool to include the chain parameters in the next compact block:
        // self.object_put(state_key::chain_params_changed(), ());

        // Change the IBC parameters:
        self.put(state_key::ibc_params().into(), params)
    }
}

impl<T: StateWrite> StateWriteExt for T {}

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
