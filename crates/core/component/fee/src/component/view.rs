use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_asset::asset;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};

use crate::{params::FeeParameters, state_key, Fee, GasPrices};

/// This trait provides read access to fee-related parts of the Penumbra
/// state store.
#[async_trait]
pub trait StateReadExt: StateRead {
    /// Gets the fee parameters from the JMT.
    async fn get_fee_params(&self) -> Result<FeeParameters> {
        self.get(state_key::fee_params())
            .await?
            .ok_or_else(|| anyhow!("Missing FeeParameters"))
    }

    /// Gets the current gas prices for the fee token.
    async fn get_gas_prices(&self) -> Result<GasPrices> {
        // When we implement dynamic gas pricing, we will want
        // to read the prices we computed. But until then, we need to
        // read these from the _fee params_ instead, since those are
        // the values that will get updated by governance.
        let params = self.get_fee_params().await?;
        Ok(params.fixed_gas_prices)
    }

    /// Gets the current gas prices for alternative fee tokens.
    async fn get_alt_gas_prices(&self) -> Result<Vec<GasPrices>> {
        // When we implement dynamic gas pricing, we will want
        // to read the prices we computed. But until then, we need to
        // read these from the _fee params_ instead, since those are
        // the values that will get updated by governance.
        let params = self.get_fee_params().await?;
        Ok(params.fixed_alt_gas_prices)
    }

    /// Returns true if the gas prices have been changed in this block.
    fn gas_prices_changed(&self) -> bool {
        self.object_get::<()>(state_key::gas_prices_changed())
            .is_some()
    }

    /// The accumulated base fees and tips for this block, indexed by asset ID.
    fn accumulated_base_fees_and_tips(&self) -> im::OrdMap<asset::Id, (Amount, Amount)> {
        self.object_get(state_key::fee_accumulator())
            .unwrap_or_default()
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Writes the provided fee parameters to the JMT.
    fn put_fee_params(&mut self, params: FeeParameters) {
        self.put(state_key::fee_params().into(), params);
        // This could have changed the gas prices, so mark them as changed.
        self.object_put(state_key::gas_prices_changed(), ());
    }

    /*
    We shouldn't be setting gas prices directly, until we have dynamic gas pricing.
    /// Writes the provided gas prices to the JMT.
    fn put_gas_prices(&mut self, gas_prices: GasPrices) {
        // Change the gas prices:
        self.put(state_key::gas_prices().into(), gas_prices);

        // Mark that they've changed
        self.object_put(state_key::gas_prices_changed(), ());
    }
     */

    /// Takes the accumulated base fees and tips for this block, resetting them to zero.
    fn take_accumulated_base_fees_and_tips(&mut self) -> im::OrdMap<asset::Id, (Amount, Amount)> {
        let old = self.accumulated_base_fees_and_tips();
        let new = im::OrdMap::<asset::Id, (Amount, Amount)>::new();
        self.object_put(state_key::fee_accumulator(), new);
        old
    }

    fn raw_accumulate_base_fee(&mut self, base_fee: Fee) {
        let old = self.accumulated_base_fees_and_tips();
        let new = old.alter(
            |maybe_amounts| match maybe_amounts {
                Some((base, tip)) => Some((base + base_fee.amount(), tip)),
                None => Some((base_fee.amount(), Amount::zero())),
            },
            base_fee.asset_id(),
        );
        self.object_put(state_key::fee_accumulator(), new);
    }

    fn raw_accumulate_tip(&mut self, tip_fee: Fee) {
        let old = self.accumulated_base_fees_and_tips();
        let new = old.alter(
            |maybe_amounts| match maybe_amounts {
                Some((base, tip)) => Some((base, tip + tip_fee.amount())),
                None => Some((Amount::zero(), tip_fee.amount())),
            },
            tip_fee.asset_id(),
        );
        self.object_put(state_key::fee_accumulator(), new);
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
