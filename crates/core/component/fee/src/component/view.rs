use anyhow::{anyhow, Result};
use async_trait::async_trait;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};

use crate::{params::FeeParameters, state_key, GasPrices};

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

    /// Gets the gas prices from the JMT.
    async fn get_gas_prices(&self) -> Result<GasPrices> {
        self.get(state_key::gas_prices())
            .await?
            .ok_or_else(|| anyhow!("Missing GasPrices"))
    }

    /// Returns true if the gas prices have been changed in this block.
    fn gas_prices_changed(&self) -> bool {
        self.object_get::<()>(state_key::gas_prices_changed())
            .is_some()
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
        // Note that the fee params have been updated:
        self.object_put(state_key::fee_params_updated(), ());

        // Change the fee parameters:
        self.put(state_key::fee_params().into(), params)
    }

    /// Writes the provided gas prices to the JMT.
    fn put_gas_prices(&mut self, gas_prices: GasPrices) {
        // Change the gas prices:
        self.put(state_key::gas_prices().into(), gas_prices);

        // Mark that they've changed
        self.object_put(state_key::gas_prices_changed(), ());
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
