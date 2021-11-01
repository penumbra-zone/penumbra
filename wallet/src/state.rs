use std::collections::{BTreeMap, HashSet};

use penumbra_crypto::{keys, memo::MemoPlaintext, merkle, note, Note, Nullifier};

const MAX_MERKLE_CHECKPOINTS_CLIENT: usize = 10;

/// State about the chain and our transactions.
pub struct ClientState {
    // The last block height we've scanned to.
    pub last_block_height: i64,
    // Note commitment tree.
    pub note_commitment_tree: merkle::BridgeTree<note::Commitment, { merkle::DEPTH as u8 }>,
    // Our nullifiers and the notes they correspond to.
    pub nullifier_map: BTreeMap<Nullifier, Note>,
    // Notes that we have received.
    pub received_set: HashSet<(Note, MemoPlaintext)>,
    // Notes that we have spent.
    pub spent_set: HashSet<Note>,
    // Map of transaction ID to full transaction data for transactions we have visibility into.
    pub transactions: BTreeMap<[u8; 32], Vec<u8>>,
    // Key material.
    pub spend_key: keys::SpendKey,
}

impl ClientState {
    pub fn new(spend_key: keys::SpendKey) -> Self {
        Self {
            last_block_height: 0,
            note_commitment_tree: merkle::BridgeTree::new(MAX_MERKLE_CHECKPOINTS_CLIENT),
            nullifier_map: BTreeMap::new(),
            received_set: HashSet::new(),
            spent_set: HashSet::new(),
            transactions: BTreeMap::new(),
            spend_key,
        }
    }

    // TODO: For each output in scanned transactions, try to decrypt the note ciphertext.
    // If the note decrypts, we:
    // * add the (note plaintext, memo) to the received_set.
    // * compute and add the nullifier to the nullifier map.
    // * add the note commitment to the note commitment tree.
    // * witness the note commitment value.
    //
    // For each spend, if the revealed nf is in our nullifier_map, then
    // we add nullifier_map[nf] to spent_set.
}
