use std::convert::{TryFrom, TryInto};

use anyhow::Error;
use penumbra_sdk_asset::{balance, Value};
use penumbra_sdk_proto::{core::component::shielded_pool::v1 as pb, DomainType};
use penumbra_sdk_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};

/// Explicitly burns a value, removing it from circulation.
///
/// This is a generic burn action that works on any asset type,
/// including LP NFTs (to make liquidity immutable) and mint
/// capability NFTs (to fix token supply).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::ActionBurn", into = "pb::ActionBurn")]
pub struct ActionBurn {
    pub value: Value,
}

impl ActionBurn {
    /// Returns the balance contribution of this burn action.
    /// Burns consume value, so this returns a negative balance.
    pub fn balance(&self) -> balance::Balance {
        -balance::Balance::from(self.value)
    }
}

impl EffectingData for ActionBurn {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl DomainType for ActionBurn {
    type Proto = pb::ActionBurn;
}

impl From<ActionBurn> for pb::ActionBurn {
    fn from(burn: ActionBurn) -> Self {
        pb::ActionBurn {
            value: Some(burn.value.into()),
        }
    }
}

impl TryFrom<pb::ActionBurn> for ActionBurn {
    type Error = Error;

    fn try_from(proto: pb::ActionBurn) -> anyhow::Result<Self, Self::Error> {
        Ok(ActionBurn {
            value: proto
                .value
                .ok_or_else(|| anyhow::anyhow!("missing value in ActionBurn"))?
                .try_into()?,
        })
    }
}
