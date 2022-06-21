use penumbra_crypto::{
    value, DelegationToken, Fr, IdentityKey, Value, Zero, STAKING_TOKEN_ASSET_ID,
};
use penumbra_proto::{dex as pb, Protobuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::SwapClaim", into = "pb::SwapClaim")]
pub struct SwapClaim {}

impl Protobuf<pb::SwapClaim> for SwapClaim {}

impl From<SwapClaim> for pb::SwapClaim {
    fn from(d: SwapClaim) -> Self {
        pb::SwapClaim {}
    }
}

impl TryFrom<pb::SwapClaim> for SwapClaim {
    type Error = anyhow::Error;
    fn try_from(d: pb::SwapClaim) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}
