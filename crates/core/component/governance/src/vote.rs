use std::{
    fmt::{self, Display},
    str::FromStr,
};

use anyhow::anyhow;
use penumbra_sdk_proto::{penumbra::core::component::governance::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/// A vote on a proposal.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(try_from = "pb::Vote", into = "pb::Vote")]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum Vote {
    /// Vote to approve the proposal.
    Yes,
    /// Vote is to reject the proposal.
    No,
    /// Vote to abstain from the proposal.
    Abstain,
}

impl FromStr for Vote {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Vote> {
        match s {
            "yes" | "y" => Ok(Vote::Yes),
            "no" | "n" => Ok(Vote::No),
            "abstain" | "a" => Ok(Vote::Abstain),
            _ => Err(anyhow::anyhow!("invalid vote: {}", s)),
        }
    }
}

impl Display for Vote {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Vote::Yes => write!(f, "yes"),
            Vote::No => write!(f, "no"),
            Vote::Abstain => write!(f, "abstain"),
        }
    }
}

impl From<Vote> for pb::Vote {
    fn from(value: Vote) -> Self {
        pb_from_vote(value)
    }
}

// Factored out so it can be used in a const
const fn pb_from_vote(vote: Vote) -> pb::Vote {
    match vote {
        Vote::Yes => pb::Vote {
            vote: pb::vote::Vote::Yes as i32,
        },
        Vote::No => pb::Vote {
            vote: pb::vote::Vote::No as i32,
        },
        Vote::Abstain => pb::Vote {
            vote: pb::vote::Vote::Abstain as i32,
        },
    }
}

impl TryFrom<pb::Vote> for Vote {
    type Error = anyhow::Error;

    fn try_from(msg: pb::Vote) -> Result<Self, Self::Error> {
        let vote_state = pb::vote::Vote::try_from(msg.vote)
            .map_err(|e| anyhow::anyhow!("invalid vote state, error: {e}"))?;

        match vote_state {
            pb::vote::Vote::Abstain => Ok(Vote::Abstain),
            pb::vote::Vote::Yes => Ok(Vote::Yes),
            pb::vote::Vote::No => Ok(Vote::No),
            pb::vote::Vote::Unspecified => Err(anyhow!("unspecified vote state")),
        }
    }
}

#[cfg(test)]
mod test {
    use proptest::proptest;

    proptest! {
        #[test]
        fn vote_roundtrip_serialize(vote: super::Vote) {
            let pb_vote: super::pb::Vote = vote.into();
            let vote2 = super::Vote::try_from(pb_vote).unwrap();
            assert_eq!(vote, vote2);
        }
    }
}

impl DomainType for Vote {
    type Proto = pb::Vote;
}
