use serde::{Deserialize, Serialize};

use penumbra_sdk_proto::{penumbra::core::component::governance::v1 as pb, DomainType};

use crate::MAX_VALIDATOR_VOTE_REASON_LENGTH;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "pb::ProposalState", into = "pb::ProposalState")]
pub enum State {
    Voting,
    Withdrawn { reason: String },
    Finished { outcome: Outcome<String> },
    Claimed { outcome: Outcome<String> },
}

impl State {
    pub fn is_voting(&self) -> bool {
        matches!(self, State::Voting)
    }

    pub fn is_withdrawn(&self) -> bool {
        matches!(self, State::Withdrawn { .. })
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, State::Finished { .. })
    }

    pub fn is_claimed(&self) -> bool {
        matches!(self, State::Claimed { .. })
    }

    pub fn is_passed(&self) -> bool {
        match self {
            State::Finished { outcome } => outcome.is_passed(),
            State::Claimed { outcome } => outcome.is_passed(),
            _ => false,
        }
    }

    pub fn is_failed(&self) -> bool {
        match self {
            State::Finished { outcome } => outcome.is_failed(),
            State::Claimed { outcome } => outcome.is_failed(),
            _ => false,
        }
    }

    pub fn is_slashed(&self) -> bool {
        match self {
            State::Finished { outcome } => outcome.is_slashed(),
            State::Claimed { outcome } => outcome.is_slashed(),
            _ => false,
        }
    }
}

impl State {
    pub fn withdrawn(self) -> Withdrawn<String> {
        match self {
            State::Voting => Withdrawn::No,
            State::Withdrawn { reason } => Withdrawn::WithReason { reason },
            State::Finished { outcome } => match outcome {
                Outcome::Passed => Withdrawn::No,
                Outcome::Failed { withdrawn } | Outcome::Slashed { withdrawn } => withdrawn,
            },
            State::Claimed { outcome } => match outcome {
                Outcome::Passed => Withdrawn::No,
                Outcome::Failed { withdrawn } | Outcome::Slashed { withdrawn } => withdrawn,
            },
        }
    }
}

impl DomainType for State {
    type Proto = pb::ProposalState;
}

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

// This is parameterized by `W`, the withdrawal reason, so that we can use `()` where a reason
// doesn't need to be specified. When this is the case, the serialized format in protobufs uses an
// empty string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[serde(
    try_from = "pb::ProposalOutcome",
    into = "pb::ProposalOutcome",
    bound = "W: Clone, pb::ProposalOutcome: From<Outcome<W>>, Outcome<W>: TryFrom<pb::ProposalOutcome, Error = anyhow::Error>"
)]
pub enum Outcome<W> {
    Passed,
    Failed { withdrawn: Withdrawn<W> },
    Slashed { withdrawn: Withdrawn<W> },
}

impl<W> Outcome<W> {
    /// Determines if the outcome should be refunded (i.e. it was not slashed).
    pub fn should_be_refunded(&self) -> bool {
        !self.is_slashed()
    }

    pub fn is_slashed(&self) -> bool {
        matches!(self, Outcome::Slashed { .. })
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Outcome::Failed { .. } | Outcome::Slashed { .. })
    }

    pub fn is_passed(&self) -> bool {
        matches!(self, Outcome::Passed)
    }

    pub fn as_ref(&self) -> Outcome<&W> {
        match self {
            Outcome::Passed => Outcome::Passed,
            Outcome::Failed { withdrawn } => Outcome::Failed {
                withdrawn: withdrawn.as_ref(),
            },
            Outcome::Slashed { withdrawn } => Outcome::Slashed {
                withdrawn: withdrawn.as_ref(),
            },
        }
    }

    pub fn map<X>(self, f: impl FnOnce(W) -> X) -> Outcome<X> {
        match self {
            Outcome::Passed => Outcome::Passed,
            Outcome::Failed { withdrawn } => Outcome::Failed {
                withdrawn: Option::from(withdrawn).map(f).into(),
            },
            Outcome::Slashed { withdrawn } => Outcome::Slashed {
                withdrawn: Option::from(withdrawn).map(f).into(),
            },
        }
    }
}

// This is parameterized by `W`, the withdrawal reason, so that we can use `()` where a reason
// doesn't need to be specified. When this is the case, the serialized format in protobufs uses an
// empty string.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Withdrawn<W> {
    No,
    WithReason { reason: W },
}

impl<W> Withdrawn<W> {
    pub fn as_ref(&self) -> Withdrawn<&W> {
        match self {
            Withdrawn::No => Withdrawn::No,
            Withdrawn::WithReason { reason } => Withdrawn::WithReason { reason },
        }
    }
}

impl<W> From<Option<W>> for Withdrawn<W> {
    fn from(reason: Option<W>) -> Self {
        match reason {
            Some(reason) => Withdrawn::WithReason { reason },
            None => Withdrawn::No,
        }
    }
}

impl<W> From<Withdrawn<W>> for Option<W> {
    fn from(withdrawn: Withdrawn<W>) -> Self {
        match withdrawn {
            Withdrawn::No => None,
            Withdrawn::WithReason { reason } => Some(reason),
        }
    }
}

impl TryFrom<Withdrawn<String>> for Withdrawn<()> {
    type Error = anyhow::Error;

