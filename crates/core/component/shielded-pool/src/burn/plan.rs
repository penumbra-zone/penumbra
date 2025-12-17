use std::convert::{TryFrom, TryInto};

use anyhow::Error;
use decaf377::Fr;
use penumbra_sdk_asset::{balance, Value};
use penumbra_sdk_proto::{core::component::shielded_pool::v1 as pb, DomainType};
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};

use super::ActionBurn;

/// A plan for a burn action.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::ActionBurnPlan", into = "pb::ActionBurnPlan")]
pub struct ActionBurnPlan {
    pub value: Value,
    pub value_blinding: Fr,
}

impl ActionBurnPlan {
    /// Create a new burn plan for the given value.
    pub fn new<R: CryptoRng + Rng>(rng: &mut R, value: Value) -> Self {
        Self {
            value,
            value_blinding: Fr::rand(rng),
        }
    }

    /// Returns the balance contribution of this burn plan.
    pub fn balance(&self) -> balance::Balance {
        -balance::Balance::from(self.value)
    }

    /// Returns the value blinding factor.
    pub fn value_blinding(&self) -> Fr {
        self.value_blinding
    }

    /// Build the action from the plan.
    pub fn build(self) -> ActionBurn {
        ActionBurn { value: self.value }
    }
}

impl DomainType for ActionBurnPlan {
    type Proto = pb::ActionBurnPlan;
}

impl From<ActionBurnPlan> for pb::ActionBurnPlan {
    fn from(plan: ActionBurnPlan) -> Self {
        pb::ActionBurnPlan {
            value: Some(plan.value.into()),
            value_blinding: plan.value_blinding.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::ActionBurnPlan> for ActionBurnPlan {
    type Error = Error;

    fn try_from(proto: pb::ActionBurnPlan) -> anyhow::Result<Self, Self::Error> {
        let value_blinding_bytes: [u8; 32] = proto
            .value_blinding
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid value_blinding length"))?;

        Ok(ActionBurnPlan {
            value: proto
                .value
                .ok_or_else(|| anyhow::anyhow!("missing value in ActionBurnPlan"))?
                .try_into()?,
            value_blinding: Fr::from_bytes_checked(&value_blinding_bytes)
                .map_err(|_| anyhow::anyhow!("invalid value_blinding"))?,
        })
    }
}
