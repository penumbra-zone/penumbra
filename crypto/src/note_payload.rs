use anyhow::Error;
use bytes::Bytes;
use penumbra_proto::{crypto as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::{ka, note};

#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::NotePayload", into = "pb::NotePayload")]
pub struct NotePayload {
    pub note_commitment: note::Commitment,
    pub ephemeral_key: ka::Public,
    pub encrypted_note: [u8; note::NOTE_CIPHERTEXT_BYTES],
}

impl std::fmt::Debug for NotePayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NotePayload")
            .field("note_commitment", &self.note_commitment)
            .field("ephemeral_key", &self.ephemeral_key)
            .field("encrypted_note", &"...")
            .finish()
    }
}

impl Protobuf<pb::NotePayload> for NotePayload {}

impl From<NotePayload> for pb::NotePayload {
    fn from(msg: NotePayload) -> Self {
        pb::NotePayload {
            note_commitment: Some(msg.note_commitment.into()),
            ephemeral_key: Bytes::copy_from_slice(&msg.ephemeral_key.0),
            encrypted_note: Bytes::copy_from_slice(&msg.encrypted_note),
        }
    }
}

impl TryFrom<pb::NotePayload> for NotePayload {
    type Error = Error;

    fn try_from(proto: pb::NotePayload) -> anyhow::Result<Self, Self::Error> {
        Ok(NotePayload {
            note_commitment: proto
                .note_commitment
                .ok_or_else(|| anyhow::anyhow!("missing note commitment"))?
                .try_into()?,
            ephemeral_key: ka::Public::try_from(&proto.ephemeral_key[..])
                .map_err(|_| anyhow::anyhow!("output body malformed"))?,
            encrypted_note: proto.encrypted_note[..]
                .try_into()
                .map_err(|_| anyhow::anyhow!("output body malformed"))?,
        })
    }
}
