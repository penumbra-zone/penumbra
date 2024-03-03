use decaf377::{Fq, Fr};
use penumbra_asset::{asset, balance, Balance};
use penumbra_num::Amount;
use penumbra_proof_params::CONVERT_PROOF_PROVING_KEY;
use penumbra_proto::{penumbra::core::component::stake::v1 as pb, DomainType};

use serde::{Deserialize, Serialize};

use crate::{
    IdentityKey, Penalty, UnbondingToken, UndelegateClaim, UndelegateClaimBody,
    UndelegateClaimProof,
};

use super::UndelegateClaimProofPublic;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::UndelegateClaimPlan", into = "pb::UndelegateClaimPlan")]
pub struct UndelegateClaimPlan {
    /// The identity key of the validator to undelegate from.
    pub validator_identity: IdentityKey,
    /// The epoch in which unbonding began, used to verify the penalty.
    pub start_epoch_index: u64,
    /// The penalty applied to undelegation, in bps^2.
    pub penalty: Penalty,
    /// The amount of unbonding tokens to claim. This is a bare number because its denom is determined by the preceding data.
    pub unbonding_amount: Amount,
    /// The blinding factor that will be used for the balance commitment.
    /// The action's contribution to the transaction's value balance.
    pub balance_blinding: Fr,
    /// The first blinding factor used for generating the ZK proof.
    pub proof_blinding_r: Fq,
    /// The second blinding factor used for generating the ZK proof.
    pub proof_blinding_s: Fq,
}

impl UndelegateClaimPlan {
    /// Convenience method to construct the [`UndelegateClaim`] described by this [`UndelegateClaimPlan`].
    pub fn undelegate_claim(&self) -> UndelegateClaim {
        UndelegateClaim {
            body: self.undelegate_claim_body(),
            proof: self.undelegate_claim_proof(),
        }
    }

    /// Construct the [`UndelegateClaimBody`] described by this [`UndelegateClaimPlan`].
    pub fn undelegate_claim_body(&self) -> UndelegateClaimBody {
        UndelegateClaimBody {
            validator_identity: self.validator_identity,
            start_epoch_index: self.start_epoch_index,
            penalty: self.penalty,
            balance_commitment: self.balance().commit(self.balance_blinding),
        }
    }

    /// Construct the [`UndelegateClaimProof`] required by the [`UndelegateClaimBody`] described by this [`UndelegateClaimPlan`].
    pub fn undelegate_claim_proof(&self) -> UndelegateClaimProof {
        UndelegateClaimProof::prove(
            self.proof_blinding_r,
            self.proof_blinding_s,
            &CONVERT_PROOF_PROVING_KEY,
            UndelegateClaimProofPublic {
                balance_commitment: self.balance_commitment(),
                unbonding_id: self.unbonding_id(),
                penalty: self.penalty,
            },
            super::UndelegateClaimProofPrivate {
                unbonding_amount: self.unbonding_amount,
                balance_blinding: self.balance_blinding,
            },
        )
        .expect("can generate undelegate claim proof")
    }

    pub fn unbonding_token(&self) -> UnbondingToken {
        UnbondingToken::new(self.validator_identity, self.start_epoch_index)
    }

    pub fn unbonding_id(&self) -> asset::Id {
        self.unbonding_token().id()
    }

    pub fn balance_commitment(&self) -> balance::Commitment {
        self.balance().commit(self.balance_blinding)
    }

    pub fn balance(&self) -> Balance {
        self.penalty
            .balance_for_claim(self.unbonding_id(), self.unbonding_amount)
    }
}

impl DomainType for UndelegateClaimPlan {
    type Proto = pb::UndelegateClaimPlan;
}

impl From<UndelegateClaimPlan> for pb::UndelegateClaimPlan {
    fn from(msg: UndelegateClaimPlan) -> Self {
        Self {
            validator_identity: Some(msg.validator_identity.into()),
            start_epoch_index: msg.start_epoch_index,
            penalty: Some(msg.penalty.into()),
            unbonding_amount: Some(msg.unbonding_amount.into()),
            balance_blinding: msg.balance_blinding.to_bytes().to_vec(),
            proof_blinding_r: msg.proof_blinding_r.to_bytes().to_vec(),
            proof_blinding_s: msg.proof_blinding_s.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::UndelegateClaimPlan> for UndelegateClaimPlan {
    type Error = anyhow::Error;
    fn try_from(msg: pb::UndelegateClaimPlan) -> Result<Self, Self::Error> {
        let proof_blinding_r_bytes: [u8; 32] = msg
            .proof_blinding_r
            .try_into()
            .map_err(|_| anyhow::anyhow!("malformed r in `UndelegateClaimPlan`"))?;
        let proof_blinding_s_bytes: [u8; 32] = msg
            .proof_blinding_s
            .try_into()
            .map_err(|_| anyhow::anyhow!("malformed s in `UndelegateClaimPlan`"))?;

        Ok(Self {
            validator_identity: msg
                .validator_identity
                .ok_or_else(|| anyhow::anyhow!("missing validator_identity"))?
                .try_into()?,
            start_epoch_index: msg.start_epoch_index,
            penalty: msg
                .penalty
                .ok_or_else(|| anyhow::anyhow!("missing penalty"))?
                .try_into()?,
            unbonding_amount: msg
                .unbonding_amount
                .ok_or_else(|| anyhow::anyhow!("missing unbonding_amount"))?
                .try_into()?,
            balance_blinding: Fr::from_bytes_checked(
                msg.balance_blinding
                    .as_slice()
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("expected 32 bytes"))?,
            )
            .map_err(|_| anyhow::anyhow!("invalid balance_blinding"))?,
            proof_blinding_r: Fq::from_bytes_checked(&proof_blinding_r_bytes)
                .expect("proof_blinding_r malformed"),
            proof_blinding_s: Fq::from_bytes_checked(&proof_blinding_s_bytes)
                .expect("proof_blinding_s malformed"),
        })
    }
}
