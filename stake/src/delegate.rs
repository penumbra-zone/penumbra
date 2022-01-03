use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::IdentityKey;

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
    pub unbonded_amount: u64,
    /// The amount of delegation tokens produced by this action.
    ///
    /// This is implied by the validator's exchange rate in the specified epoch
    /// (and should be checked in transaction validation!), but including it allows
    /// stateless verification that the transaction is internally consistent.
    pub delegation_amount: u64,
}

impl Protobuf<pb::Delegate> for Delegate {}

impl From<Delegate> for pb::Delegate {
    fn from(d: Delegate) -> Self {
        pb::Delegate {
            validator_identity: Some(d.validator_identity.into()),
            epoch_index: d.epoch_index,
            unbonded_amount: d.unbonded_amount,
            delegation_amount: d.delegation_amount,
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
            unbonded_amount: d.unbonded_amount,
            delegation_amount: d.delegation_amount,
        })
    }
}
