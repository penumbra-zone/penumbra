use std::collections::{BTreeMap, HashSet};

use penumbra_crypto::{fmd, memo::MemoPlaintext, merkle, note, Address, Note, Nullifier};

use crate::storage::Wallet;

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
    pub wallet: Wallet,
}

impl ClientState {
    pub fn new(wallet: Wallet) -> Self {
        Self {
            last_block_height: 0,
            note_commitment_tree: merkle::BridgeTree::new(MAX_MERKLE_CHECKPOINTS_CLIENT),
            nullifier_map: BTreeMap::new(),
            received_set: HashSet::new(),
            spent_set: HashSet::new(),
            transactions: BTreeMap::new(),
            wallet,
        }
    }

    /// Generate a new diversified `Address` and its corresponding `DetectionKey`.
    pub fn new_address(&mut self) -> (Address, fmd::DetectionKey) {
        self.wallet.new_address()
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
