use anyhow::{ensure, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sdk_asset::Value;
use penumbra_sdk_proto::core::component::fee::v1 as pb;
use penumbra_sdk_proto::state::StateWriteProto as _;

use crate::{Fee, Gas};

use super::view::{StateReadExt, StateWriteExt};

/// Allows payment of transaction fees.
#[async_trait]
pub trait FeePay: StateWrite {
    /// Uses the provided `fee` to pay for `gas_used`, erroring if the fee is insufficient.
    async fn pay_fee(&mut self, gas_used: Gas, fee: Fee) -> Result<()> {
        let current_gas_prices = if fee.asset_id() == *penumbra_sdk_asset::STAKING_TOKEN_ASSET_ID {
            self.get_gas_prices()
                .await
                .expect("gas prices must be present in state")
        } else {
            let alt_gas_prices = self
                .get_alt_gas_prices()
                .await
                .expect("alt gas prices must be present in state");
            // This does a linear scan, but we think that's OK because we're expecting
            // a small number of alt gas prices before switching to the DEX directly.
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

        // Compute the base fee for the `gas_used`.
        let base_fee = current_gas_prices.fee(&gas_used);

        // The provided fee must be at least the base fee.
        ensure!(
            fee.amount() >= base_fee.amount(),
            "fee must be greater than or equal to the transaction base price (supplied: {}, base: {})",
            fee.amount(),
            base_fee.amount(),
        );

        // Otherwise, the fee less the base fee is the proposer tip.
        let tip = Fee(Value {
            amount: fee.amount() - base_fee.amount(),
            asset_id: fee.asset_id(),
        });

        // Record information about the fee payment in an event.
        self.record_proto(pb::EventPaidFee {
            fee: Some(fee.into()),
            base_fee: Some(base_fee.into()),
            gas_used: Some(gas_used.into()),
            tip: Some(tip.into()),
        });

        // Finally, queue the paid fee for processing at the end of the block.
        self.raw_accumulate_base_fee(base_fee);
        self.raw_accumulate_tip(tip);

        Ok(())
    }
}

impl<S: StateWrite + ?Sized> FeePay for S {}
