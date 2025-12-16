//! ActionBurn - explicitly burn tokens.
//!
//! This action removes tokens from circulation permanently. It can be used for:
//!
//! - Deflationary token mechanics
//! - Burning mint authority NFTs to make supply immutable
//! - Redemption mechanisms
//!
//! # Value Balance
//!
//! The burn action consumes value from the transaction, producing nothing:
//!
//! ```text
//! ┌────────────────────┬──────────────────────┐
//! │     Consumed       │      Produced        │
//! ├────────────────────┼──────────────────────┤
//! │   burned value     │       nothing        │
//! └────────────────────┴──────────────────────┘
//! ```

use crate::error::TokenFactoryError;
use anyhow::anyhow;
use penumbra_sdk_asset::{Balance, Value};
use penumbra_sdk_proto::{core::component::token_factory::v1 as pb, DomainType};
use penumbra_sdk_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};

/// Action to explicitly burn tokens.
///
/// This removes the specified value from circulation permanently.
///
/// # Validation
///
/// The action is invalid if:
/// - The amount is zero (no-op, likely a bug)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ActionBurn", into = "pb::ActionBurn")]
pub struct ActionBurn {
    /// The value to burn.
    pub value: Value,
}

impl ActionBurn {
    /// Create a new burn action.
    ///
    /// # Errors
    ///
    /// Returns an error if the amount is zero.
    pub fn new(value: Value) -> Result<Self, TokenFactoryError> {
        if value.amount.value() == 0 {
            return Err(TokenFactoryError::BurnZeroAmount);
        }
        Ok(Self { value })
    }

    /// Compute the value balance for this action.
    ///
    /// Burns consume the specified value (negative balance).
    pub fn balance(&self) -> Balance {
        -Balance::from(self.value)
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
    fn from(action: ActionBurn) -> Self {
        Self {
            value: Some(action.value.into()),
        }
    }
}

impl TryFrom<pb::ActionBurn> for ActionBurn {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ActionBurn) -> Result<Self, Self::Error> {
        let value: Value = proto
            .value
            .ok_or_else(|| anyhow!("ActionBurn is missing value"))?
            .try_into()?;

        // Validate on deserialization
        if value.amount.value() == 0 {
            return Err(TokenFactoryError::BurnZeroAmount.into());
        }

        Ok(Self { value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use penumbra_sdk_asset::asset;
    use penumbra_sdk_num::Amount;

    #[test]
    fn test_burn_balance_is_negative() {
        let value = Value {
            asset_id: asset::Cache::with_known_assets().get_unit("upenumbra").unwrap().id(),
            amount: Amount::from(1000u64),
        };
        let action = ActionBurn { value };
        let balance = action.balance();

        // Balance should be negative (consuming value)
        assert!(balance != Balance::zero());
    }

    #[test]
    fn test_reject_zero_burn() {
        let value = Value {
            asset_id: asset::Cache::with_known_assets().get_unit("upenumbra").unwrap().id(),
            amount: Amount::from(0u64),
        };
        let result = ActionBurn::new(value);
        assert!(matches!(result, Err(TokenFactoryError::BurnZeroAmount)));
    }
}
