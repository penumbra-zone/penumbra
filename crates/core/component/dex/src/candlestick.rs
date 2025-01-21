use anyhow::Result;
use serde::{Deserialize, Serialize};

use penumbra_sdk_proto::{core::component::dex::v1 as pb, DomainType};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "pb::CandlestickData", into = "pb::CandlestickData")]
pub struct CandlestickData {
    /// The block height of the candlestick data.
    pub height: u64,
    /// The first observed price during the block execution.
    pub open: f64,
    /// The last observed price during the block execution.
    pub close: f64,
    /// The highest observed price during the block execution.
    pub high: f64,
    /// The lowest observed price during the block execution.
    pub low: f64,
    /// The volume that traded "directly", during individual position executions.
    pub direct_volume: f64,
    /// The volume that traded as part of swaps, which could have traversed multiple routes.
    pub swap_volume: f64,
}

impl DomainType for CandlestickData {
    type Proto = pb::CandlestickData;
}

impl From<CandlestickData> for pb::CandlestickData {
    fn from(cd: CandlestickData) -> Self {
        Self {
            height: cd.height,
            open: cd.open,
            close: cd.close,
            high: cd.high,
            low: cd.low,
            direct_volume: cd.direct_volume,
            swap_volume: cd.swap_volume,
        }
    }
}

impl TryFrom<pb::CandlestickData> for CandlestickData {
    type Error = anyhow::Error;
    fn try_from(cd: pb::CandlestickData) -> Result<Self, Self::Error> {
        Ok(Self {
            height: cd.height,
            open: cd.open,
            close: cd.close,
            high: cd.high,
            low: cd.low,
            direct_volume: cd.direct_volume,
            swap_volume: cd.swap_volume,
        })
    }
}
