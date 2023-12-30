use penumbra_asset::{Balance, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_num::Amount;
use penumbra_proto::{penumbra::core::component::stake::v1alpha1 as pb, DomainType};
use penumbra_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};

use crate::{DelegationToken, IdentityKey};

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
    /// TODO(erwan): make sure this is checked in tx validation
    pub delegation_amount: Amount,
}

impl EffectingData for Delegate {
    fn effect_hash(&self) -> EffectHash {
        // For delegations, the entire action is considered effecting data.
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl Delegate {
    /// Return the balance resulting from issuing delegation tokens from staking tokens.
    pub fn balance(&self) -> Balance {
        let stake = Balance::from(Value {
            amount: self.unbonded_amount,
            asset_id: STAKING_TOKEN_ASSET_ID.clone(),
        });

        let delegation = Balance::from(Value {
            amount: self.delegation_amount,
            asset_id: DelegationToken::new(self.validator_identity.clone()).id(),
        });

        // We produce the delegation tokens and consume the staking tokens.
        delegation - stake
    }
}

impl DomainType for Delegate {
    type Proto = pb::Delegate;
}

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
