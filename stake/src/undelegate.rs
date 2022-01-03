use penumbra_proto::{stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::IdentityKey;

/// A transaction action withdrawing stake from a validator's delegation pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::Undelegate", into = "pb::Undelegate")]
pub struct Undelegate {
    /// The identity key of the validator to undelegate from.
    pub validator_identity: IdentityKey,
    /// The index of the epoch in which this undelegation was performed.
    /// The undelegation takes effect after the unbonding period.
    pub epoch_index: u64,
    /// The amount to undelegate, in units of unbonded stake.
    pub unbonded_amount: u64,
    /// The amount of delegation tokens produced by this action.
    ///
    /// This is implied by the validator's exchange rate in the specified epoch
    /// (and should be checked in transaction validation!), but including it allows
    /// stateless verification that the transaction is internally consistent.
    pub delegation_amount: u64,
}

impl Protobuf<pb::Undelegate> for Undelegate {}

impl From<Undelegate> for pb::Undelegate {
    fn from(d: Undelegate) -> Self {
        pb::Undelegate {
            validator_identity: Some(d.validator_identity.into()),
            epoch_index: d.epoch_index,
            unbonded_amount: d.unbonded_amount,
            delegation_amount: d.delegation_amount,
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
            epoch_index: d.epoch_index,
            unbonded_amount: d.unbonded_amount,
            delegation_amount: d.delegation_amount,
        })
    }
}
