use std::ops::{Add, AddAssign};

use serde::{Deserialize, Serialize};

use penumbra_chain::params::{ChainParameters, Ratio};
use penumbra_proto::{core::governance::v1alpha1 as pb, DomainType};
use penumbra_transaction::{
    action::Vote,
    proposal::{self, Withdrawn},
};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "pb::Tally", into = "pb::Tally")]
pub struct Tally {
    yes: u64,
    no: u64,
    abstain: u64,
}

impl Tally {
    pub fn yes(&self) -> u64 {
        self.yes
    }

    pub fn no(&self) -> u64 {
        self.no
    }

    pub fn abstain(&self) -> u64 {
        self.abstain
    }

    pub fn total(&self) -> u64 {
        self.yes + self.no + self.abstain
    }
}

impl From<Tally> for pb::Tally {
    fn from(tally: Tally) -> Self {
        Self {
            yes: tally.yes,
            no: tally.no,
            abstain: tally.abstain,
        }
    }
}

impl From<pb::Tally> for Tally {
    fn from(tally: pb::Tally) -> Self {
        Self {
            yes: tally.yes,
            no: tally.no,
            abstain: tally.abstain,
        }
    }
}

impl DomainType for Tally {
    type Proto = pb::Tally;
}

impl From<(Vote, u64)> for Tally {
    fn from((vote, power): (Vote, u64)) -> Self {
        let mut tally = Self::default();
        *match vote {
            Vote::Yes => &mut tally.yes,
            Vote::No => &mut tally.no,
            Vote::Abstain => &mut tally.abstain,
        } = power;
        tally
    }
}

impl From<(u64, Vote)> for Tally {
    fn from((power, vote): (u64, Vote)) -> Self {
        Self::from((vote, power))
    }
}

impl Add for Tally {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            yes: self.yes + rhs.yes,
            no: self.no + rhs.no,
            abstain: self.abstain + rhs.abstain,
        }
    }
}

impl AddAssign for Tally {
    fn add_assign(&mut self, rhs: Self) {
        self.yes += rhs.yes;
        self.no += rhs.no;
        self.abstain += rhs.abstain;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Fail,
    Slash,
}

impl Outcome {
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, Self::Fail)
    }

    pub fn is_slash(&self) -> bool {
        matches!(self, Self::Slash)
    }
}

impl<T> From<Outcome> for proposal::Outcome<T> {
    fn from(outcome: Outcome) -> Self {
        match outcome {
            Outcome::Pass => Self::Passed,
            Outcome::Fail => Self::Failed {
                withdrawn: Withdrawn::No,
            },
            Outcome::Slash => Self::Slashed {
                withdrawn: Withdrawn::No,
            },
        }
    }
}

impl Tally {
    fn meets_quorum(&self, total_voting_power: u64, params: &ChainParameters) -> bool {
        Ratio::new(self.total(), total_voting_power) >= params.proposal_valid_quorum
    }

    fn slashed(&self, params: &ChainParameters) -> bool {
        Ratio::new(self.no, self.total()) > params.proposal_slash_threshold
    }

    fn yes_ratio(&self) -> Ratio {
        Ratio::new(self.yes, (self.yes + self.no).min(1))
        // ^ in the above, the `.min(1)` is to prevent a divide-by-zero error when the only votes
        // cast are abstains -- this results in a 0:1 ratio in that case, which will never pass, as
        // desired in that situation
    }

    pub fn outcome(self, total_voting_power: u64, params: &ChainParameters) -> Outcome {
        use Outcome::*;

        // Check to see if we've met quorum
        if !self.meets_quorum(total_voting_power, params) {
            return Fail;
        }

        // Check to see if it has been slashed
        if self.slashed(params) {
            return Slash;
        }

        // Now that we've checked for slash and quorum, we can just check to see if it should pass
        if self.yes_ratio() > params.proposal_pass_threshold {
            Pass
        } else {
            Fail
        }
    }

    pub fn emergency_pass(self, total_voting_power: u64, params: &ChainParameters) -> bool {
        // Check to see if we've met quorum
        if !self.meets_quorum(total_voting_power, params) {
            return false;
        }

        // Check to see if it has been slashed (this check should be redundant, but we'll do it anyway)
        if self.slashed(params) {
            return false;
        }

        // Now that we've checked for slash and quorum, we can just check to see if it should pass in
        // the emergency condition of 2/3 majority
        self.yes_ratio() > Ratio::new(2, 3)
    }
}
