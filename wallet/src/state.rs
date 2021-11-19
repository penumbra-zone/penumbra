use anyhow::Context;
use penumbra_proto::wallet::{CompactBlock, StateFragment};
use rand_core::{CryptoRng, RngCore};
use std::collections::BTreeMap;

use penumbra_crypto::{
    fmd, merkle,
    merkle::{Frontier, Tree, TreeExt},
    note, Address, Note, Nullifier, Transaction, CURRENT_CHAIN_ID,
};

use crate::storage::Wallet;

const MAX_MERKLE_CHECKPOINTS_CLIENT: usize = 10;

/// State about the chain and our transactions.
#[derive(Clone, Debug)]
pub struct ClientState {
    /// The last block height we've scanned to.
    last_block_height: u32,
    /// Note commitment tree.
    note_commitment_tree: merkle::BridgeTree<note::Commitment, { merkle::DEPTH as u8 }>,
    /// Our nullifiers and the notes they correspond to.
    nullifier_map: BTreeMap<Nullifier, note::Commitment>,
    /// Notes that we have received.
    unspent_set: BTreeMap<note::Commitment, Note>,
    /// Notes that we have spent.
    spent_set: BTreeMap<note::Commitment, Note>,
    /// Map of note commitment to full transaction data for transactions we have visibility into.
    transactions: BTreeMap<note::Commitment, Option<Vec<u8>>>,
    /// Key material.
    wallet: Wallet,
}

impl ClientState {
    pub fn new(wallet: Wallet) -> Self {
        Self {
            last_block_height: 0,
            note_commitment_tree: merkle::BridgeTree::new(MAX_MERKLE_CHECKPOINTS_CLIENT),
            nullifier_map: BTreeMap::new(),
            unspent_set: BTreeMap::new(),
            spent_set: BTreeMap::new(),
            transactions: BTreeMap::new(),
            wallet,
        }
    }

    /// Generate a new diversified `Address` and its corresponding `DetectionKey`.
    pub fn new_address(&mut self, label: String) -> (usize, Address, fmd::DetectionKey) {
        self.wallet.new_address(label)
    }

    /// Generate a new transaction.
    pub fn new_transaction<R: RngCore + CryptoRng>(
        &mut self,
        mut rng: &mut R,
        fee: u64,
    ) -> Result<Transaction, penumbra_crypto::transaction::Error> {
        // xx Could populate chain_id from the info endpoint on the node, or at least
        // error if there is an inconsistency

        Transaction::build_with_root(self.note_commitment_tree.root2())
            .set_fee(&mut rng, fee)
            .set_chain_id(CURRENT_CHAIN_ID.to_string())
            .finalize(rng)
    }

    /// Returns the last block height the client state has synced up to.
    pub fn last_block_height(&self) -> u32 {
        self.last_block_height
    }

    /// Scan the provided block and update the client state.
    ///
    /// The provided block must be the one immediately following [`Self::last_block_height`].
    pub fn scan_block(
        &mut self,
        CompactBlock { height, fragments }: CompactBlock,
    ) -> Result<(), anyhow::Error> {
        if height != self.last_block_height + 1 {
            return Err(anyhow::anyhow!(
                "incorrect block height in `scan_block`; expected {} but got {}",
                self.last_block_height + 1,
                height
            ));
        }

        for StateFragment {
            note_commitment,
            ephemeral_key,
            encrypted_note,
        } in fragments.into_iter()
        {
            // Unconditionally insert the note commitment into the merkle tree
            let note_commitment = note_commitment
                .as_ref()
                .try_into()
                .context("invalid note commitment")?;
            self.note_commitment_tree.append(&note_commitment);

            // Try to decrypt the encrypted note using the ephemeral key and persistent incoming
            // viewing key
            if let Ok(note) = Note::decrypt(
                encrypted_note.as_ref(),
                &self.wallet.incoming_viewing_key(),
                &ephemeral_key
                    .as_ref()
                    .try_into()
                    .context("invalid ephemeral key")?,
            ) {
                // Mark the most-recently-inserted note commitment (the one corresponding to this
                // note) as worth keeping track of, because it's ours
                self.note_commitment_tree.witness();

                // Insert the note associated with its computed nullifier into the nullifier map
                let (pos, _auth_path) = self
                    .note_commitment_tree
                    .authentication_path(&note_commitment)
                    .expect("we just witnessed this commitment");
                self.nullifier_map.insert(
                    self.wallet
                        .full_viewing_key()
                        .derive_nullifier(pos, &note_commitment),
                    note_commitment,
                );

                // Insert the note into the received set
                self.unspent_set.insert(note_commitment, note.clone());
            }
        }

        // Remember that we've scanned this block & we're ready for the next one.
        self.last_block_height += 1;

        Ok(())
    }
}
