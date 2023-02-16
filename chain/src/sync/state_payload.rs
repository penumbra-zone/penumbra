use std::convert::TryFrom;

use anyhow::Result;
use penumbra_crypto::{
    dex::{lp::LpNft, swap::SwapPayload},
    note, NotePayload,
};
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
        note: NotePayload,
    },
    Swap {
        source: NoteSource,
        swap: SwapPayload,
    },
    Position {
        lpnft: LpNft,
        commitment: note::Commitment,
    },
}

impl StatePayload {
    pub fn commitment(&self) -> &note::Commitment {
        match self {
            Self::RolledUp(commitment) => commitment,
            Self::Note { note, .. } => &note.note_commitment,
            Self::Swap { swap, .. } => &swap.commitment,
            Self::Position { commitment, .. } => commitment,
        }
    }

    pub fn source(&self) -> Option<&NoteSource> {
        match self {
            Self::RolledUp(_) => None,
            Self::Note { source, .. } => Some(source),
            Self::Swap { source, .. } => Some(source),
            Self::Position { .. } => None,
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
            StatePayload::Position { lpnft, commitment } => pb::StatePayload {
                state_payload: Some(pb::state_payload::StatePayload::Position(
                    pb::state_payload::Position {
                        lp_nft: Some(lpnft.into()),
                        commitment: Some(commitment.into()),
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
            Some(pb::state_payload::StatePayload::Position(pb::state_payload::Position {
                lp_nft,
                commitment,
            })) => Ok(StatePayload::Position {
                lpnft: lp_nft
                    .ok_or_else(|| anyhow::anyhow!("missing LP NFT"))?
                    .try_into()?,
                commitment: commitment
                    .ok_or_else(|| anyhow::anyhow!("missing commitment"))?
                    .try_into()?,
            }),
            None => Err(anyhow::anyhow!("missing state payload")),
        }
    }
}
