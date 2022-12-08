use ark_ff::UniformRand;
use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_crypto::{
    proofs::transparent::{SpendProof, UndelegateClaimProof},
    Address, Amount, FieldExt, Fq, Fr, FullViewingKey, IdentityKey, Note, Value,
    STAKING_TOKEN_ASSET_ID,
};
use penumbra_proto::{core::stake::v1alpha1 as pb, Protobuf};
use penumbra_tct as tct;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::action::{spend, Spend, UndelegateClaim, UndelegateClaimBody};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::UndelegateClaimPlan", into = "pb::UndelegateClaimPlan")]
pub struct UndelegateClaimPlan {
    /// The identity key of the validator to undelegate from.
    pub validator_identity: IdentityKey,
    /// The epoch in which unbonding began, used to verify the penalty.
    pub start_epoch_index: u64,
    /// The epoch in which unbonding ended, used to verify the penalty.
    pub end_epoch_index: u64,
    /// The penalty applied to undelegation, in bps^2.
    pub penalty: u64,
    /// The amount of unbonding tokens to claim. This is a bare number because its denom is determined by the preceding data.
    pub unbonding_amount: Amount,
    /// The blinding factor that will be used for the balance commitment.
    /// The action's contribution to the transaction's value balance.
    pub balance_blinding: Fr,
}

impl UndelegateClaimPlan {
    /// Convenience method to construct the [`UndelegateClaim`] described by this [`UndelegateClaimPlan`].
    pub fn undelegate_claim(&self) -> UndelegateClaim {
        todo!()
    }

    /// Construct the [`UndelegateClaimBody`] described by this [`UndelegateClaimPlan`].
    pub fn undelegate_claim_body(&self) -> UndelegateClaimBody {
        todo!()
    }

    /// Construct the [`UndelegateClaimProof`] required by the [`UndelegateClaimBody`] described by this [`UndelegateClaimPlan`].
    pub fn undelegate_claim_proof(&self) -> UndelegateClaimProof {
        todo!()
    }

    pub fn balance(&self) -> penumbra_crypto::Balance {
        todo!()
    }
}

impl Protobuf<pb::UndelegateClaimPlan> for UndelegateClaimPlan {}

impl From<UndelegateClaimPlan> for pb::UndelegateClaimPlan {
    fn from(msg: UndelegateClaimPlan) -> Self {
        Self {
            validator_identity: Some(msg.validator_identity.into()),
            start_epoch_index: msg.start_epoch_index,
            end_epoch_index: msg.end_epoch_index,
            penalty: msg.penalty,
            unbonding_amount: Some(msg.unbonding_amount.into()),
            balance_blinding: msg.balance_blinding.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::UndelegateClaimPlan> for UndelegateClaimPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::UndelegateClaimPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            validator_identity: msg
                .validator_identity
                .ok_or_else(|| anyhow::anyhow!("missing validator_identity"))?
                .try_into()?,
            start_epoch_index: msg.start_epoch_index,
            end_epoch_index: msg.end_epoch_index,
            penalty: msg.penalty,
            unbonding_amount: msg
                .unbonding_amount
                .ok_or_else(|| anyhow::anyhow!("missing unbonding_amount"))?
                .try_into()?,
            balance_blinding: Fr::from_bytes(
                msg.balance_blinding
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("expected 32 bytes"))?,
            )
            .map_err(|_| anyhow::anyhow!("invalid balance_blinding"))?,
        })
    }
}
