use std::ops::{Add, AddAssign};

// use anyhow::Result;
// use penumbra_chain::params::ChainParameters;
use penumbra_proto::{core::governance::v1alpha1 as pb, DomainType};
use penumbra_transaction::action::Vote;
use tendermint::vote::Power;

// use super::{proposal::Withdrawn, StateReadExt as _};
// use crate::stake::StateReadExt as _;
// use penumbra_chain::{params::Ratio, StateReadExt as _};
// use penumbra_storage::StateRead;

// use super::proposal::Outcome;

#[derive(Clone, Copy, Debug, Default)]
pub struct Tally {
    yes: u64,
    no: u64,
    abstain: u64,
}

impl Tally {
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
    fn from((vote, u64): (Vote, u64)) -> Self {
        let mut tally = Self::default();
        *match vote {
            Vote::Yes => &mut tally.yes,
            Vote::No => &mut tally.no,
            Vote::Abstain => &mut tally.abstain,
        } = u64.into();
        tally
    }
}

impl From<(u64, Vote)> for Tally {
    fn from((u64, vote): (u64, Vote)) -> Self {
        Self::from((vote, u64))
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

// /// The parameters used for tallying.
// #[derive(Debug, Clone)]
// pub struct Parameters {
//     pub valid_quorum: Ratio,
//     pub pass_threshold: Ratio,
//     pub veto_threshold: Ratio,
// }

// impl From<&ChainParameters> for Parameters {
//     fn from(params: &ChainParameters) -> Self {
//         Parameters {
//             valid_quorum: params.proposal_valid_quorum,
//             pass_threshold: params.proposal_pass_threshold,
//             veto_threshold: params.proposal_veto_threshold,
//         }
//     }
// }

// impl Parameters {
//     pub async fn new(state: impl StateRead) -> Result<Self> {
//         Ok(Parameters::from(&state.get_chain_params().await?))
//     }
// }

// /// Constants during all tallies for a given block.
// #[derive(Debug, Clone, Copy)]
// pub struct Circumstance {
//     pub total_voting_power: u64,
//     pub current_block: u64,
// }

// impl Circumstance {
//     pub async fn new(state: impl StateRead) -> Result<Self> {
//         Ok(Circumstance {
//             total_voting_power: state.total_voting_power().await?,
//             current_block: state.get_block_height().await?,
//         })
//     }
// }

// /// Intermediate structure used for tallying votes for a particular proposal.
// #[derive(Debug, Clone)]
// pub struct Tally {
//     yes: u64,
//     no: u64,
//     abstain: u64,
//     // Constant during tallying:
//     circumstance: Circumstance,
//     ending_block: u64,
//     withdrawn: Withdrawn<String>,
//     emergency: bool,
// }

// impl Tally {
//     pub fn new(
//         circumstance: Circumstance,
//         ending_block: u64,
//         withdrawn: Withdrawn<String>,
//         emergency: bool,
//     ) -> Self {
//         Self {
//             yes: 0,
//             no: 0,
//             abstain: 0,
//             circumstance,
//             ending_block,
//             withdrawn,
//             emergency,
//         }
//     }

//     pub fn add(&mut self, vote: Vote, power: u64) {
//         *match vote {
//             Vote::Yes => &mut self.yes,
//             Vote::No => &mut self.no,
//             Vote::Abstain => &mut self.abstain,
//         } += power;
//     }

//     pub fn total(&self) -> u64 {
//         self.yes + self.no + self.abstain
//     }

//     pub fn total_without_abstain(&self) -> u64 {
//         self.yes + self.no
//     }

//     pub fn evaluate(self, parameters: &Parameters) -> Option<Outcome<String>> {
//         // Are we before the end of normal voting?
//         let before_end = self.circumstance.current_block < self.ending_block;

//         // Check to see if proposal is an emergency proposal or if it's at the right height to
//         // render an outcome
//         if !self.emergency && before_end {
//             return None;
//         }

//         // Check to see if we've met quorum
//         if Ratio::new(self.total(), self.circumstance.total_voting_power) < parameters.valid_quorum
//         {
//             return Some(Outcome::Failed {
//                 withdrawn: self.withdrawn,
//             });
//         }

//         // Check to see if it has been vetoed
//         if Ratio::new(self.no, self.total()) > parameters.veto_threshold {
//             return Some(Outcome::Vetoed {
//                 withdrawn: self.withdrawn,
//             });
//         }

//         // Calculate the current yes/total-without-abstain ratio, which will be used to determine if
//         // the proposal has yet passed
//         let ratio_without_abstain = Ratio::new(self.yes, self.total_without_abstain().min(1));
//         // ^ in the above, the `.min(1)` is to prevent a divide-by-zero error when the only votes
//         // cast are abstains -- this results in a 0:1 ratio in that case, which will never pass, as
//         // desired in that situation

//         // Different logic is used to determine pass threshold depending on emergency/not
//         if self.emergency && before_end {
//             // A 2/3 supermajority is required to pass an emergency proposal before it ends normally
//             if Ratio::new(2, 3) < ratio_without_abstain {
//                 // We might yet reach 2/3 supermajority, but we're not there yet
//                 return None;
//             }
//         } else {
//             // Otherwise, the ratio is whatever is specified in the parameters
//             if parameters.pass_threshold < ratio_without_abstain {
//                 // The proposal has failed at this point because it's non-emergency, so we are not
//                 // evaluating it mid-proposal
//                 return Some(Outcome::Failed {
//                     withdrawn: self.withdrawn,
//                 });
//             }
//         };

//         // Ensure that we never pass a withdrawn proposal
//         if matches!(self.withdrawn, Withdrawn::WithReason { .. }) {
//             return Some(Outcome::Failed {
//                 withdrawn: self.withdrawn,
//             });
//         }

//         // If all the checked above didn't return early, only now can we pass the proposal
//         Some(Outcome::Passed)
//     }
// }

// impl Parameters {
//     pub async fn tally(
//         &self,
//         state: impl StateRead,
//         circumstance: Circumstance,
//         proposal_id: u64,
//     ) -> Result<Option<Outcome<String>>> {
//         // Determine the withdrawal state of the proposal
//         let withdrawn = state
//             .proposal_state(proposal_id)
//             .await?
//             .ok_or_else(|| anyhow::anyhow!("missing proposal state"))?
//             .withdrawn();

//         // Determine if the proposal is an emergency proposal
//         let emergency = state
//             .proposal_payload(proposal_id)
//             .await?
//             .ok_or_else(|| anyhow::anyhow!("missing proposal payload"))?
//             .is_emergency();

//         // Find out when the voting normally ends for this proposal
//         let ending_block = state
//             .proposal_voting_end(proposal_id)
//             .await?
//             .ok_or_else(|| anyhow::anyhow!("missing proposal end block"))?;

//         // Initialize a tally for this proposal
//         let mut tally = Tally::new(circumstance, ending_block, withdrawn, emergency);

//         for identity_key in state.voting_validators(proposal_id).await? {
//             let vote = state
//                 .validator_vote(proposal_id, identity_key)
//                 .await?
//                 .expect("validator voted");
//             let power = state
//                 .validator_power(&identity_key)
//                 .await?
//                 .expect("validator has a power");
//             tally.add(vote, power);
//         }

//         Ok(tally.evaluate(self))
//     }
// }
