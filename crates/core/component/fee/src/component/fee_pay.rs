use anyhow::{ensure, Result};
use cnidarium::StateWrite;

use crate::{Fee, Gas};

use super::view::{StateReadExt, StateWriteExt};

/// Allows payment of transaction fees.
pub trait FeePay: StateWrite {
    async fn pay_fee(&mut self, gas_used: Gas, fee: Fee) -> Result<()> {
        let current_gas_prices = if fee.asset_id() == *penumbra_asset::STAKING_TOKEN_ASSET_ID {
            self.get_gas_prices()
                .await
                .expect("gas prices must be present in state")
        } else {
            let alt_gas_prices = self
                .get_alt_gas_prices()
                .await
                .expect("alt gas prices must be present in state");
            alt_gas_prices
                .into_iter()
                .find(|prices| prices.asset_id == fee.asset_id())
                .ok_or_else(|| {
                    anyhow::anyhow!("fee token {} not recognized by the chain", fee.asset_id())
                })?
        };

        // Double check that the gas price assets match.
        ensure!(
            current_gas_prices.asset_id == fee.asset_id(),
            "unexpected mismatch between fee and queried gas prices (expected: {}, found: {})",
            fee.asset_id(),
            current_gas_prices.asset_id,
        );

        let base_fee = current_gas_prices.fee(&gas_used);

        ensure!(
            fee.amount() >= base_fee.amount(),
            "fee must be greater than or equal to the transaction base price (supplied: {}, base: {})",
            fee.amount(),
            base_fee.amount(),
        );

        self.record_proto()

        Ok(())
    }
}
