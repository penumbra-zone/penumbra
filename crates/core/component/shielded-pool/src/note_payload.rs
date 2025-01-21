use anyhow::{Context, Error};

use penumbra_sdk_keys::keys::FullViewingKey;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{penumbra::core::component::shielded_pool::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::{note, Note, NoteCiphertext};
use decaf377_ka as ka;

#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::NotePayload", into = "pb::NotePayload")]
pub struct NotePayload {
    pub note_commitment: note::StateCommitment,
    pub ephemeral_key: ka::Public,
    pub encrypted_note: NoteCiphertext,
}

impl NotePayload {
    pub fn trial_decrypt(&self, fvk: &FullViewingKey) -> Option<Note> {
        // Try to decrypt the encrypted note using the ephemeral key and persistent incoming
        // viewing key -- if it doesn't decrypt, it wasn't meant for us.
        let note = Note::decrypt(&self.encrypted_note, fvk.incoming(), &self.ephemeral_key).ok()?;
        tracing::debug!(note_commitment = ?note.commit(), ?note, "found note while scanning");

        // Verification logic (if any fails, return None & log error)
        // Reject notes with zero amount
        if note.amount() == Amount::zero() {
            // This is only debug-level because it can happen honestly (e.g., swap claims, dummy spends).
            tracing::debug!("ignoring note recording zero assets");
            return None;
        }
        // Make sure spendable by keys
        if !note.controlled_by(fvk) {
            // This should be a warning, because no honestly generated note plaintext should
            // mismatch the FVK that can detect and decrypt it.
            tracing::warn!("decrypted note that is not spendable by provided full viewing key");
            return None;
        }
        // Make sure note commitment matches
        if note.commit() != self.note_commitment {
            // This should be a warning, because no honestly generated note plaintext should
            // fail to match the note commitment actually included in the chain.
            tracing::warn!("decrypted note does not match provided note commitment");
            return None;
        }

        // NOTE: We intentionally return `Option` here instead of `Result`
        // such that we gracefully drop malformed notes instead of returning an error
        // that may propagate up the call stack and cause a panic.
        // All errors in parsing notes must not cause a panic in the view service.
        // A panic when parsing a specific note could link the fact that the malformed
        // note can be successfully decrypted with a specific IP.
        //
        // See "REJECT" attack (CVE-2019-16930) for a similar attack in ZCash
        // Section 4.1 in https://crypto.stanford.edu/timings/pingreject.pdf
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

impl DomainType for NotePayload {
    type Proto = pb::NotePayload;
}

impl From<NotePayload> for pb::NotePayload {
    fn from(msg: NotePayload) -> Self {
        pb::NotePayload {
            note_commitment: Some(msg.note_commitment.into()),
            ephemeral_key: msg.ephemeral_key.0.to_vec(),
            encrypted_note: Some(msg.encrypted_note.into()),
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
                .context("ephemeral key malformed")?,
            encrypted_note: proto
                .encrypted_note
                .ok_or_else(|| anyhow::anyhow!("missing encrypted note"))?
                .try_into()?,
        })
    }
}
