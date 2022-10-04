use std::{
    fmt::{self, Display},
    str::FromStr,
};

use decaf377_rdsa::{Signature, SpendAuth};
use penumbra_crypto::{GovernanceKey, IdentityKey};
use penumbra_proto::{
    core::governance::v1alpha1 as pb_g, core::transaction::v1alpha1 as pb_t, Protobuf,
};
use serde::{Deserialize, Serialize};

use crate::{ActionView, IsAction, TransactionPerspective};

/// A vote on a proposal.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(try_from = "pb_g::Vote", into = "pb_g::Vote")]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
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
        let pb_vote = pb_g::vote::Vote::from_str(s)?;
        let vote = pb_g::Vote {
            vote: pb_vote as i32,
        }
        .try_into()?;
        Ok(vote)
    }
}

impl Display for Vote {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        pb_g::vote::Vote::from_i32(pb_g::Vote::from(*self).vote)
            .unwrap()
            .fmt(f)
    }
}

impl From<Vote> for pb_g::Vote {
    fn from(value: Vote) -> Self {
        pb_from_vote(value)
    }
}

// Factored out so it can be used in a const
const fn pb_from_vote(vote: Vote) -> pb_g::Vote {
    match vote {
        Vote::Yes => pb_g::Vote {
            vote: pb_g::vote::Vote::Yes as i32,
        },
        Vote::No => pb_g::Vote {
            vote: pb_g::vote::Vote::No as i32,
        },
        Vote::Abstain => pb_g::Vote {
            vote: pb_g::vote::Vote::Abstain as i32,
        },
        Vote::NoWithVeto => pb_g::Vote {
            vote: pb_g::vote::Vote::NoWithVeto as i32,
        },
    }
}

impl TryFrom<pb_g::Vote> for Vote {
    type Error = anyhow::Error;

    fn try_from(msg: pb_g::Vote) -> Result<Self, Self::Error> {
        Ok(
            match pb_g::vote::Vote::from_i32(msg.vote)
                .ok_or_else(|| anyhow::anyhow!("invalid vote"))?
            {
                pb_g::vote::Vote::Abstain => Vote::Abstain,
                pb_g::vote::Vote::Yes => Vote::Yes,
                pb_g::vote::Vote::No => Vote::No,
                pb_g::vote::Vote::NoWithVeto => Vote::NoWithVeto,
            },
        )
    }
}

#[cfg(test)]
mod test {
    use proptest::proptest;

    proptest! {
        #[test]
        fn vote_roundtrip_serialize(vote: super::Vote) {
            let pb_vote: super::pb_g::Vote = vote.into();
            let vote2 = super::Vote::try_from(pb_vote).unwrap();
            assert_eq!(vote, vote2);
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
    /// The signature authorizing the vote (signed with governance key over the body).
    pub auth_sig: Signature<SpendAuth>,
}

impl IsAction for ValidatorVote {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        Default::default()
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::ValidatorVote(self.to_owned())
    }
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
    /// The governance key for the validator who is voting.
    pub governance_key: GovernanceKey,
}

impl From<ValidatorVoteBody> for pb_t::ValidatorVoteBody {
    fn from(value: ValidatorVoteBody) -> Self {
        pb_t::ValidatorVoteBody {
            proposal: value.proposal,
            vote: Some(value.vote.into()),
            identity_key: Some(value.identity_key.into()),
            governance_key: Some(value.governance_key.into()),
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
            governance_key: msg
                .governance_key
                .ok_or_else(|| {
                    anyhow::anyhow!("missing validator governance key in `ValidatorVote`")
                })?
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
