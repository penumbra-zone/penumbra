use penumbra_crypto::{
    asset::Amount, Balance, DelegationToken, IdentityKey, Value, STAKING_TOKEN_ASSET_ID,
};
use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

/// A transaction action adding stake to a validator's delegation pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Delegate", into = "pb::Delegate")]
pub struct Delegate {
    /// The identity key of the validator to delegate to.
    pub validator_identity: IdentityKey,
    /// The index of the epoch in which this delegation was performed.
    /// The delegation takes effect in the next epoch.
    pub epoch_index: u64,
    /// The delegation amount, in units of unbonded stake.
    /// TODO: use flow aggregation to hide this, replacing it with bytes amount_ciphertext;
    pub unbonded_amount: Amount,
    /// The amount of delegation tokens produced by this action.
    ///
    /// This is implied by the validator's exchange rate in the specified epoch
    /// (and should be checked in transaction validation!), but including it allows
    /// stateless verification that the transaction is internally consistent.
    pub delegation_amount: Amount,
}

impl Delegate {
    /// Compute a commitment to the value contributed to a transaction by this delegation.
    pub fn balance(&self) -> Balance {
        let stake = Value {
            amount: self.unbonded_amount,
            asset_id: STAKING_TOKEN_ASSET_ID.clone(),
        };
        let delegation = Value {
            amount: self.delegation_amount,
            asset_id: DelegationToken::new(self.validator_identity.clone()).id(),
        };

        // We produce the delegation tokens and consume the staking tokens.
        Balance::from(delegation) - stake
    }
}

impl Protobuf<pb::Delegate> for Delegate {}

impl From<Delegate> for pb::Delegate {
    fn from(d: Delegate) -> Self {
        pb::Delegate {
            validator_identity: Some(d.validator_identity.into()),
            epoch_index: d.epoch_index,
            unbonded_amount: Some(d.unbonded_amount.into()),
            delegation_amount: Some(d.delegation_amount.into()),
        }
    }
}

impl TryFrom<pb::Delegate> for Delegate {
    type Error = anyhow::Error;
    fn try_from(d: pb::Delegate) -> Result<Self, Self::Error> {
        Ok(Self {
            validator_identity: d
                .validator_identity
                .ok_or_else(|| anyhow::anyhow!("missing validator identity"))?
                .try_into()?,
            epoch_index: d.epoch_index,
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
