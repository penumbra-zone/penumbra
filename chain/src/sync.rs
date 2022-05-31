use std::convert::TryFrom;

use anyhow::Result;
use bytes::Bytes;
use penumbra_crypto::{FieldExt, NotePayload, Nullifier};
use penumbra_proto::{chain as pb, Protobuf};
use penumbra_tct as tct;
use serde::{Deserialize, Serialize};

/// A compressed delta update with the minimal data from a block required to
/// synchronize private client state.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::CompactBlock", into = "pb::CompactBlock")]
pub struct CompactBlock {
    pub height: u64,
    // Output bodies describing new notes.
    pub note_payloads: Vec<NotePayload>,
    // Nullifiers identifying spent notes.
    pub nullifiers: Vec<Nullifier>,
    // The block root of this block.
    pub block_root: tct::builder::block::Root,
    // The epoch root of this epoch, if this block ends an epoch (`None` otherwise).
    pub epoch_root: Option<tct::builder::epoch::Root>,
}

impl Default for CompactBlock {
    fn default() -> Self {
        Self {
            height: 0,
            note_payloads: Vec::new(),
            nullifiers: Vec::new(),
            block_root: tct::builder::block::Finalized::default().root(),
            epoch_root: None,
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
            nullifiers: cb
                .nullifiers
                .into_iter()
                .map(|v| Bytes::copy_from_slice(&v.0.to_bytes()))
                .collect(),
            // We don't serialize block roots if they are the empty block, because we don't need to
            block_root: Some(cb.block_root)
                .filter(|root| root.is_empty_finalized())
                .map(Into::into),
            epoch_root: cb.epoch_root.map(Into::into),
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
                .map(|v| Nullifier::try_from(&*v))
                .collect::<Result<Vec<Nullifier>>>()?,
            block_root: value
                .block_root
                .map(TryInto::try_into)
                .transpose()?
                // If the block root wasn't present, that means it's the default finalized block root
                .unwrap_or_else(|| tct::builder::block::Finalized::default().root()),
            epoch_root: value.epoch_root.map(TryInto::try_into).transpose()?,
        })
    }
}
