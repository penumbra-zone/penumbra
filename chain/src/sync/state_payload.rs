use std::convert::TryFrom;

use anyhow::Result;
use penumbra_crypto::{dex::swap::SwapPayload, note, EncryptedNote};
use penumbra_proto::core::chain::v1alpha1::{self as pb};

use serde::{Deserialize, Serialize};

use crate::NoteSource;

/// A note payload annotated with the source of the note.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::StatePayload", into = "pb::StatePayload")]
pub enum StatePayload {
    RolledUp(note::Commitment),
    Note {
        source: NoteSource,
        note: EncryptedNote,
    },
    Swap {
        source: NoteSource,
        swap: SwapPayload,
    },
}

impl StatePayload {
    pub fn commitment(&self) -> &note::Commitment {
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
                        note: Some(note.into()),
                    },
                )),
            },
            StatePayload::Swap { source, swap } => pb::StatePayload {
                state_payload: Some(pb::state_payload::StatePayload::Swap(
                    pb::state_payload::Swap {
                        source: Some(source.into()),
                        swap: Some(swap.into()),
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
                note: note
                    .ok_or_else(|| anyhow::anyhow!("missing note"))?
                    .try_into()?,
                source: source
                    .ok_or_else(|| anyhow::anyhow!("missing source"))?
                    .try_into()?,
            }),
            Some(pb::state_payload::StatePayload::Swap(pb::state_payload::Swap {
                source,
                swap,
            })) => Ok(StatePayload::Swap {
                swap: swap
                    .ok_or_else(|| anyhow::anyhow!("missing swap"))?
                    .try_into()?,
                source: source
                    .ok_or_else(|| anyhow::anyhow!("missing source"))?
                    .try_into()?,
            }),
            None => Err(anyhow::anyhow!("missing state payload")),
        }
    }
}
