use penumbra_crypto::{
    balance,
    proofs::groth16::UndelegateClaimProof,
    stake::{IdentityKey, Penalty},
};
use penumbra_proto::{core::stake::v1alpha1 as pb, DomainType, TypeUrl};
use serde::{Deserialize, Serialize};

use crate::{ActionView, IsAction, TransactionPerspective};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::UndelegateClaimBody", into = "pb::UndelegateClaimBody")]
pub struct UndelegateClaimBody {
    /// The identity key of the validator to undelegate from.
    pub validator_identity: IdentityKey,
    /// The epoch in which unbonding began, used to verify the penalty.
    pub start_epoch_index: u64,
    /// The penalty applied to undelegation, in bps^2.
    pub penalty: Penalty,
    /// The action's contribution to the transaction's value balance.
    pub balance_commitment: balance::Commitment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl TypeUrl for UndelegateClaimBody {
    const TYPE_URL: &'static str = "/penumbra.core.stake.v1alpha1.UndelegateClaimBody";
}

impl DomainType for UndelegateClaimBody {
    type Proto = pb::UndelegateClaimBody;
}

impl From<UndelegateClaimBody> for pb::UndelegateClaimBody {
    fn from(d: UndelegateClaimBody) -> Self {
        pb::UndelegateClaimBody {
            validator_identity: Some(d.validator_identity.into()),
            start_epoch_index: d.start_epoch_index,
            penalty: Some(d.penalty.into()),
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
            penalty: d
                .penalty
                .ok_or_else(|| anyhow::anyhow!("missing penalty"))?
                .try_into()?,
            balance_commitment: d
                .balance_commitment
                .ok_or_else(|| anyhow::anyhow!("missing balance_commitment"))?
                .try_into()?,
        })
    }
}

impl TypeUrl for UndelegateClaim {
    const TYPE_URL: &'static str = "/penumbra.core.stake.v1alpha1.UndelegateClaim";
}

impl DomainType for UndelegateClaim {
    type Proto = pb::UndelegateClaim;
}

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
