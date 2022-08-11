use serde::{Deserialize, Serialize};

use penumbra_proto::{governance as pb, Protobuf};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ProposalState", into = "pb::ProposalState")]
pub enum State {
    Proposed,
    Voting,
    Withdrawn,
    Finished { outcome: Outcome },
}

impl Protobuf<pb::ProposalState> for State {}

impl From<State> for pb::ProposalState {
    fn from(s: State) -> Self {
        let state = match s {
            State::Proposed => pb::proposal_state::State::Proposed(pb::proposal_state::Proposed {}),
            State::Voting => pb::proposal_state::State::Voting(pb::proposal_state::Voting {}),
            State::Withdrawn => {
                pb::proposal_state::State::Withdrawn(pb::proposal_state::Withdrawn {})
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
                pb::proposal_state::State::Proposed(pb::proposal_state::Proposed {}) => {
                    State::Proposed
                }
                pb::proposal_state::State::Voting(pb::proposal_state::Voting {}) => State::Voting,
                pb::proposal_state::State::Withdrawn(pb::proposal_state::Withdrawn {}) => {
                    State::Withdrawn
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ProposalOutcome", into = "pb::ProposalOutcome")]
pub enum Outcome {
    Passed,
    Failed { withdrawn: bool },
    Vetoed { withdrawn: bool },
    WithdrawnBeforeVote,
}

impl Protobuf<pb::ProposalOutcome> for Outcome {}

impl From<Outcome> for pb::ProposalOutcome {
    fn from(o: Outcome) -> Self {
        let outcome = match o {
            Outcome::Passed => {
                pb::proposal_outcome::Outcome::Passed(pb::proposal_outcome::Passed {})
            }
            Outcome::Failed { withdrawn } => {
                pb::proposal_outcome::Outcome::Failed(pb::proposal_outcome::Failed { withdrawn })
            }
            Outcome::Vetoed { withdrawn } => {
                pb::proposal_outcome::Outcome::Vetoed(pb::proposal_outcome::Vetoed { withdrawn })
            }
            Outcome::WithdrawnBeforeVote => pb::proposal_outcome::Outcome::WithdrawnBeforeVote(
                pb::proposal_outcome::WithdrawnBeforeVote {},
            ),
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
                    withdrawn,
                }) => Outcome::Failed { withdrawn },
                pb::proposal_outcome::Outcome::Vetoed(pb::proposal_outcome::Vetoed {
                    withdrawn,
                }) => Outcome::Vetoed { withdrawn },
                pb::proposal_outcome::Outcome::WithdrawnBeforeVote(
                    pb::proposal_outcome::WithdrawnBeforeVote {},
                ) => Outcome::WithdrawnBeforeVote,
            },
        )
    }
}
