use rand_core::{CryptoRng, OsRng, RngCore};
use std::collections::{BTreeMap, HashSet};

use penumbra_crypto::{keys, memo::MemoPlaintext, merkle, note, Note, Nullifier};

const MAX_MERKLE_CHECKPOINTS_CLIENT: usize = 10;

/// State about the chain and our transactions.
pub struct ClientState {
    // The first block height we've scanned from.
    pub first_block_height: i64,
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
    // Transaction IDs we have visibility into.
    pub transactions: Vec<Vec<u8>>,
    // Key material.
    // TODO: Allow multiple spend authorities.
    // By making this Vec<Wallet>?
    pub wallet: Wallet,
}

/// Store all key material for a single spend authority.
pub struct Wallet {
    pub spend_key: keys::SpendKey,
    // Store derived keys here?
}

impl Wallet {
    /// Generate key material for a new spend authority.
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        Wallet {
            spend_key: keys::SpendKey::generate(rng),
        }

        // TODO: Persist to disk.
    }
}

impl Default for ClientState {
    fn default() -> Self {
        let mut rng = OsRng;

        Self {
            first_block_height: 0,
            last_block_height: 0,
            note_commitment_tree: merkle::BridgeTree::new(MAX_MERKLE_CHECKPOINTS_CLIENT),
            nullifier_map: BTreeMap::new(),
            received_set: HashSet::new(),
            spent_set: HashSet::new(),
            transactions: Vec::new(),
            wallet: Wallet::generate(&mut rng),
        }
    }
}

impl ClientState {
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
