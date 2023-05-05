use ark_ff::Zero;
use penumbra_crypto::{
    asset::Amount,
    stake::{DelegationToken, IdentityKey, UnbondingToken},
    Balance, Fr, Value,
};
use penumbra_proto::{core::stake::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

use crate::{ActionView, IsAction, TransactionPerspective};

/// A transaction action withdrawing stake from a validator's delegation pool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::Undelegate", into = "pb::Undelegate")]
pub struct Undelegate {
    /// The identity key of the validator to undelegate from.
    pub validator_identity: IdentityKey,
    /// The index of the epoch in which this undelegation was performed.
    /// The undelegation takes effect after the unbonding period.
    pub start_epoch_index: u64,
    /// The amount to undelegate, in units of unbonding tokens.
    pub unbonded_amount: Amount,
    /// The amount of delegation tokens produced by this action.
    ///
    /// This is implied by the validator's exchange rate in the specified epoch
    /// (and should be checked in transaction validation!), but including it allows
    /// stateless verification that the transaction is internally consistent.
    pub delegation_amount: Amount,
}

impl IsAction for Undelegate {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::Undelegate(self.to_owned())
    }
}

impl Undelegate {
    /// Return the balance after consuming delegation tokens, and producing unbonding tokens.
    pub fn balance(&self) -> Balance {
        let stake = Balance::from(Value {
            amount: self.unbonded_amount,
            asset_id: self.unbonding_token().id(),
        });

        let delegation = Balance::from(Value {
            amount: self.delegation_amount,
            asset_id: self.delegation_token().id(),
        });

        // We consume the delegation tokens and produce the staking tokens.
        stake - delegation
    }

    pub fn unbonding_token(&self) -> UnbondingToken {
        UnbondingToken::new(self.validator_identity.clone(), self.start_epoch_index)
    }

    pub fn delegation_token(&self) -> DelegationToken {
        DelegationToken::new(self.validator_identity.clone())
    }
}

impl TypeUrl for Undelegate {
    const TYPE_URL: &'static str = "/penumbra.core.stake.v1alpha1.Undelegate";
}

impl DomainType for Undelegate {
    type Proto = pb::Undelegate;
}

impl From<Undelegate> for pb::Undelegate {
    fn from(d: Undelegate) -> Self {
        pb::Undelegate {
            validator_identity: Some(d.validator_identity.into()),
            start_epoch_index: d.start_epoch_index,
            unbonded_amount: Some(d.unbonded_amount.into()),
            delegation_amount: Some(d.delegation_amount.into()),
        }
    }
}

impl TryFrom<pb::Undelegate> for Undelegate {
    type Error = anyhow::Error;
    fn try_from(d: pb::Undelegate) -> Result<Self, Self::Error> {
        Ok(Self {
            validator_identity: d
                .validator_identity
                .ok_or_else(|| anyhow::anyhow!("missing validator identity"))?
                .try_into()?,
            start_epoch_index: d.start_epoch_index,
            unbonded_amount: d
                .unbonded_amount
                .ok_or_else(|| anyhow::anyhow!("missing unbonded amount"))?
                .try_into()?,
            delegation_amount: d
                .delegation_amount
                .ok_or_else(|| anyhow::anyhow!("missing delegation amount"))?
                .try_into()?,
        })
    }
}
