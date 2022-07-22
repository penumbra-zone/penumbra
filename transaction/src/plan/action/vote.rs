use decaf377::{FieldExt, Fr};
use penumbra_crypto::{IdentityKey, Note};
use penumbra_proto::{transaction as pb, Protobuf};
use penumbra_tct as tct;
use serde::{Deserialize, Serialize};

/// A vote on a proposal.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb::Vote", into = "pb::Vote")]
pub enum Vote {
    /// The vote is to approve the proposal.
    Yes,
    /// The vote is to reject the proposal.
    No,
    /// The vote is to abstain from the proposal.
    Abstain,
    /// The vote is to reject the proposal, and burn the deposit of the proposer.
    NoWithVeto,
}

impl From<Vote> for pb::Vote {
    fn from(value: Vote) -> Self {
        match value {
            Vote::Yes => pb::Vote {
                vote: Some(pb::vote::Vote::Yes(pb::vote::Yes {})),
            },
            Vote::No => pb::Vote {
                vote: Some(pb::vote::Vote::No(pb::vote::No {})),
            },
            Vote::Abstain => pb::Vote {
                vote: Some(pb::vote::Vote::Abstain(pb::vote::Abstain {})),
            },
            Vote::NoWithVeto => pb::Vote {
                vote: Some(pb::vote::Vote::NoWithVeto(pb::vote::NoWithVeto {})),
            },
        }
    }
}

impl TryFrom<pb::Vote> for Vote {
    type Error = anyhow::Error;

    fn try_from(msg: pb::Vote) -> Result<Self, Self::Error> {
        match msg.vote {
            Some(pb::vote::Vote::Yes(_)) => Ok(Vote::Yes),
            Some(pb::vote::Vote::No(_)) => Ok(Vote::No),
            Some(pb::vote::Vote::Abstain(_)) => Ok(Vote::Abstain),
            Some(pb::vote::Vote::NoWithVeto(_)) => Ok(Vote::NoWithVeto),
            None => Err(anyhow::anyhow!("missing vote in `Vote`")),
        }
    }
}

impl Protobuf<pb::Vote> for Vote {}

/// A plan to vote as a validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorVotePlan", into = "pb::ValidatorVotePlan")]
pub struct ValidatorVotePlan {
    /// The proposal ID to vote on.
    pub proposal: u64,
    /// The vote to cast.
    pub vote: Vote,
    /// The identity of the validator who is voting.
    pub validator_identity: IdentityKey,
}

impl From<ValidatorVotePlan> for pb::ValidatorVotePlan {
    fn from(value: ValidatorVotePlan) -> Self {
        pb::ValidatorVotePlan {
            proposal: value.proposal,
            vote: Some(value.vote.into()),
            validator_identity: Some(value.validator_identity.into()),
        }
    }
}

impl TryFrom<pb::ValidatorVotePlan> for ValidatorVotePlan {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ValidatorVotePlan) -> Result<Self, Self::Error> {
        Ok(ValidatorVotePlan {
            proposal: msg.proposal,
            vote: msg
                .vote
                .ok_or_else(|| anyhow::anyhow!("missing vote in `ValidatorVotePlan`"))?
                .try_into()?,
            validator_identity: msg
                .validator_identity
                .ok_or_else(|| {
                    anyhow::anyhow!("missing validator identity in `ValidatorVotePlan`")
                })?
                .try_into()?,
        })
    }
}

impl Protobuf<pb::ValidatorVotePlan> for ValidatorVotePlan {}

/// A plan to vote as a delegator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::DelegatorVotePlan", into = "pb::DelegatorVotePlan")]
pub struct DelegatorVotePlan {
    /// The proposal ID to vote on.
    pub proposal: u64,
    /// The vote to cast.
    pub vote: Vote,
    /// A staked note that was spendable before the proposal started.
    pub staked_note: Note,
    /// The position of the staked note.
    pub position: tct::Position,
    /// The randomizer to use.
    pub randomizer: Fr,
}

impl From<DelegatorVotePlan> for pb::DelegatorVotePlan {
    fn from(inner: DelegatorVotePlan) -> Self {
        pb::DelegatorVotePlan {
            proposal: inner.proposal,
            vote: Some(inner.vote.into()),
            staked_note: Some(inner.staked_note.into()),
            position: inner.position.into(),
            randomizer: inner.randomizer.to_bytes().to_vec().into(),
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
            staked_note: value
                .staked_note
                .ok_or_else(|| anyhow::anyhow!("missing staked_note in `DelegatorVotePlan`"))?
                .try_into()?,
            position: value.position.into(),
            randomizer: Fr::from_bytes(value.randomizer.as_ref().try_into()?)?,
        })
    }
}

impl Protobuf<pb::DelegatorVotePlan> for DelegatorVotePlan {}
