use anyhow::Context;
use ark_ff::Zero;
use decaf377::Fr;
use decaf377_rdsa::{Signature, SpendAuth, VerificationKey};
use penumbra_crypto::{
    proofs::groth16::DelegatorVoteProof, Amount, Nullifier, Value, VotingReceiptToken,
};
use penumbra_proto::{core::governance::v1alpha1 as pb, DomainType};
use penumbra_tct as tct;

use crate::{
    view::action_view::DelegatorVoteView, vote::Vote, Action, ActionView, IsAction,
    TransactionPerspective,
};

#[derive(Debug, Clone)]
pub struct DelegatorVote {
    pub body: DelegatorVoteBody,
    pub auth_sig: Signature<SpendAuth>,
    pub proof: DelegatorVoteProof,
}

impl IsAction for DelegatorVote {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        Value {
            asset_id: VotingReceiptToken::new(self.body.proposal).id(),
            amount: self.body.unbonded_amount,
        }
        .commit(Fr::zero())
    }

    fn view_from_perspective(&self, txp: &TransactionPerspective) -> ActionView {
        let delegator_vote_view = match txp.spend_nullifiers.get(&self.body.nullifier) {
            Some(note) => DelegatorVoteView::Visible {
                delegator_vote: self.to_owned(),
                note: txp.view_note(note.to_owned()),
            },
            None => DelegatorVoteView::Opaque {
                delegator_vote: self.to_owned(),
            },
        };

        ActionView::DelegatorVote(delegator_vote_view)
    }
}

/// The body of a delegator vote.
#[derive(Debug, Clone)]
pub struct DelegatorVoteBody {
    /// The proposal ID the vote is for.
    pub proposal: u64,
    /// The start position of the proposal in the TCT.
    pub start_position: tct::Position,
    /// The vote on the proposal.
    pub vote: Vote, // With flow encryption, this will be a triple of flow ciphertexts
    /// The value of the staked note being used to vote.
    pub value: Value, // With flow encryption, this will be a triple of balance commitments, and a public denomination
    /// The unbonded amount equivalent to the value above
    pub unbonded_amount: Amount,
    /// The nullifier of the staked note being used to vote.
    pub nullifier: Nullifier,
    /// The randomized validating key for the spend authorization signature.
    pub rk: VerificationKey<SpendAuth>,
}

impl From<DelegatorVoteBody> for pb::DelegatorVoteBody {
    fn from(value: DelegatorVoteBody) -> Self {
        pb::DelegatorVoteBody {
            proposal: value.proposal,
            start_position: value.start_position.into(),
            vote: Some(value.vote.into()),
            value: Some(value.value.into()),
            unbonded_amount: Some(value.unbonded_amount.into()),
            nullifier: value.nullifier.to_bytes().into(),
            rk: value.rk.to_bytes().into(),
        }
    }
}

impl TryFrom<pb::DelegatorVoteBody> for DelegatorVoteBody {
    type Error = anyhow::Error;

    fn try_from(msg: pb::DelegatorVoteBody) -> Result<Self, Self::Error> {
        Ok(DelegatorVoteBody {
            proposal: msg.proposal,
            start_position: msg
                .start_position
                .try_into()
                .context("invalid start position in `DelegatorVote`")?,
            vote: msg
                .vote
                .ok_or_else(|| anyhow::anyhow!("missing vote in `DelegatorVote`"))?
                .try_into()?,
            value: msg
                .value
                .ok_or_else(|| anyhow::anyhow!("missing value in `DelegatorVote`"))?
                .try_into()?,
            unbonded_amount: msg
                .unbonded_amount
                .ok_or_else(|| anyhow::anyhow!("missing unbonded amount in `DelegatorVote`"))?
                .try_into()?,
            nullifier: msg
                .nullifier
                .try_into()
                .context("invalid nullifier in `DelegatorVote`")?,
            rk: {
                let rk_bytes: [u8; 32] = (msg.rk[..])
                    .try_into()
                    .context("expected 32-byte rk in `DelegatorVote`")?;
                rk_bytes
                    .try_into()
                    .context("invalid  rk in `DelegatorVote`")?
            },
        })
    }
}

impl DomainType for DelegatorVoteBody {
    type Proto = pb::DelegatorVoteBody;
}

impl From<DelegatorVote> for pb::DelegatorVote {
    fn from(value: DelegatorVote) -> Self {
        pb::DelegatorVote {
            body: Some(value.body.into()),
            auth_sig: Some(value.auth_sig.into()),
            proof: Some(value.proof.into()),
        }
    }
}

impl TryFrom<pb::DelegatorVote> for DelegatorVote {
    type Error = anyhow::Error;

    fn try_from(msg: pb::DelegatorVote) -> Result<Self, Self::Error> {
        Ok(DelegatorVote {
            body: msg
                .body
                .ok_or_else(|| anyhow::anyhow!("missing body in `DelegatorVote`"))?
                .try_into()?,
            auth_sig: msg
                .auth_sig
                .ok_or_else(|| anyhow::anyhow!("missing auth sig in `DelegatorVote`"))?
                .try_into()?,
            proof: msg
                .proof
                .ok_or_else(|| anyhow::anyhow!("missing delegator vote proof"))?
                .try_into()
                .context("delegator vote proof malformed")?,
        })
    }
}

impl From<DelegatorVote> for Action {
    fn from(value: DelegatorVote) -> Self {
        Action::DelegatorVote(value)
    }
}
