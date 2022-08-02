use anyhow::Error;
use bytes::Bytes;
use penumbra_proto::{crypto as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::{ka, note, FullViewingKey, Note};

#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::NotePayload", into = "pb::NotePayload")]
pub struct NotePayload {
    pub note_commitment: note::Commitment,
    pub ephemeral_key: ka::Public,
    pub encrypted_note: [u8; note::NOTE_CIPHERTEXT_BYTES],
}

impl NotePayload {
    pub fn trial_decrypt(&self, fvk: &FullViewingKey) -> Option<Note> {
        // Try to decrypt the encrypted note using the ephemeral key and persistent incoming
        // viewing key -- if it doesn't decrypt, it wasn't meant for us.
        let note = Note::decrypt(
            self.encrypted_note.as_ref(),
            fvk.incoming(),
            &self.ephemeral_key,
        )
        .ok()?;
        tracing::debug!(note_commitment = ?note.commit(), ?note, "found note while scanning");

        // Verification logic (if any fails, return None & log error)
        // Reject notes with zero amount
        if note.amount() == 0 {
            tracing::warn!("note contains zero assets");
            return None;
        }
        // Make sure spendable by keys
        if !fvk.controls(&note) {
            tracing::warn!("note is not spendable by provided full viewing key");
            return None;
        }
        // Make sure note commitment matches
        if note.commit() != self.note_commitment {
            tracing::warn!("decrypted note does not match provided note commitment");
            return None;
        }

        Some(note)
    }
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
