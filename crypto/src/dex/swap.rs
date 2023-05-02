use anyhow::{anyhow, Result};
use ark_ff::PrimeField;

use decaf377::Fq;
use once_cell::sync::Lazy;
use penumbra_proto::{
    client::v1alpha1::BatchSwapOutputDataResponse, core::dex::v1alpha1 as pb, DomainType, TypeUrl,
};
use serde::{Deserialize, Serialize};

use crate::{fixpoint::U128x128, Amount};

use super::TradingPair;

mod ciphertext;
mod payload;
mod plaintext;

pub use ciphertext::SwapCiphertext;
pub use payload::SwapPayload;
pub use plaintext::{SwapPlaintext, SwapPlaintextVar};

// Swap ciphertext byte length.
pub const SWAP_CIPHERTEXT_BYTES: usize = 272;
// Swap plaintext byte length.
pub const SWAP_LEN_BYTES: usize = 256;

pub static DOMAIN_SEPARATOR: Lazy<Fq> =
    Lazy::new(|| Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.swap").as_bytes()));

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::BatchSwapOutputData", into = "pb::BatchSwapOutputData")]
pub struct BatchSwapOutputData {
    /// The total amount of asset 1 that was input to the batch swap.
    pub delta_1: Amount,
    /// The total amount of asset 2 that was input to the batch swap.
    pub delta_2: Amount,
    /// The total amount of asset 1 that was output from the batch swap for 1=>2 trades.
    pub lambda_1_1: Amount,
    /// The total amount of asset 2 that was output from the batch swap for 1=>2 trades.
    pub lambda_2_1: Amount,
    /// The total amount of asset 1 that was output from the batch swap for 2=>1 trades.
    pub lambda_1_2: Amount,
    /// The total amount of asset 2 that was output from the batch swap for 2=>1 trades.
    pub lambda_2_2: Amount,
    /// The height for which the batch swap data is valid.
    pub height: u64,
    /// The trading pair associated with the batch swap.
    pub trading_pair: TradingPair,
    /// The starting block height of the epoch for which the batch swap data is valid.
    pub epoch_height: u64,
}

impl BatchSwapOutputData {
    /// Given a user's inputs `(delta_1_i, delta_2_i)`, compute their pro rata share
    /// of the batch output `(lambda_1_i, lambda_2_i)`.
    pub fn pro_rata_outputs(&self, (delta_1_i, delta_2_i): (Amount, Amount)) -> (Amount, Amount) {
        // The pro rata fraction is delta_j_i / delta_j, which we can multiply through:
        //   lambda_2_i = (delta_1_i / delta_1) * lambda_2_1 + (delta_2_i / delta_2) * lambda_2_2
        //   lambda_1_i = (delta_1_i / delta_1) * lambda_1_1 + (delta_2_i / delta_2) * lambda_1_2

        let delta_1_i = U128x128::from(delta_1_i);
        let delta_2_i = U128x128::from(delta_2_i);
        let delta_1 = U128x128::from(self.delta_1);
        let delta_2 = U128x128::from(self.delta_2);
        let lambda_1_1 = U128x128::from(self.lambda_1_1);
        let lambda_1_2 = U128x128::from(self.lambda_1_2);
        let lambda_2_1 = U128x128::from(self.lambda_2_1);
        let lambda_2_2 = U128x128::from(self.lambda_2_2);

        // Compute the user i's share of the batch inputs of assets 1 and 2.
        // The .unwrap_or_default ensures that when the batch input delta_1 is zero, all pro-rata shares of it are also zero.
        let pro_rata_input_1 = (delta_1_i / delta_1).unwrap_or_default();
        let pro_rata_input_2 = (delta_2_i / delta_2).unwrap_or_default();

        let lambda_2_i = (pro_rata_input_1 * lambda_2_1).unwrap_or_default()
            + (pro_rata_input_2 * lambda_2_2).unwrap_or_default();
        let lambda_1_i = (pro_rata_input_1 * lambda_1_1).unwrap_or_default()
            + (pro_rata_input_2 * lambda_1_2).unwrap_or_default();

        (
            lambda_1_i
                .unwrap_or_default()
                .round_down()
                .try_into()
                .expect("rounded amount is integral"),
            lambda_2_i
                .unwrap_or_default()
                .round_down()
                .try_into()
                .expect("rounded amount is integral"),
        )
    }
}

impl TypeUrl for BatchSwapOutputData {
    const TYPE_URL: &'static str = "/penumbra.dex.v1alpha1.BatchSwapOutputData";
}

impl DomainType for BatchSwapOutputData {
    type Proto = pb::BatchSwapOutputData;
}

impl From<BatchSwapOutputData> for pb::BatchSwapOutputData {
    fn from(s: BatchSwapOutputData) -> Self {
        pb::BatchSwapOutputData {
            delta_1: Some(s.delta_1.into()),
            delta_2: Some(s.delta_2.into()),
            trading_pair: Some(s.trading_pair.into()),
            height: s.height,
            lambda_1_1: Some(s.lambda_1_1.into()),
            lambda_2_1: Some(s.lambda_2_1.into()),
            lambda_1_2: Some(s.lambda_1_2.into()),
            lambda_2_2: Some(s.lambda_2_2.into()),
            epoch_height: s.epoch_height,
        }
    }
}

impl From<BatchSwapOutputData> for BatchSwapOutputDataResponse {
    fn from(s: BatchSwapOutputData) -> Self {
        BatchSwapOutputDataResponse {
            data: Some(s.into()),
        }
    }
}

impl TryFrom<pb::BatchSwapOutputData> for BatchSwapOutputData {
    type Error = anyhow::Error;
    fn try_from(s: pb::BatchSwapOutputData) -> Result<Self, Self::Error> {
        Ok(Self {
            delta_1: s
                .delta_1
                .ok_or_else(|| anyhow!("Missing delta_1"))?
                .try_into()?,
            delta_2: s
                .delta_2
                .ok_or_else(|| anyhow!("Missing delta_2"))?
                .try_into()?,
            lambda_1_1: s
                .lambda_1_1
                .ok_or_else(|| anyhow!("Missing lambda_1_1"))?
                .try_into()?,
            lambda_2_1: s
                .lambda_2_1
                .ok_or_else(|| anyhow!("Missing lambda_2_1"))?
                .try_into()?,
            lambda_1_2: s
                .lambda_1_2
                .ok_or_else(|| anyhow!("Missing lambda_1_2"))?
                .try_into()?,
            lambda_2_2: s
                .lambda_2_2
                .ok_or_else(|| anyhow!("Missing lambda_2_2"))?
                .try_into()?,
            height: s.height,
            trading_pair: s
                .trading_pair
                .ok_or_else(|| anyhow!("Missing trading_pair"))?
                .try_into()?,
            epoch_height: s.epoch_height,
        })
    }
}

impl TryFrom<BatchSwapOutputDataResponse> for BatchSwapOutputData {
    type Error = anyhow::Error;
    fn try_from(value: BatchSwapOutputDataResponse) -> Result<Self, Self::Error> {
        value
            .data
            .ok_or_else(|| anyhow::anyhow!("empty BatchSwapOutputDataResponse message"))?
            .try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pasiphae_inflation_bug() {
        let bsod: BatchSwapOutputData = serde_json::from_str(
            r#"
{
    "delta1": {
        "lo": "31730032"
    },
    "delta2": {},
    "lambda11": {},
    "lambda21": {
        "lo": "28766268"
    },
    "lambda12": {},
    "lambda22": {},
    "height": "2185",
    "tradingPair": {
        "asset1": {
        "inner": "HW2Eq3UZVSBttoUwUi/MUtE7rr2UU7/UH500byp7OAc="
        },
        "asset2": {
        "inner": "KeqcLzNx9qSH5+lcJHBB9KNW+YPrBk5dKzvPMiypahA="
        }
    }
}"#,
        )
        .unwrap();

        let (delta_1_i, delta_2_i) = (Amount::from(31730032u64), Amount::from(0u64));

        let (lambda_1_i, lambda_2_i) = bsod.pro_rata_outputs((delta_1_i, delta_2_i));

        assert_eq!(lambda_1_i, Amount::from(0u64));
        assert_eq!(lambda_2_i, Amount::from(28766268u64));
    }
}
