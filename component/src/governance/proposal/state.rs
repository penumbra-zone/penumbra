use serde::{Deserialize, Serialize};

use penumbra_proto::{core::governance::v1alpha1 as pb, Protobuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ProposalState", into = "pb::ProposalState")]
pub enum State {
    Voting,
    Withdrawn { reason: String },
    Finished { outcome: Outcome },
}

impl State {
    pub fn withdrawn(self) -> Withdrawn {
        match self {
            State::Voting => Withdrawn::No,
            State::Withdrawn { reason } => Withdrawn::WithReason { reason },
            State::Finished { outcome } => match outcome {
                Outcome::Passed => Withdrawn::No,
                Outcome::Failed { withdrawn } | Outcome::Vetoed { withdrawn } => withdrawn,
            },
        }
    }
}

impl Protobuf<pb::ProposalState> for State {}

impl From<State> for pb::ProposalState {
    fn from(s: State) -> Self {
        let state = match s {
            State::Voting => pb::proposal_state::State::Voting(pb::proposal_state::Voting {}),
            State::Withdrawn { reason } => {
                pb::proposal_state::State::Withdrawn(pb::proposal_state::Withdrawn { reason })
            }
            State::Finished { outcome } => {
                pb::proposal_state::State::Finished(pb::proposal_state::Finished {
                    outcome: Some(outcome.into()),
                })
            }
        };
        pb::ProposalState { state: Some(state) }
    }
}

impl TryFrom<pb::ProposalState> for State {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ProposalState) -> Result<Self, Self::Error> {
        Ok(
            match msg
                .state
                .ok_or_else(|| anyhow::anyhow!("missing proposal state"))?
            {
                pb::proposal_state::State::Voting(pb::proposal_state::Voting {}) => State::Voting,
                pb::proposal_state::State::Withdrawn(pb::proposal_state::Withdrawn { reason }) => {
                    State::Withdrawn { reason }
                }
                pb::proposal_state::State::Finished(pb::proposal_state::Finished { outcome }) => {
                    State::Finished {
                        outcome: outcome
                            .ok_or_else(|| anyhow::anyhow!("missing proposal outcome"))?
                            .try_into()?,
                    }
                }
            },
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ProposalOutcome", into = "pb::ProposalOutcome")]
pub enum Outcome {
    Passed,
    Failed { withdrawn: Withdrawn },
    Vetoed { withdrawn: Withdrawn },
}

impl Outcome {
    /// Determines if the outcome should be refunded (i.e. it was not vetoed).
    pub fn should_be_refunded(&self) -> bool {
        !self.is_vetoed()
    }

    pub fn is_vetoed(&self) -> bool {
        matches!(self, Outcome::Vetoed { .. })
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Outcome::Failed { .. } | Outcome::Vetoed { .. })
    }

    pub fn is_passed(&self) -> bool {
        matches!(self, Outcome::Passed)
    }
}

#[derive(Debug, Clone)]
pub enum Withdrawn {
    No,
    WithReason { reason: String },
}

impl From<Option<String>> for Withdrawn {
    fn from(reason: Option<String>) -> Self {
        match reason {
            Some(reason) => Withdrawn::WithReason { reason },
            None => Withdrawn::No,
        }
    }
}

impl From<Withdrawn> for Option<String> {
    fn from(withdrawn: Withdrawn) -> Self {
        match withdrawn {
            Withdrawn::No => None,
            Withdrawn::WithReason { reason } => Some(reason),
        }
    }
}

impl Protobuf<pb::ProposalOutcome> for Outcome {}

impl From<Outcome> for pb::ProposalOutcome {
    fn from(o: Outcome) -> Self {
        let outcome = match o {
            Outcome::Passed => {
                pb::proposal_outcome::Outcome::Passed(pb::proposal_outcome::Passed {})
            }
            Outcome::Failed { withdrawn } => {
                pb::proposal_outcome::Outcome::Failed(pb::proposal_outcome::Failed {
                    withdrawn_with_reason: withdrawn.into(),
                })
            }
            Outcome::Vetoed { withdrawn } => {
                pb::proposal_outcome::Outcome::Vetoed(pb::proposal_outcome::Vetoed {
                    withdrawn_with_reason: withdrawn.into(),
                })
            }
        };
        pb::ProposalOutcome {
            outcome: Some(outcome),
        }
    }
}

impl TryFrom<pb::ProposalOutcome> for Outcome {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ProposalOutcome) -> Result<Self, Self::Error> {
        Ok(
            match msg
                .outcome
                .ok_or_else(|| anyhow::anyhow!("missing proposal outcome"))?
            {
                pb::proposal_outcome::Outcome::Passed(pb::proposal_outcome::Passed {}) => {
                    Outcome::Passed
                }
                pb::proposal_outcome::Outcome::Failed(pb::proposal_outcome::Failed {
                    withdrawn_with_reason,
                }) => Outcome::Failed {
                    withdrawn: withdrawn_with_reason.into(),
                },
                pb::proposal_outcome::Outcome::Vetoed(pb::proposal_outcome::Vetoed {
                    withdrawn_with_reason,
                }) => Outcome::Vetoed {
                    withdrawn: withdrawn_with_reason.into(),
                },
            },
        )
    }
}
