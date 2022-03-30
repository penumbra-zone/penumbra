use bytes::Bytes;
use std::convert::TryFrom;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use penumbra_crypto::{ka, note, FieldExt, Nullifier};
use penumbra_proto::{chain as pb, Protobuf};

// Domain type for CompactOutput.
// The minimum data needed to identify a new note.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::CompactOutput", into = "pb::CompactOutput")]
pub struct CompactOutput {
    // The note commitment for the output note. 32 bytes.
    pub note_commitment: note::Commitment,
    // The encoding of an ephemeral public key. 32 bytes.
    pub ephemeral_key: ka::Public,
    // An encryption of the newly created note.
    // 132 = 1(type) + 11(d) + 8(amount) + 32(asset_id) + 32(rcm) + 32(pk_d) + 16(MAC) bytes.
    pub encrypted_note: Vec<u8>,
}

impl Protobuf<pb::CompactOutput> for CompactOutput {}

impl From<CompactOutput> for pb::CompactOutput {
    fn from(co: CompactOutput) -> Self {
        pb::CompactOutput {
            note_commitment: Bytes::copy_from_slice(&co.note_commitment.0.to_bytes()),
            ephemeral_key: Bytes::copy_from_slice(&co.ephemeral_key.0),
            encrypted_note: co.encrypted_note.into(),
        }
    }
}

impl TryFrom<pb::CompactOutput> for CompactOutput {
    type Error = anyhow::Error;

    fn try_from(co: pb::CompactOutput) -> Result<Self, Self::Error> {
        Ok(CompactOutput {
            note_commitment: note::Commitment::try_from(&*co.note_commitment)?,
            ephemeral_key: ka::Public::try_from(&*co.ephemeral_key)?,
            encrypted_note: co.encrypted_note.to_vec(),
        })
    }
}

// Domain type for CompactBlock.
// Contains the minimum data needed to update client state.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(try_from = "pb::CompactBlock", into = "pb::CompactBlock")]
pub struct CompactBlock {
    pub height: u64,
    // Fragments of new notes.
    pub fragments: Vec<CompactOutput>,
    // Nullifiers identifying spent notes.
    pub nullifiers: Vec<Nullifier>,
}

impl Protobuf<pb::CompactBlock> for CompactBlock {}

impl From<CompactBlock> for pb::CompactBlock {
    fn from(cb: CompactBlock) -> Self {
        pb::CompactBlock {
            height: cb.height,
            fragments: cb.fragments.into_iter().map(Into::into).collect(),
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
            fragments: value
                .fragments
                .into_iter()
                .map(CompactOutput::try_from)
                .collect::<Result<Vec<CompactOutput>>>()?,
            nullifiers: value
                .nullifiers
                .into_iter()
                .map(|v| Nullifier::try_from(&*v))
                .collect::<Result<Vec<Nullifier>>>()?,
        })
    }
}
