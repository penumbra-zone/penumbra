use penumbra_sdk_asset::{Balance, Value};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{penumbra::core::component::stake::v1 as pb, DomainType};
use penumbra_sdk_sct::epoch::Epoch;
use penumbra_sdk_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};

use crate::{DelegationToken, IdentityKey, UnbondingToken};

/// A transaction action withdrawing stake from a validator's delegation pool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::Undelegate", into = "pb::Undelegate")]
pub struct Undelegate {
    /// The identity key of the validator to undelegate from.
    pub validator_identity: IdentityKey,
    /// The epoch at which the undelegation was performed.
    /// The undelegation takes effect after the unbonding period.
    pub from_epoch: Epoch,
    /// The amount to undelegate, in units of unbonding tokens.
    pub unbonded_amount: Amount,
    /// The amount of delegation tokens produced by this action.
    ///
    /// This is implied by the validator's exchange rate in the specified epoch
    /// (and should be checked in transaction validation!), but including it allows
    /// stateless verification that the transaction is internally consistent.
    pub delegation_amount: Amount,
}

impl EffectingData for Undelegate {
    fn effect_hash(&self) -> EffectHash {
        // For undelegations, the entire action is considered effecting data.
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl Undelegate {
    /// Return the balance after consuming delegation tokens, and producing unbonding tokens.
    pub fn balance(&self) -> Balance {
        let undelegation: Balance = self.unbonded_value().into();
        let delegation: Balance = self.delegation_value().into();

        // We consume the delegation tokens and produce the undelegation tokens.
        undelegation - delegation
    }

    pub fn unbonding_token(&self) -> UnbondingToken {
        // We produce undelegation tokens at a rate of 1:1 with the unbonded
        // value of the delegated stake. When these tokens are claimed, we
        // apply penalties that accumulated during the unbonding window.
        UnbondingToken::new(
            self.validator_identity.clone(),
            self.from_epoch.start_height,
        )
    }

    /// Returns the [`Value`] of the unbonded [`Amount`].
    pub fn unbonded_value(&self) -> Value {
        Value {
            amount: self.unbonded_amount,
            asset_id: self.unbonding_token().id(),
        }
    }

    pub fn delegation_token(&self) -> DelegationToken {
        DelegationToken::new(self.validator_identity.clone())
    }

    /// Returns the [`Value`] of the delegation [`Amount`].
    pub fn delegation_value(&self) -> Value {
        Value {
            amount: self.delegation_amount,
            asset_id: self.delegation_token().id(),
        }
    }
}

impl DomainType for Undelegate {
    type Proto = pb::Undelegate;
}

impl From<Undelegate> for pb::Undelegate {
    #[allow(deprecated)]
    fn from(d: Undelegate) -> Self {
        pb::Undelegate {
            validator_identity: Some(d.validator_identity.into()),
            unbonded_amount: Some(d.unbonded_amount.into()),
            delegation_amount: Some(d.delegation_amount.into()),
            from_epoch: Some(d.from_epoch.into()),
            start_epoch_index: 0,
        }
    }
}

impl TryFrom<pb::Undelegate> for Undelegate {
    type Error = anyhow::Error;
    fn try_from(d: pb::Undelegate) -> Result<Self, Self::Error> {
        Ok(Self {
            validator_identity: d
                .validator_identity
                .ok_or_else(|| anyhow::anyhow!("missing validator_identity"))?
                .try_into()?,
            from_epoch: d
                .from_epoch
                .ok_or_else(|| anyhow::anyhow!("missing from_epoch"))?
                .try_into()?,
            unbonded_amount: d
                .unbonded_amount
                .ok_or_else(|| anyhow::anyhow!("missing unbonded_amount"))?
                .try_into()?,
            delegation_amount: d
                .delegation_amount
                .ok_or_else(|| anyhow::anyhow!("missing delegation_amount"))?
                .try_into()?,
        })
    }
}
