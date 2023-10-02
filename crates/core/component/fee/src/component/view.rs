use anyhow::{anyhow, Result};
use async_trait::async_trait;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};

use crate::{params::FeeParameters, state_key};

/// This trait provides read access to fee-related parts of the Penumbra
/// state store.
///
/// Note: the `get_` methods in this trait assume that the state store has been
/// initialized, so they will error on an empty state.
//#[async_trait(?Send)]
#[async_trait]
pub trait StateReadExt: StateRead {
    /// Gets the fee parameters from the JMT.
    async fn get_fee_params(&self) -> Result<FeeParameters> {
        self.get(state_key::fee_params())
            .await?
            .ok_or_else(|| anyhow!("Missing FeeParameters"))
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

/// This trait provides write access to common parts of the Penumbra
/// state store.
///
/// Note: the `get_` methods in this trait assume that the state store has been
/// initialized, so they will error on an empty state.
//#[async_trait(?Send)]
#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Writes the provided fee parameters to the JMT.
    fn put_fee_params(&mut self, params: FeeParameters) {
        // TODO: this needs to be handled on a per-component basis or possibly removed from the compact block
        // entirely, currently disabled, see https://github.com/penumbra-zone/penumbra/issues/3107
        // Note to the shielded pool to include the chain parameters in the next compact block:
        // self.object_put(state_key::chain_params_changed(), ());

        // Change the fee parameters:
        self.put(state_key::fee_params().into(), params)
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
