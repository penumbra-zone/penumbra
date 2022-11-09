use anyhow::Result;
use num_rational::Ratio;
use penumbra_chain::params::ChainParameters;
use penumbra_transaction::action::Vote;

use super::{proposal::Withdrawn, StateReadExt as _};
use crate::stake::StateReadExt as _;
use penumbra_chain::StateReadExt as _;
use penumbra_storage::StateRead;

use super::proposal::Outcome;

/// The parameters used for tallying.
#[derive(Debug, Clone)]
pub struct Parameters {
    pub valid_quorum: Ratio<u64>,
    pub pass_threshold: Ratio<u64>,
    pub veto_threshold: Ratio<u64>,
}

impl From<&ChainParameters> for Parameters {
    fn from(params: &ChainParameters) -> Self {
        Parameters {
            valid_quorum: params.proposal_valid_quorum,
            pass_threshold: params.proposal_pass_threshold,
            veto_threshold: params.proposal_veto_threshold,
        }
    }
}

impl Parameters {
    pub async fn new(state: impl StateRead) -> Result<Self> {
        Ok(Parameters::from(&state.get_chain_params().await?))
    }
}

/// Constants during all tallies for a given block.
#[derive(Debug, Clone, Copy)]
pub struct Circumstance {
    pub total_voting_power: u64,
    pub current_block: u64,
}

impl Circumstance {
    pub async fn new(state: impl StateRead) -> Result<Self> {
        Ok(Circumstance {
            total_voting_power: state.total_voting_power().await?,
            current_block: state.get_block_height().await?,
        })
    }
}

/// Intermediate structure used for tallying votes for a particular proposal.
#[derive(Debug, Clone)]
pub struct Tally {
    yes: u64,
    no: u64,
    no_with_veto: u64,
    abstain: u64,
    // Constant during tallying:
    circumstance: Circumstance,
    ending_block: u64,
    withdrawn: Withdrawn,
    emergency: bool,
}

impl Tally {
    pub fn new(
        circumstance: Circumstance,
        ending_block: u64,
        withdrawn: Withdrawn,
        emergency: bool,
    ) -> Self {
        Self {
            yes: 0,
            no: 0,
            no_with_veto: 0,
            abstain: 0,
            circumstance,
            ending_block,
            withdrawn,
            emergency,
        }
    }

    pub fn add(&mut self, vote: Vote, power: u64) {
        *match vote {
            Vote::Yes => &mut self.yes,
            Vote::No => &mut self.no,
            Vote::NoWithVeto => &mut self.no_with_veto,
            Vote::Abstain => &mut self.abstain,
        } += power;
    }

    pub fn total(&self) -> u64 {
        self.yes + self.no + self.no_with_veto + self.abstain
    }

    pub fn total_without_abstain(&self) -> u64 {
        self.yes + self.no + self.no_with_veto
    }

    pub fn evaluate(self, parameters: &Parameters) -> Option<Outcome> {
        // Are we before the end of normal voting?
        let before_end = self.circumstance.current_block < self.ending_block;

        // Check to see if proposal is an emergency proposal or if it's at the right height to
        // render an outcome
        if !self.emergency && before_end {
            return None;
        }

        // Check to see if we've met quorum
        if Ratio::new(self.total(), self.circumstance.total_voting_power) < parameters.valid_quorum
        {
            return Some(Outcome::Failed {
                withdrawn: self.withdrawn,
            });
        }

        // Check to see if it has been vetoed
        if Ratio::new(self.no_with_veto, self.total()) > parameters.veto_threshold {
            return Some(Outcome::Vetoed {
                withdrawn: self.withdrawn,
            });
        }

        // Calculate the current yes/total-without-abstain ratio, which will be used to determine if
        // the proposal has yet passed
        let ratio_without_abstain = Ratio::new(self.yes, self.total_without_abstain().min(1));
        // ^ in the above, the `.min(1)` is to prevent a divide-by-zero error when the only votes
        // cast are abstains -- this results in a 0:1 ratio in that case, which will never pass, as
        // desired in that situation

        // Different logic is used to determine pass threshold depending on emergency/not
        if self.emergency && before_end {
            // A 2/3 supermajority is required to pass an emergency proposal before it ends normally
            if Ratio::new(2, 3) < ratio_without_abstain {
                // We might yet reach 2/3 supermajority, but we're not there yet
                return None;
            }
        } else {
            // Otherwise, the ratio is whatever is specified in the parameters
            if parameters.pass_threshold < ratio_without_abstain {
                // The proposal has failed at this point because it's non-emergency, so we are not
                // evaluating it mid-proposal
                return Some(Outcome::Failed {
                    withdrawn: self.withdrawn,
                });
            }
        };

        // Ensure that we never pass a withdrawn proposal
        if matches!(self.withdrawn, Withdrawn::WithReason { .. }) {
            return Some(Outcome::Failed {
                withdrawn: self.withdrawn,
            });
        }

        // If all the checked above didn't return early, only now can we pass the proposal
        Some(Outcome::Passed)
    }
}

impl Parameters {
    pub async fn tally(
        &self,
        state: impl StateRead,
        circumstance: Circumstance,
        proposal_id: u64,
    ) -> Result<Option<Outcome>> {
        // Determine the withdrawal state of the proposal
        let withdrawn = state
            .proposal_state(proposal_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing proposal state"))?
            .withdrawn();

        // Determine if the proposal is an emergency proposal
        let emergency = state
            .proposal_payload(proposal_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing proposal payload"))?
            .is_emergency();

        // Find out when the voting normally ends for this proposal
        let ending_block = state
            .proposal_voting_end(proposal_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing proposal end block"))?;

        // Initialize a tally for this proposal
        let mut tally = Tally::new(circumstance, ending_block, withdrawn, emergency);

        for identity_key in state.voting_validators(proposal_id).await? {
            let vote = state
                .validator_vote(proposal_id, identity_key)
                .await?
                .expect("validator voted");
            let power = state
                .validator_power(&identity_key)
                .await?
                .expect("validator has a power");
            tally.add(vote, power);
        }

        Ok(tally.evaluate(self))
    }
}
