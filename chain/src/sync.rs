use std::convert::TryFrom;

use anyhow::Result;
use penumbra_crypto::{note, NotePayload, Nullifier};
use penumbra_proto::{chain as pb, Protobuf};
use penumbra_tct::builder::{block, epoch};
use serde::{Deserialize, Serialize};

/// A compressed delta update with the minimal data from a block required to
/// synchronize private client state.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::CompactBlock", into = "pb::CompactBlock")]
pub struct CompactBlock {
    pub height: u64,
    // Note payloads describing new notes.
    pub note_payloads: Vec<NotePayload>,
    // Nullifiers identifying spent notes.
    pub nullifiers: Vec<Nullifier>,
    // The block root of this block.
    pub block_root: block::Root,
    // The epoch root of this epoch, if this block ends an epoch (`None` otherwise).
    pub epoch_root: Option<epoch::Root>,
    // Note payloads describing new quarantined notes.
    pub quarantined_note_payloads: Vec<NotePayload>,
    // Nullifiers identifying quarantined spent notes.
    pub quarantined_nullifiers: Vec<Nullifier>,
    // Note commitments identifying quarantined notes that have been rolled back.
    pub rolled_back_note_commitments: Vec<note::Commitment>,
    // Nullifiers identifying quarantined notes that have been rolled back.
    pub rolled_back_nullifiers: Vec<Nullifier>,
}

impl Default for CompactBlock {
    fn default() -> Self {
        Self {
            height: 0,
            note_payloads: Vec::new(),
            nullifiers: Vec::new(),
            block_root: block::Finalized::default().root(),
            epoch_root: None,
            quarantined_note_payloads: Vec::new(),
            quarantined_nullifiers: Vec::new(),
            rolled_back_note_commitments: Vec::new(),
            rolled_back_nullifiers: Vec::new(),
        }
    }
}

impl CompactBlock {
    /// Returns true if the compact block is empty.
    pub fn is_empty(&self) -> bool {
        self.note_payloads.is_empty() && self.nullifiers.is_empty()
    }
}

impl Protobuf<pb::CompactBlock> for CompactBlock {}

impl From<CompactBlock> for pb::CompactBlock {
    fn from(cb: CompactBlock) -> Self {
        pb::CompactBlock {
            height: cb.height,
            note_payloads: cb.note_payloads.into_iter().map(Into::into).collect(),
            nullifiers: cb.nullifiers.into_iter().map(Into::into).collect(),
            // We don't serialize block roots if they are the empty block, because we don't need to
            block_root: if cb.block_root.is_empty_finalized() {
                None
            } else {
                Some(cb.block_root.into())
            },
            epoch_root: cb.epoch_root.map(Into::into),
            quarantined_note_payloads: cb
                .quarantined_note_payloads
                .into_iter()
                .map(Into::into)
                .collect(),
            quarantined_nullifiers: cb
                .quarantined_nullifiers
                .into_iter()
                .map(Into::into)
                .collect(),
            rolled_back_note_commitments: cb
                .rolled_back_note_commitments
                .into_iter()
                .map(Into::into)
                .collect(),
            rolled_back_nullifiers: cb
                .rolled_back_nullifiers
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl TryFrom<pb::CompactBlock> for CompactBlock {
    type Error = anyhow::Error;

    fn try_from(value: pb::CompactBlock) -> Result<Self, Self::Error> {
        Ok(CompactBlock {
            height: value.height,
            note_payloads: value
                .note_payloads
                .into_iter()
                .map(NotePayload::try_from)
                .collect::<Result<Vec<NotePayload>>>()?,
            nullifiers: value
                .nullifiers
                .into_iter()
                .map(Nullifier::try_from)
                .collect::<Result<Vec<Nullifier>>>()?,
            block_root: value
                .block_root
                .map(TryInto::try_into)
                .transpose()?
                // If the block root wasn't present, that means it's the default finalized block root
                .unwrap_or_else(|| block::Finalized::default().root()),
            epoch_root: value.epoch_root.map(TryInto::try_into).transpose()?,
            quarantined_note_payloads: value
                .quarantined_note_payloads
                .into_iter()
                .map(NotePayload::try_from)
                .collect::<Result<Vec<NotePayload>>>()?,
            quarantined_nullifiers: value
                .quarantined_nullifiers
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<Nullifier>>>()?,
            rolled_back_note_commitments: value
                .rolled_back_note_commitments
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<note::Commitment>, _>>()?,
            rolled_back_nullifiers: value
                .rolled_back_nullifiers
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<Nullifier>>>()?,
        })
    }
}
