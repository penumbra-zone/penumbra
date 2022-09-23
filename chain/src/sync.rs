use std::convert::TryFrom;

use anyhow::Result;
use penumbra_crypto::{IdentityKey, NotePayload, Nullifier};
use penumbra_proto::{core::chain::v1alpha1 as pb, Protobuf};
use penumbra_tct::builder::{block, epoch};
use serde::{Deserialize, Serialize};

use crate::{params::FmdParameters, quarantined::Quarantined, NoteSource};

/// A note payload annotated with the source of the note.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    try_from = "pb::AnnotatedNotePayload",
    into = "pb::AnnotatedNotePayload"
)]
pub struct AnnotatedNotePayload {
    pub payload: NotePayload,
    pub source: NoteSource,
}

/// A compressed delta update with the minimal data from a block required to
/// synchronize private client state.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::CompactBlock", into = "pb::CompactBlock")]
pub struct CompactBlock {
    pub height: u64,
    // Annotated note payloads describing new notes.
    pub note_payloads: Vec<AnnotatedNotePayload>,
    // Nullifiers identifying spent notes.
    pub nullifiers: Vec<Nullifier>,
    // The block root of this block.
    pub block_root: block::Root,
    // The epoch root of this epoch, if this block ends an epoch (`None` otherwise).
    pub epoch_root: Option<epoch::Root>,
    // Newly quarantined things in this block.
    pub quarantined: Quarantined,
    // Newly slashed validators in this block.
    pub slashed: Vec<IdentityKey>,
    // Latest FMD parameters. `None` if unchanged.
    pub fmd_parameters: Option<FmdParameters>,
    // If the block indicated a proposal was being started.
    pub proposal_started: bool,
    // **IMPORTANT NOTE FOR FUTURE HUMANS**: if you want to add new fields to the `CompactBlock`,
    // you must update `CompactBlock::requires_scanning` to check for the emptiness of those fields,
    // because the client will skip processing any compact block that is marked as not requiring
    // scanning.
}

impl Default for CompactBlock {
    fn default() -> Self {
        Self {
            height: 0,
            note_payloads: Vec::new(),
            nullifiers: Vec::new(),
            block_root: block::Finalized::default().root(),
            epoch_root: None,
            quarantined: Quarantined::default(),
            slashed: Vec::new(),
            fmd_parameters: None,
            proposal_started: false,
        }
    }
}

impl CompactBlock {
    /// Returns true if the compact block is empty.
    pub fn requires_scanning(&self) -> bool {
        !self.note_payloads.is_empty() // need to scan notes
            || !self.nullifiers.is_empty() // need to collect nullifiers
            || !self.quarantined.is_empty() // need to scan quarantined notes
            || !self.slashed.is_empty() // need to process slashing
            || self.fmd_parameters.is_some() // need to save latest FMD parameters
            || self.proposal_started // need to process proposal start
    }
}

impl Protobuf<pb::AnnotatedNotePayload> for AnnotatedNotePayload {}

impl From<AnnotatedNotePayload> for pb::AnnotatedNotePayload {
    fn from(v: AnnotatedNotePayload) -> Self {
        pb::AnnotatedNotePayload {
            payload: Some(v.payload.into()),
            source: Some(v.source.into()),
        }
    }
}

impl TryFrom<pb::AnnotatedNotePayload> for AnnotatedNotePayload {
    type Error = anyhow::Error;

    fn try_from(value: pb::AnnotatedNotePayload) -> Result<Self, Self::Error> {
        Ok(AnnotatedNotePayload {
            payload: value
                .payload
                .ok_or_else(|| anyhow::anyhow!("missing note payload"))?
                .try_into()?,
            source: value
                .source
                .ok_or_else(|| anyhow::anyhow!("missing note source"))?
                .try_into()?,
        })
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
            quarantined: if cb.quarantined.is_empty() {
                None
            } else {
                Some(cb.quarantined.into())
            },
            slashed: cb.slashed.into_iter().map(Into::into).collect(),
            fmd_parameters: cb.fmd_parameters.map(Into::into),
            proposal_started: cb.proposal_started,
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
                .map(AnnotatedNotePayload::try_from)
                .collect::<Result<Vec<AnnotatedNotePayload>>>()?,
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
            quarantined: value
                .quarantined
                .map(TryInto::try_into)
                .transpose()?
                // If the quarantined set wasn't present, that means it contained nothing, so make
                // it the default, empty set
                .unwrap_or_default(),
            slashed: value
                .slashed
                .into_iter()
                .map(IdentityKey::try_from)
                .collect::<Result<Vec<_>>>()?,
            fmd_parameters: value.fmd_parameters.map(TryInto::try_into).transpose()?,
            proposal_started: value.proposal_started,
        })
    }
}
