use std::convert::TryFrom;

use anyhow::{Context, Result};
use penumbra_sdk_dex::swap::SwapPayload;
use penumbra_sdk_proto::penumbra::core::component::compact_block::v1::{self as pb};
use penumbra_sdk_shielded_pool::{note, NotePayload};

use serde::{Deserialize, Serialize};

use penumbra_sdk_sct::CommitmentSource;

/// A note payload annotated with the source of the note.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::StatePayload", into = "pb::StatePayload")]
pub enum StatePayload {
    RolledUp {
        source: CommitmentSource,
        commitment: note::StateCommitment,
    },
    Note {
        source: CommitmentSource,
        note: Box<NotePayload>,
    },
    Swap {
        source: CommitmentSource,
        swap: Box<SwapPayload>,
    },
}

pub struct StatePayloadDebugKind<'a>(pub &'a StatePayload);

impl<'a> std::fmt::Debug for StatePayloadDebugKind<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            StatePayload::RolledUp { .. } => f.debug_struct("RolledUp").finish_non_exhaustive(),
            StatePayload::Note { .. } => f.debug_struct("Note").finish_non_exhaustive(),
            StatePayload::Swap { .. } => f.debug_struct("Swap").finish_non_exhaustive(),
        }
    }
}

impl StatePayload {
    pub fn commitment(&self) -> &note::StateCommitment {
        match self {
            Self::RolledUp { commitment, .. } => commitment,
            Self::Note { note, .. } => &note.note_commitment,
            Self::Swap { swap, .. } => &swap.commitment,
        }
    }

    pub fn source(&self) -> &CommitmentSource {
        match self {
            Self::RolledUp { source, .. } => source,
            Self::Note { source, .. } => source,
            Self::Swap { source, .. } => source,
        }
    }
}

impl From<note::StateCommitment> for StatePayload {
    fn from(commitment: note::StateCommitment) -> Self {
        Self::RolledUp {
            commitment,
            source: CommitmentSource::transaction(),
        }
    }
}

impl From<(NotePayload, CommitmentSource)> for StatePayload {
    fn from((note, source): (NotePayload, CommitmentSource)) -> Self {
        Self::Note {
            note: Box::new(note),
            source,
        }
    }
}

impl From<(SwapPayload, CommitmentSource)> for StatePayload {
    fn from((swap, source): (SwapPayload, CommitmentSource)) -> Self {
        Self::Swap {
            swap: Box::new(swap),
            source,
        }
    }
}

impl From<StatePayload> for pb::StatePayload {
    fn from(msg: StatePayload) -> Self {
        match msg {
            StatePayload::RolledUp { source, commitment } => pb::StatePayload {
                source: Some(source.into()),
                state_payload: Some(pb::state_payload::StatePayload::RolledUp(
                    pb::state_payload::RolledUp {
                        commitment: Some(commitment.into()),
                    },
                )),
            },
            StatePayload::Note { source, note } => pb::StatePayload {
                source: Some(source.into()),
                state_payload: Some(pb::state_payload::StatePayload::Note(
                    pb::state_payload::Note {
                        note: Some((*note).into()),
                    },
                )),
            },
            StatePayload::Swap { source, swap } => pb::StatePayload {
                source: Some(source.into()),
                state_payload: Some(pb::state_payload::StatePayload::Swap(
                    pb::state_payload::Swap {
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
        let source = value
            .source
            .ok_or_else(|| anyhow::anyhow!("state payload missing source"))?
            .try_into()
            .context("could not parse commitment source")?;
        match value.state_payload {
            Some(pb::state_payload::StatePayload::RolledUp(pb::state_payload::RolledUp {
                commitment,
            })) => Ok(StatePayload::RolledUp {
                source,
                commitment: commitment
                    .ok_or_else(|| anyhow::anyhow!("missing commitment"))?
                    .try_into()?,
            }),
            Some(pb::state_payload::StatePayload::Note(pb::state_payload::Note { note })) => {
                Ok(StatePayload::Note {
                    note: Box::new(
                        note.ok_or_else(|| anyhow::anyhow!("missing note"))?
                            .try_into()?,
                    ),
                    source,
                })
            }
            Some(pb::state_payload::StatePayload::Swap(pb::state_payload::Swap { swap })) => {
                Ok(StatePayload::Swap {
                    swap: Box::new(
                        swap.ok_or_else(|| anyhow::anyhow!("missing swap"))?
                            .try_into()?,
                    ),
                    source,
                })
            }
            None => Err(anyhow::anyhow!("missing state payload")),
        }
    }
}
