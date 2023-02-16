use ark_ff::Zero;
use decaf377::{FieldExt, Fr};
use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_crypto::{
    proofs::transparent::{DelegatorVoteProof, SpendProof},
    Amount, FullViewingKey, Note, VotingReceiptToken,
};
use penumbra_proto::{core::governance::v1alpha1 as pb, DomainType};
use penumbra_tct as tct;
use serde::{Deserialize, Serialize};

use crate::action::{DelegatorVote, DelegatorVoteBody};

/// A plan to vote as a delegator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::DelegatorVotePlan", into = "pb::DelegatorVotePlan")]
pub struct DelegatorVotePlan {
    /// The proposal ID to vote on.
    pub proposal: u64,
    /// The start height of the proposal.
    pub start_height: tct::Position,
    /// The vote to cast.
    pub vote: crate::action::Vote,
    /// A staked note that was spendable before the proposal started.
    pub staked_note: Note,
    /// The unbonded amount corresponding to the staked note.
    pub unbonded_amount: Amount,
    /// The position of the staked note.
    pub position: tct::Position,
    /// The randomizer to use.
    pub randomizer: Fr,
}

impl DelegatorVotePlan {
    /// Convenience method to construct the [`DelegatorVote`] described by this [`DelegatorVotePlan`].
    pub fn delegator_vote(
        &self,
        fvk: &FullViewingKey,
        auth_sig: Signature<SpendAuth>,
        auth_path: tct::Proof,
    ) -> DelegatorVote {
        DelegatorVote {
            body: self.delegator_vote_body(fvk),
            auth_sig,
            proof: self.delegator_vote_proof(fvk, auth_path),
        }
    }

    /// Construct the [`DelegatorVoteBody`] described by this [`DelegatorVotePlan`].
    pub fn delegator_vote_body(&self, fvk: &FullViewingKey) -> DelegatorVoteBody {
        DelegatorVoteBody {
            proposal: self.proposal,
            start_height: self.start_height,
            vote: self.vote,
            value: self.staked_note.value(),
            unbonded_amount: self.unbonded_amount,
            nullifier: fvk.derive_nullifier(self.position, &self.staked_note.commit()),
            rk: fvk.spend_verification_key().randomize(&self.randomizer),
        }
    }

    /// Construct the [`DelegatorVoteProof`] required by the [`DelegatorVoteBody`] described by this [`DelegatorVotePlan`].
    pub fn delegator_vote_proof(
        &self,
        fvk: &FullViewingKey,
        state_commitment_proof: tct::Proof,
    ) -> DelegatorVoteProof {
        DelegatorVoteProof {
            spend_proof: SpendProof {
                state_commitment_proof,
                note: self.staked_note.clone(),
                v_blinding: Fr::zero(),
                spend_auth_randomizer: self.randomizer,
                ak: *fvk.spend_verification_key(),
                nk: *fvk.nullifier_key(),
            },
        }
    }

    pub fn balance(&self) -> penumbra_crypto::Balance {
        penumbra_crypto::Value {
            amount: self.unbonded_amount,
            asset_id: VotingReceiptToken::new(self.proposal).id(),
        }
        .into()
    }
}

impl From<DelegatorVotePlan> for pb::DelegatorVotePlan {
    fn from(inner: DelegatorVotePlan) -> Self {
        pb::DelegatorVotePlan {
            proposal: inner.proposal,
            vote: Some(inner.vote.into()),
            start_height: inner.start_height.into(),
            staked_note: Some(inner.staked_note.into()),
            unbonded_amount: Some(inner.unbonded_amount.into()),
            position: inner.position.into(),
            randomizer: inner.randomizer.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::DelegatorVotePlan> for DelegatorVotePlan {
    type Error = anyhow::Error;

    fn try_from(value: pb::DelegatorVotePlan) -> Result<Self, Self::Error> {
        Ok(DelegatorVotePlan {
            proposal: value.proposal,
            vote: value
                .vote
                .ok_or_else(|| anyhow::anyhow!("missing vote in `DelegatorVotePlan`"))?
                .try_into()?,
            start_height: value.start_height.try_into()?,
            staked_note: value
                .staked_note
                .ok_or_else(|| anyhow::anyhow!("missing staked note in `DelegatorVotePlan`"))?
                .try_into()?,
            unbonded_amount: value
                .unbonded_amount
                .ok_or_else(|| anyhow::anyhow!("missing unbonded amount in `DelegatorVotePlan`"))?
                .try_into()?,
            position: value.position.into(),
            randomizer: Fr::from_bytes(
                value
                    .randomizer
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("invalid randomizer"))?,
            )?,
        })
    }
}

impl DomainType for DelegatorVotePlan {
    type Proto = pb::DelegatorVotePlan;
}
