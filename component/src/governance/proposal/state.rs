use serde::{Deserialize, Serialize};

use penumbra_proto::{core::governance::v1alpha1 as pb, Protobuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::ProposalState", into = "pb::ProposalState")]
pub enum State {
    Voting,
    Withdrawn { reason: String },
    Finished { outcome: Outcome },
    Claimed { outcome: Outcome },
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
            State::Claimed { outcome } => match outcome {
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
            State::Claimed { outcome } => {
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
                pb::proposal_state::State::Claimed(pb::proposal_state::Claimed { outcome }) => {
                    State::Claimed {
                        outcome: outcome
                            .ok_or_else(|| anyhow::anyhow!("missing proposal outcome"))?
                            .try_into()?,
                    }
                }
            },
        )
    }
}
