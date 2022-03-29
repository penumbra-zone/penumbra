use serde::{Deserialize, Serialize};

use penumbra_crypto::{ka, note, nullifier};
use penumbra_proto::chain as pb;

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

// Domain type for CompactBlock.
// Contains the minimum data needed to update client state.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pb::CompactBlock", into = "pb::CompactBlock")]
pub struct CompactBlock {
    pub height: u64,
    // Fragments of new notes.
    pub fragments: Vec<CompactOutput>,
    // Nullifiers identifying spent notes.
    pub nullifiers: Vec<nullifier::Nullifier>,
}
