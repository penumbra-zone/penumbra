use anyhow::Context;

use crate::{vote::Vote, DelegatorVoteProof};
use decaf377_rdsa::{Signature, SpendAuth, VerificationKey};
use penumbra_asset::Value;
use penumbra_num::Amount;
use penumbra_proto::{core::component::governance::v1alpha1 as pb, DomainType};
use penumbra_sct::Nullifier;
use penumbra_tct as tct;
use penumbra_txhash::{EffectHash, EffectingData};

#[derive(Debug, Clone)]
pub struct DelegatorVote {
    pub body: DelegatorVoteBody,
    pub auth_sig: Signature<SpendAuth>,
    pub proof: DelegatorVoteProof,
}

impl EffectingData for DelegatorVote {
    fn effect_hash(&self) -> EffectHash {
        self.body.effect_hash()
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

impl EffectingData for DelegatorVoteBody {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl From<DelegatorVoteBody> for pb::DelegatorVoteBody {
    fn from(value: DelegatorVoteBody) -> Self {
        pb::DelegatorVoteBody {
            proposal: value.proposal,
            start_position: value.start_position.into(),
            vote: Some(value.vote.into()),
            value: Some(value.value.into()),
            unbonded_amount: Some(value.unbonded_amount.into()),
            nullifier: Some(value.nullifier.into()),
            rk: Some(value.rk.into()),
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
                .ok_or_else(|| anyhow::anyhow!("missing nullifier in `DelegatorVote`"))?
                .try_into()
                .context("invalid nullifier in `DelegatorVote`")?,
            rk: msg
                .rk
                .ok_or_else(|| anyhow::anyhow!("missing rk in `DelegatorVote`"))?
                .try_into()
                .context("invalid rk in `DelegatorVote`")?,
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
