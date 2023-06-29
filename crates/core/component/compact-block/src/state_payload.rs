use std::convert::TryFrom;

use anyhow::Result;
use penumbra_crypto::{note, NotePayload};
use penumbra_dex::swap::SwapPayload;
use penumbra_proto::core::chain::v1alpha1::{self as pb};

use serde::{Deserialize, Serialize};

use penumbra_chain::NoteSource;

/// A note payload annotated with the source of the note.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::StatePayload", into = "pb::StatePayload")]
pub enum StatePayload {
    RolledUp(note::StateCommitment),
    Note {
        source: NoteSource,
        note: Box<NotePayload>,
    },
    Swap {
        source: NoteSource,
        swap: Box<SwapPayload>,
    },
}

pub struct StatePayloadDebugKind<'a>(pub &'a StatePayload);

impl<'a> std::fmt::Debug for StatePayloadDebugKind<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            StatePayload::RolledUp(_) => f.debug_struct("RolledUp").finish_non_exhaustive(),
            StatePayload::Note { .. } => f.debug_struct("Note").finish_non_exhaustive(),
            StatePayload::Swap { .. } => f.debug_struct("Swap").finish_non_exhaustive(),
        }
    }
}

impl StatePayload {
    pub fn commitment(&self) -> &note::StateCommitment {
        match self {
            Self::RolledUp(commitment) => commitment,
            Self::Note { note, .. } => &note.note_commitment,
            Self::Swap { swap, .. } => &swap.commitment,
        }
    }

    pub fn source(&self) -> Option<&NoteSource> {
        match self {
            Self::RolledUp(_) => None,
            Self::Note { source, .. } => Some(source),
            Self::Swap { source, .. } => Some(source),
        }
    }
}

impl From<note::StateCommitment> for StatePayload {
    fn from(commitment: note::StateCommitment) -> Self {
        Self::RolledUp(commitment)
    }
}

impl From<(NotePayload, NoteSource)> for StatePayload {
    fn from((note, source): (NotePayload, NoteSource)) -> Self {
        Self::Note {
            note: Box::new(note),
            source,
        }
    }
}

impl From<(SwapPayload, NoteSource)> for StatePayload {
    fn from((swap, source): (SwapPayload, NoteSource)) -> Self {
        Self::Swap {
            swap: Box::new(swap),
            source,
        }
    }
}

impl From<StatePayload> for pb::StatePayload {
    fn from(msg: StatePayload) -> Self {
        match msg {
            StatePayload::RolledUp(commitment) => pb::StatePayload {
                state_payload: Some(pb::state_payload::StatePayload::RolledUp(
                    pb::state_payload::RolledUp {
                        commitment: Some(commitment.into()),
                    },
                )),
            },
            StatePayload::Note { source, note } => pb::StatePayload {
                state_payload: Some(pb::state_payload::StatePayload::Note(
                    pb::state_payload::Note {
                        source: Some(source.into()),
                        note: Some((*note).into()),
                    },
                )),
            },
            StatePayload::Swap { source, swap } => pb::StatePayload {
                state_payload: Some(pb::state_payload::StatePayload::Swap(
                    pb::state_payload::Swap {
                        source: Some(source.into()),
                        swap: Some((*swap).into()),
                    },
                )),
            },
        }
    }
}

impl TryFrom<pb::StatePayload> for StatePayload {
    type Error = anyhow::Error;
    fn try_from(value: pb::StatePayload) -> Result<Self, Self::Error> {
        match value.state_payload {
            Some(pb::state_payload::StatePayload::RolledUp(pb::state_payload::RolledUp {
                commitment,
            })) => Ok(StatePayload::RolledUp(
                commitment
                    .ok_or_else(|| anyhow::anyhow!("missing commitment"))?
                    .try_into()?,
            )),
            Some(pb::state_payload::StatePayload::Note(pb::state_payload::Note {
                source,
                note,
            })) => Ok(StatePayload::Note {
                note: Box::new(
                    note.ok_or_else(|| anyhow::anyhow!("missing note"))?
                        .try_into()?,
                ),
                source: source
                    .ok_or_else(|| anyhow::anyhow!("missing source"))?
                    .try_into()?,
            }),
            Some(pb::state_payload::StatePayload::Swap(pb::state_payload::Swap {
                source,
                swap,
            })) => Ok(StatePayload::Swap {
                swap: Box::new(
                    swap.ok_or_else(|| anyhow::anyhow!("missing swap"))?
                        .try_into()?,
                ),
                source: source
                    .ok_or_else(|| anyhow::anyhow!("missing source"))?
                    .try_into()?,
            }),
            None => Err(anyhow::anyhow!("missing state payload")),
        }
    }
}