    fn try_from(withdrawn: Withdrawn<String>) -> Result<Self, Self::Error> {
        Ok(match withdrawn {
            Withdrawn::No => Withdrawn::No,
            Withdrawn::WithReason { reason } => {
                if reason.is_empty() {
                    Withdrawn::WithReason { reason: () }
                } else {
                    anyhow::bail!("withdrawn reason is not empty")
                }
            }
        })
    }
}

impl DomainType for Outcome<String> {
    type Proto = pb::ProposalOutcome;
}

impl From<Outcome<String>> for pb::ProposalOutcome {
    fn from(o: Outcome<String>) -> Self {
        let outcome = match o {
            Outcome::Passed => {
                pb::proposal_outcome::Outcome::Passed(pb::proposal_outcome::Passed {})
            }
            Outcome::Failed { withdrawn } => {
                pb::proposal_outcome::Outcome::Failed(pb::proposal_outcome::Failed {
                    withdrawn: match withdrawn {
                        Withdrawn::No => None,
                        Withdrawn::WithReason { reason } => {
                            Some(pb::proposal_outcome::Withdrawn { reason })
                        }
                    },
                })
            }
            Outcome::Slashed { withdrawn } => {
                pb::proposal_outcome::Outcome::Slashed(pb::proposal_outcome::Slashed {
                    withdrawn: match withdrawn {
                        Withdrawn::No => None,
                        Withdrawn::WithReason { reason } => {
                            Some(pb::proposal_outcome::Withdrawn { reason })
                        }
                    },
                })
            }
        };
        pb::ProposalOutcome {
            outcome: Some(outcome),
        }
    }
}

impl TryFrom<pb::ProposalOutcome> for Outcome<String> {
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
                }) => Outcome::Failed {
                    withdrawn: if let Some(pb::proposal_outcome::Withdrawn { reason }) = withdrawn {
                        // Max reason length is 1kb
                        if reason.len() > MAX_VALIDATOR_VOTE_REASON_LENGTH {
                            anyhow::bail!("withdrawn reason must be smaller than 1kb")
                        }

                        Withdrawn::WithReason { reason }
                    } else {
                        Withdrawn::No
                    },
                },
                pb::proposal_outcome::Outcome::Slashed(pb::proposal_outcome::Slashed {
                    withdrawn,
                }) => Outcome::Slashed {
                    withdrawn: if let Some(pb::proposal_outcome::Withdrawn { reason }) = withdrawn {
                        // Max reason length is 1kb
                        if reason.len() > MAX_VALIDATOR_VOTE_REASON_LENGTH {
                            anyhow::bail!("withdrawn reason must be smaller than 1kb")
                        }
                        Withdrawn::WithReason { reason }
                    } else {
                        Withdrawn::No
                    },
                },
            },
        )
    }
}

impl DomainType for Outcome<()> {
    type Proto = pb::ProposalOutcome;
}

impl From<Outcome<()>> for pb::ProposalOutcome {
    fn from(o: Outcome<()>) -> Self {
        let outcome = match o {
            Outcome::Passed => {
                pb::proposal_outcome::Outcome::Passed(pb::proposal_outcome::Passed {})
            }
            Outcome::Failed { withdrawn } => {
                pb::proposal_outcome::Outcome::Failed(pb::proposal_outcome::Failed {
                    withdrawn: <Option<()>>::from(withdrawn).map(|()| {
                        pb::proposal_outcome::Withdrawn {
                            reason: "".to_string(),
                        }
                    }),
                })
            }
            Outcome::Slashed { withdrawn } => {
                pb::proposal_outcome::Outcome::Slashed(pb::proposal_outcome::Slashed {
                    withdrawn: <Option<()>>::from(withdrawn).map(|()| {
                        pb::proposal_outcome::Withdrawn {
                            reason: "".to_string(),
                        }
                    }),
                })
            }
        };
        pb::ProposalOutcome {
            outcome: Some(outcome),
        }
    }
}

impl TryFrom<pb::ProposalOutcome> for Outcome<()> {
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
                }) => {
                    // Max reason length is 1kb
                    if withdrawn.is_some() {
                        let reason = &withdrawn.as_ref().expect("reason is some").reason;
                        if reason.len() > MAX_VALIDATOR_VOTE_REASON_LENGTH {
                            anyhow::bail!("withdrawn reason must be smaller than 1kb");
                        }
                    }
                    Outcome::Failed {
                        withdrawn: <Withdrawn<String>>::from(withdrawn.map(|w| w.reason))
                            .try_into()?,
                    }
                }
                pb::proposal_outcome::Outcome::Slashed(pb::proposal_outcome::Slashed {
                    withdrawn,
                }) => {
                    // Max reason length is 1kb
                    if withdrawn.is_some() {
                        let reason = &withdrawn.as_ref().expect("reason is some").reason;
                        if reason.len() > MAX_VALIDATOR_VOTE_REASON_LENGTH {
                            anyhow::bail!("withdrawn reason must be smaller than 1kb");
                        }
                    }

                    Outcome::Slashed {
                        withdrawn: <Withdrawn<String>>::from(withdrawn.map(|w| w.reason))
                            .try_into()?,
                    }
                }
            },
        )
    }
}
