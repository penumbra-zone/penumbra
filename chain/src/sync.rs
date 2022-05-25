use std::convert::TryFrom;

use anyhow::Result;
use bytes::Bytes;
use penumbra_crypto::{FieldExt, NotePayload, Nullifier};
use penumbra_proto::{chain as pb, Protobuf};
use serde::{Deserialize, Serialize};

/// A compressed delta update with the minimal data from a block required to
/// synchronize private client state.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(try_from = "pb::CompactBlock", into = "pb::CompactBlock")]
pub struct CompactBlock {
    pub height: u64,
    // Output bodies describing new notes.
    pub note_payloads: Vec<NotePayload>,
    // Nullifiers identifying spent notes.
    pub nullifiers: Vec<Nullifier>,
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
        })
    }
}
