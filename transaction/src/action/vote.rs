use penumbra_crypto::IdentityKey;
use penumbra_proto::{transaction as pb, Protobuf};
use serde::{Deserialize, Serialize};

/// A vote on a proposal.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
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
pub struct ValidatorVote {
    /// The proposal ID to vote on.
    pub proposal: u64,
    /// The vote to cast.
    pub vote: Vote,
    /// The identity of the validator who is voting.
    pub validator_identity: IdentityKey,
}

impl From<ValidatorVote> for pb::ValidatorVotePlan {
    fn from(value: ValidatorVote) -> Self {
        pb::ValidatorVotePlan {
            proposal: value.proposal,
            vote: Some(value.vote.into()),
            validator_identity: Some(value.validator_identity.into()),
        }
    }
}

impl TryFrom<pb::ValidatorVotePlan> for ValidatorVote {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ValidatorVotePlan) -> Result<Self, Self::Error> {
        Ok(ValidatorVote {
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

impl Protobuf<pb::ValidatorVotePlan> for ValidatorVote {}
