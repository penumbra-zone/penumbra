use penumbra_crypto::{
    value, DelegationToken, Fr, IdentityKey, Value, Zero, STAKING_TOKEN_ASSET_ID,
};
use penumbra_proto::{dex as pb, Protobuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Swap", into = "pb::Swap")]
pub struct Swap {}

impl Protobuf<pb::Swap> for Swap {}

impl From<Swap> for pb::Swap {
    fn from(d: Swap) -> Self {
        pb::Swap {}
    }
}

impl TryFrom<pb::Swap> for Swap {
    type Error = anyhow::Error;
    fn try_from(d: pb::Swap) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}
