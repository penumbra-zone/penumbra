use std::str::FromStr;

use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_crypto::IdentityKey;
use penumbra_proto::{governance as pb_g, transaction as pb_t, Protobuf};
use serde::{Deserialize, Serialize};

/// A vote on a proposal.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(try_from = "pb_g::Vote", into = "pb_g::Vote")]
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

impl FromStr for Vote {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s
            .replace('-', "")
            .replace('_', "")
            .replace(' ', "")
            .to_lowercase()
            .as_str()
        {
            "yes" => Ok(Vote::Yes),
            "no" => Ok(Vote::No),
            "abstain" => Ok(Vote::Abstain),
            "veto" | "noveto" | "nowithveto" => Ok(Vote::NoWithVeto),
            _ => Err(anyhow::anyhow!("invalid vote: {}", s)),
        }
    }
}

impl From<Vote> for pb_g::Vote {
    fn from(value: Vote) -> Self {
        match value {
            Vote::Yes => pb_g::Vote {
                vote: Some(pb_g::vote::Vote::Yes(pb_g::vote::Yes {})),
            },
            Vote::No => pb_g::Vote {
                vote: Some(pb_g::vote::Vote::No(pb_g::vote::No {})),
            },
            Vote::Abstain => pb_g::Vote {
                vote: Some(pb_g::vote::Vote::Abstain(pb_g::vote::Abstain {})),
            },
            Vote::NoWithVeto => pb_g::Vote {
                vote: Some(pb_g::vote::Vote::NoWithVeto(pb_g::vote::NoWithVeto {})),
            },
        }
    }
}

impl TryFrom<pb_g::Vote> for Vote {
    type Error = anyhow::Error;

    fn try_from(msg: pb_g::Vote) -> Result<Self, Self::Error> {
        match msg.vote {
            Some(pb_g::vote::Vote::Yes(_)) => Ok(Vote::Yes),
            Some(pb_g::vote::Vote::No(_)) => Ok(Vote::No),
            Some(pb_g::vote::Vote::Abstain(_)) => Ok(Vote::Abstain),
            Some(pb_g::vote::Vote::NoWithVeto(_)) => Ok(Vote::NoWithVeto),
            None => Err(anyhow::anyhow!("missing vote in `Vote`")),
        }
    }
}

impl Protobuf<pb_g::Vote> for Vote {}

/// A vote by a validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb_t::ValidatorVote", into = "pb_t::ValidatorVote")]
pub struct ValidatorVote {
    /// The body of the validator vote.
    pub body: ValidatorVoteBody,
    /// The signature authorizing the vote.
    pub auth_sig: Signature<SpendAuth>,
}

impl From<ValidatorVote> for pb_t::ValidatorVote {
    fn from(msg: ValidatorVote) -> Self {
        Self {
            body: Some(msg.body.into()),
            auth_sig: Some(msg.auth_sig.into()),
        }
    }
}

impl TryFrom<pb_t::ValidatorVote> for ValidatorVote {
    type Error = anyhow::Error;

    fn try_from(msg: pb_t::ValidatorVote) -> Result<Self, Self::Error> {
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
#[serde(try_from = "pb_t::ValidatorVoteBody", into = "pb_t::ValidatorVoteBody")]
pub struct ValidatorVoteBody {
    /// The proposal ID to vote on.
    pub proposal: u64,
    /// The vote to cast.
    pub vote: Vote,
    /// The identity of the validator who is voting.
    pub identity_key: IdentityKey,
}

impl From<ValidatorVoteBody> for pb_t::ValidatorVoteBody {
    fn from(value: ValidatorVoteBody) -> Self {
        pb_t::ValidatorVoteBody {
            proposal: value.proposal,
            vote: Some(value.vote.into()),
            identity_key: Some(value.identity_key.into()),
        }
    }
}

impl TryFrom<pb_t::ValidatorVoteBody> for ValidatorVoteBody {
    type Error = anyhow::Error;

    fn try_from(msg: pb_t::ValidatorVoteBody) -> Result<Self, Self::Error> {
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

impl Protobuf<pb_t::ValidatorVoteBody> for ValidatorVoteBody {}

#[derive(Debug, Clone)]
pub struct DelegatorVote {
    // TODO: fill this in
    pub body: DelegatorVoteBody,
}

#[derive(Debug, Clone)]
pub struct DelegatorVoteBody {
    // TODO: fill this in
}
