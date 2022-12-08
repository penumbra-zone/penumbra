use ark_ff::Zero;
use penumbra_crypto::{
    asset::{self, Amount},
    balance,
    proofs::transparent::UndelegateClaimProof,
    Balance, DelegationToken, Fr, IdentityKey, Value, STAKING_TOKEN_ASSET_ID,
};
use penumbra_proto::{core::stake::v1alpha1 as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::{ActionView, IsAction, TransactionPerspective};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::UndelegateClaimBody", into = "pb::UndelegateClaimBody")]
pub struct UndelegateClaimBody {
    /// The identity key of the validator to undelegate from.
    pub validator_identity: IdentityKey,
    /// The epoch in which unbonding began, used to verify the penalty.
    pub start_epoch_index: u64,
    /// The epoch in which unbonding ended, used to verify the penalty.
    pub end_epoch_index: u64,
    /// The penalty applied to undelegation, in bps^2.
    pub penalty: u64,
    /// The action's contribution to the transaction's value balance.
    pub balance_commitment: balance::Commitment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::UndelegateClaim", into = "pb::UndelegateClaim")]
pub struct UndelegateClaim {
    pub body: UndelegateClaimBody,
    pub proof: UndelegateClaimProof,
}

impl IsAction for UndelegateClaim {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        self.body.balance_commitment
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::UndelegateClaim(self.to_owned())
    }
}

impl Protobuf<pb::UndelegateClaimBody> for UndelegateClaimBody {}

impl From<UndelegateClaimBody> for pb::UndelegateClaimBody {
    fn from(d: UndelegateClaimBody) -> Self {
        pb::UndelegateClaimBody {
            validator_identity: Some(d.validator_identity.into()),
            start_epoch_index: d.start_epoch_index,
            end_epoch_index: d.end_epoch_index,
            penalty: d.penalty,
            balance_commitment: Some(d.balance_commitment.into()),
        }
    }
}

impl TryFrom<pb::UndelegateClaimBody> for UndelegateClaimBody {
    type Error = anyhow::Error;
    fn try_from(d: pb::UndelegateClaimBody) -> Result<Self, Self::Error> {
        Ok(Self {
            validator_identity: d
                .validator_identity
                .ok_or_else(|| anyhow::anyhow!("missing validator identity"))?
                .try_into()?,
            start_epoch_index: d.start_epoch_index,
            end_epoch_index: d.end_epoch_index,
            penalty: d.penalty,
            balance_commitment: d
                .balance_commitment
                .ok_or_else(|| anyhow::anyhow!("missing balance_commitment"))?
                .try_into()?,
        })
    }
}

impl Protobuf<pb::UndelegateClaim> for UndelegateClaim {}

impl From<UndelegateClaim> for pb::UndelegateClaim {
    fn from(d: UndelegateClaim) -> Self {
        pb::UndelegateClaim {
            body: Some(d.body.into()),
            proof: d.proof.encode_to_vec(),
        }
    }
}

impl TryFrom<pb::UndelegateClaim> for UndelegateClaim {
    type Error = anyhow::Error;
    fn try_from(d: pb::UndelegateClaim) -> Result<Self, Self::Error> {
        Ok(Self {
            body: d
                .body
                .ok_or_else(|| anyhow::anyhow!("missing body"))?
                .try_into()?,
            proof: UndelegateClaimProof::decode(d.proof.as_slice())?,
        })
    }
}
