use decaf377_rdsa::{Signature, SpendAuth};
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

/// A vote by a validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorVote", into = "pb::ValidatorVote")]
pub struct ValidatorVote {
    /// The body of the validator vote.
    pub body: ValidatorVoteBody,
    /// The signature authorizing the vote.
    pub auth_sig: Signature<SpendAuth>,
}

impl From<ValidatorVote> for pb::ValidatorVote {
    fn from(msg: ValidatorVote) -> Self {
        Self {
            body: Some(msg.body.into()),
            auth_sig: Some(msg.auth_sig.into()),
        }
    }
}

impl TryFrom<pb::ValidatorVote> for ValidatorVote {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ValidatorVote) -> Result<Self, Self::Error> {
        Ok(Self {
            body: msg
                .body
                .ok_or_else(|| anyhow::anyhow!("missing validator vote body"))?
                .try_into()?,
            auth_sig: msg
                .auth_sig
                .ok_or_else(|| anyhow::anyhow!("missing validator auth sig"))?
                .try_into()?,
        })
    }
}

/// A public vote as a validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ValidatorVoteBody", into = "pb::ValidatorVoteBody")]
pub struct ValidatorVoteBody {
    /// The proposal ID to vote on.
    pub proposal: u64,
    /// The vote to cast.
    pub vote: Vote,
    /// The identity of the validator who is voting.
    pub identity_key: IdentityKey,
}

impl From<ValidatorVoteBody> for pb::ValidatorVoteBody {
    fn from(value: ValidatorVoteBody) -> Self {
        pb::ValidatorVoteBody {
            proposal: value.proposal,
            vote: Some(value.vote.into()),
            identity_key: Some(value.identity_key.into()),
        }
    }
}

impl TryFrom<pb::ValidatorVoteBody> for ValidatorVoteBody {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ValidatorVoteBody) -> Result<Self, Self::Error> {
        Ok(ValidatorVoteBody {
            proposal: msg.proposal,
            vote: msg
                .vote
                .ok_or_else(|| anyhow::anyhow!("missing vote in `ValidatorVote`"))?
                .try_into()?,
            identity_key: msg
                .identity_key
                .ok_or_else(|| anyhow::anyhow!("missing validator identity in `ValidatorVote`"))?
                .try_into()?,
        })
    }
}

impl Protobuf<pb::ValidatorVoteBody> for ValidatorVoteBody {}

#[derive(Debug, Clone)]
pub struct DelegatorVote {
    // TODO: fill this in
    pub body: DelegatorVoteBody,
}

#[derive(Debug, Clone)]
pub struct DelegatorVoteBody {
    // TODO: fill this in
}
