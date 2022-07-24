use decaf377::FieldExt;
use penumbra_proto::{dex as pb, Protobuf};

use crate::asset;

#[derive(Debug, Clone)]
pub struct TradingPair {
    pub asset_1: asset::Id,
    pub asset_2: asset::Id,
}

impl TradingPair {
    /// Convert the trading pair to bytes.
    pub fn to_bytes(&self) -> [u8; 64] {
        let mut result: [u8; 64] = [0; 64];
        result[0..32].copy_from_slice(&self.asset_1.0.to_bytes());
        result[32..64].copy_from_slice(&self.asset_2.0.to_bytes());
        result
    }
}

impl TryFrom<[u8; 64]> for TradingPair {
    type Error = anyhow::Error;
    fn try_from(bytes: [u8; 64]) -> anyhow::Result<Self> {
        let asset_1_bytes = &bytes[0..32];
        let asset_2_bytes = &bytes[32..64];
        Ok(Self {
            asset_1: asset_1_bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid asset_1 bytes in TradingPair"))?,
            asset_2: asset_2_bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid asset_2 bytes in TradingPair"))?,
        })
    }
}

impl Protobuf<pb::TradingPair> for TradingPair {}

impl TryFrom<pb::TradingPair> for TradingPair {
    type Error = anyhow::Error;
    fn try_from(tp: pb::TradingPair) -> anyhow::Result<Self> {
        Ok(Self {
            asset_1: tp
                .asset_1
                .ok_or_else(|| anyhow::anyhow!("missing trading pair asset1"))?
                .try_into()?,
            asset_2: tp
                .asset_2
                .ok_or_else(|| anyhow::anyhow!("missing trading pair asset2"))?
                .try_into()?,
        })
    }
}

impl From<TradingPair> for pb::TradingPair {
    fn from(tp: TradingPair) -> Self {
        Self {
            asset_1: Some(tp.asset_1.into()),
            asset_2: Some(tp.asset_2.into()),
        }
    }
}
