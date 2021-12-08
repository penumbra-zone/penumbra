use anyhow::Context;
use penumbra_crypto::action::{output, spend};
use penumbra_crypto::Action;
use penumbra_proto::light_wallet::{CompactBlock, StateFragment};
use rand::seq::SliceRandom;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use tracing::instrument;

use penumbra_crypto::{
    asset, memo,
    merkle::{Frontier, NoteCommitmentTree, Tree, TreeExt},
    note, Address, FieldExt, Note, Nullifier, Transaction, Value, CURRENT_CHAIN_ID,
};

use crate::Wallet;

const MAX_MERKLE_CHECKPOINTS_CLIENT: usize = 10;

/// The time after which a locally cached pending transaction is considered to have failed.
const PENDING_TRANSACTION_TIMEOUT: Duration = Duration::from_secs(60);

/// State about the chain and our transactions.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    try_from = "serde_helpers::ClientStateHelper",
    into = "serde_helpers::ClientStateHelper"
)]
pub struct ClientState {
    /// The last block height we've scanned to, if any.
    last_block_height: Option<u32>,
    /// Note commitment tree.
    note_commitment_tree: NoteCommitmentTree,
    /// Our nullifiers and the notes they correspond to.
    nullifier_map: BTreeMap<Nullifier, note::Commitment>,
    /// Notes that we have received.
    unspent_set: BTreeMap<note::Commitment, Note>,
    /// Notes that we have spent but which have not yet been confirmed on-chain.
    pending_set: BTreeMap<note::Commitment, Note>,
    /// Notes that we anticipate receiving on-chain as change but which have not yet been confirmed.
    pending_change_set: BTreeMap<note::Commitment, Note>,
    /// Commitments for notes that are pending (either spent or change), indexed by timeout.
    pending_timeouts: BTreeMap<SystemTime, Vec<PendingCommitment>>,
    /// Notes that we have spent.
    spent_set: BTreeMap<note::Commitment, Note>,
    /// Map of note commitment to full transaction data for transactions we have visibility into.
    transactions: BTreeMap<note::Commitment, Option<Vec<u8>>>,
    /// Map of asset IDs to asset denominations.
    asset_registry: BTreeMap<asset::Id, String>,
    /// Key material.
    wallet: Wallet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PendingCommitment {
    Change(note::Commitment),
    Spend(note::Commitment),
}

impl ClientState {
    pub fn new(wallet: Wallet) -> Self {
        Self {
            last_block_height: None,
            note_commitment_tree: NoteCommitmentTree::new(MAX_MERKLE_CHECKPOINTS_CLIENT),
            nullifier_map: BTreeMap::new(),
            unspent_set: BTreeMap::new(),
            pending_set: BTreeMap::new(),
            pending_change_set: BTreeMap::new(),
            pending_timeouts: BTreeMap::new(),
            spent_set: BTreeMap::new(),
            transactions: BTreeMap::new(),
            asset_registry: BTreeMap::new(),
            wallet,
        }
    }

    /// Returns the wallet the state is tracking.
    pub fn wallet(&self) -> &Wallet {
        &self.wallet
    }

    /// Returns a mutable reference to the wallet the state is tracking.
    pub fn wallet_mut(&mut self) -> &mut Wallet {
        &mut self.wallet
    }

    /// Returns a list of notes to spend to release (at least) the provided value.
    ///
    /// If `source_address` is `Some`, restrict to only the notes sent to that address.
    pub fn notes_to_spend<R: CryptoRng + RngCore>(
        &self,
        rng: &mut R,
        amount: u64,
        denom: String,
        source_address: Option<u64>,
    ) -> Result<Vec<Note>, anyhow::Error> {
        let mut notes_by_address = self
            .unspent_notes_by_denom_and_address()
            .remove(&denom)
            .ok_or_else(|| anyhow::anyhow!("no notes of denomination {} found", denom))?;

        let mut notes = if let Some(source) = source_address {
            notes_by_address.remove(&source).ok_or_else(|| {
                anyhow::anyhow!(
                    "no notes of denomination {} found in address {}",
                    denom,
                    source
                )
            })?
        } else {
            notes_by_address.values().flatten().cloned().collect()
        };

        // Draw notes in a random order, to avoid leaking information via arity.
        notes.shuffle(rng);

        let mut notes_to_spend = Vec::new();
        let mut total_spend_value = 0u64;
        for note in notes.into_iter() {
            notes_to_spend.push(note.clone());
            total_spend_value += note.amount();

            if total_spend_value >= amount {
                break;
            }
        }

        if total_spend_value >= amount {
            Ok(notes_to_spend)
        } else {
            Err(anyhow::anyhow!(
                "not enough available notes for requested spend"
            ))
        }
    }

    /// Generate a new transaction.
    ///
    /// TODO: this function is too complicated, merge with
    /// builder API ?
    pub fn new_transaction<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        amount: u64,
        denomination: String,
        address: String,
        fee: u64,
        source_address: Option<u64>,
        tx_memo: Option<String>,
    ) -> Result<Transaction, anyhow::Error> {
        // xx Could populate chain_id from the info endpoint on the node, or at least
        // error if there is an inconsistency

        let now = SystemTime::now();

        let dest_address: Address =
            Address::from_str(&address).map_err(|_| anyhow::anyhow!("address is invalid"))?;

        let mut tx_builder = Transaction::build_with_root(self.note_commitment_tree.root2())
            .set_fee(fee)
            .set_chain_id(CURRENT_CHAIN_ID.to_string());

        let mut output_value = HashMap::<String, u64>::new();
        output_value.insert(denomination, amount);

        for (denom, amount) in &output_value {
            let memo: memo::MemoPlaintext = match tx_memo {
                Some(ref input_memo) => input_memo.clone().try_into()?,
                None => memo::MemoPlaintext([0u8; memo::MEMO_LEN_BYTES]),
            };
            tx_builder = tx_builder.add_output(
                rng,
                &dest_address,
                Value {
                    amount: *amount,
                    asset_id: asset::Denom(denom.to_string()).into(),
                },
                memo,
                self.wallet.outgoing_viewing_key(),
            );
        }

        // The value we need to spend is the output value, plus fees.
        let mut value_to_spend = output_value;
        if fee > 0 {
            *value_to_spend.entry("penumbra".into()).or_default() += fee;
        }

        for (denom, amount) in value_to_spend {
            // Only produce an output if the amount is greater than zero
            if amount == 0 {
                continue;
            }

            // Select a list of notes that provides at least the required amount.
            let notes = self.notes_to_spend(rng, amount, denom.clone(), source_address)?;
            let change_address = self
                .wallet
                .change_address(notes.last().expect("spent at least one note"))?;
            let spent: u64 = notes.iter().map(|note| note.amount()).sum();

            // Spend each of the notes we selected.
            for note in notes {
                let note_commitment = note.commit();

                // Add the note to the pending set
                self.pending_set.insert(note_commitment, note.clone());

                // Add a timeout which removes this note from the pending set sometime later
                self.pending_timeouts
                    .entry(now + PENDING_TRANSACTION_TIMEOUT)
                    .or_insert_with(|| Vec::with_capacity(1))
                    .push(PendingCommitment::Spend(note_commitment));

                let auth_path = self
                    .note_commitment_tree
                    .authentication_path(&note_commitment)
                    .unwrap();
                let merkle_path = (u64::from(auth_path.0) as usize, auth_path.1);
                let merkle_position = auth_path.0;
                tx_builder = tx_builder.add_spend(
                    rng,
                    self.wallet.spend_key(),
                    merkle_path,
                    note,
                    merkle_position,
                );
            }

            // Find out how much change we have and whether to add a change output.
            let change = spent - amount;
            if change > 0 {
                // xx: add memo handling
                let memo = memo::MemoPlaintext([0u8; 512]);
                let (note, new_tx_builder) = tx_builder.add_output_producing_note(
                    rng,
                    &change_address,
                    Value {
                        amount: change,
                        asset_id: asset::Denom(denom.to_string()).into(),
                    },
                    memo,
                    self.wallet.outgoing_viewing_key(),
                );

                // Update the tx builder (notice: this must be done explicitly because we broke from
                // the builder-by-assignment pattern above)
                tx_builder = new_tx_builder;

                let note_commitment = note.commit();

                // Add the note to the pending change set
                self.pending_change_set.insert(note_commitment, note);

                // Add a timeout which removes this note from the pending set sometime later
                self.pending_timeouts
                    .entry(now + PENDING_TRANSACTION_TIMEOUT)
                    .or_insert_with(|| Vec::with_capacity(1))
                    .push(PendingCommitment::Change(note_commitment));
            }
        }

        let transaction = tx_builder
            .finalize(rng)
            .map_err(|err| anyhow::anyhow!("error during transaction finalization: {}", err))?;

        Ok(transaction)
    }

    /// Returns an iterator over unspent `(address_id, denom, note)` triples.
    pub fn unspent_notes(&self) -> impl Iterator<Item = (u64, String, Note)> + '_ {
        self.unspent_set.values().cloned().map(|note| {
            // Any notes we have in the unspent set we will have the corresponding denominations
            // for since the notes and asset registry are both part of the sync.
            let denom = self
                .asset_registry
                .get(&note.asset_id())
                .expect("all asset IDs should have denominations stored locally")
                .clone();

            let index: u64 = self
                .wallet()
                .incoming_viewing_key()
                .index_for_diversifier(&note.diversifier())
                .try_into()
                .expect("diversifiers created by `pcli` are well-formed");

            (index, denom, note)
        })
    }

    /// Returns unspent notes, grouped by address index and then by denomination.
    pub fn unspent_notes_by_address_and_denom(&self) -> BTreeMap<u64, HashMap<String, Vec<Note>>> {
        let mut notemap = BTreeMap::default();

        for (index, denom, note) in self.unspent_notes() {
            notemap
                .entry(index)
                .or_insert_with(HashMap::default)
                .entry(denom)
                .or_insert_with(Vec::default)
                .push(note.clone());
        }

        notemap
    }

    /// Returns unspent notes, grouped by denomination and then by address index.
    pub fn unspent_notes_by_denom_and_address(&self) -> HashMap<String, BTreeMap<u64, Vec<Note>>> {
        let mut notemap = HashMap::default();

        for (index, denom, note) in self.unspent_notes() {
            notemap
                .entry(denom)
                .or_insert_with(BTreeMap::default)
                .entry(index)
                .or_insert_with(Vec::default)
                .push(note.clone());
        }

        notemap
    }

    /// Add asset to local asset registry if it doesn't exist.
    pub fn add_asset_to_registry(&mut self, asset_id: asset::Id, asset_denom: String) {
        if self
            .asset_registry
            .insert(asset_id, asset_denom.clone())
            .is_none()
        {
            tracing::debug!("found new asset: {}", asset_denom);
        }
    }

    /// Returns the last block height the client state has synced up to, if any.
    pub fn last_block_height(&self) -> Option<u32> {
        self.last_block_height
    }

    /// Remove all pending spends and change whose timeouts have expired, dropping pending change
    /// and returning pending spends to the unspent set.
    #[instrument]
    pub fn prune_timeouts(&mut self) {
        // Split off the expired timeouts from the pending timeouts in 3 steps (necessary because of
        // the API provided by `split_off`):
        // 1. Split out the unexpired timeouts, leaving only the expired timeouts inside `self`
        let mut unexpired = self.pending_timeouts.split_off(&SystemTime::now());
        // 2. Swap the unexpired timeouts for the expired timeouts inside `self`
        std::mem::swap(&mut self.pending_timeouts, &mut unexpired);
        // 3. Rename `unexpired` to `expired` because its meaning has changed per the swap above
        let expired = unexpired;

        // For every expired timeout, remove its note from the corresponding set, taking care to
        // move pending notes back to the unspent set
        for (timeout, commitments) in expired {
            for commitment in commitments {
                match commitment {
                    PendingCommitment::Change(commitment) => {
                        // Drop this pending change note -- this is permissible, because dropping a
                        // pending change note just means we forget about an output, not an input
                        if self.pending_change_set.remove(&commitment).is_none() {
                            tracing::debug!(
                                ?timeout,
                                ?commitment,
                                "attempted to drop a pending change note, but found that it did not exist",
                            );
                        }
                    }
                    PendingCommitment::Spend(commitment) => {
                        // IMPORTANT: Move this pending spend note back to the unspent note set --
                        // if we forget to do this, we'll permanently lose track of this note until
                        // we reset the wallet
                        if let Some(note) = self.pending_set.remove(&commitment) {
                            self.unspent_set.insert(commitment, note);
                        } else {
                            tracing::debug!(
                                ?timeout,
                                ?commitment,
                                "attempted to move a pending spend back to the unspent set, but found that it did not exist",
                            );
                        }
                    }
                }
            }
        }
    }

    /// Scan the provided block and update the client state.
    ///
    /// The provided block must be the one immediately following [`Self::last_block_height`].
    #[instrument(skip(self, fragments, nullifiers))]
    pub fn scan_block(
        &mut self,
        CompactBlock {
            height,
            fragments,
            nullifiers,
        }: CompactBlock,
    ) -> Result<(), anyhow::Error> {
        // We have to do a bit of a dance to use None as "-1" and handle genesis notes.
        match (height, self.last_block_height()) {
            (0, None) => {}
            (height, Some(last_height)) if height == last_height + 1 => {}
            _ => return Err(anyhow::anyhow!("unexpected block height")),
        }
        tracing::debug!(fragments_len = fragments.len(), "starting block scan");

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
            tracing::debug!(?note_commitment, "appending to note commitment tree");
            self.note_commitment_tree.append(&note_commitment);

            // Try to decrypt the encrypted note using the ephemeral key and persistent incoming
            // viewing key -- if it doesn't decrypt, it wasn't meant for us.
            if let Ok(note) = Note::decrypt(
                encrypted_note.as_ref(),
                self.wallet.incoming_viewing_key(),
                &ephemeral_key
                    .as_ref()
                    .try_into()
                    .context("invalid ephemeral key")?,
            ) {
                tracing::debug!(?note_commitment, ?note, "found note while scanning");
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

                // If the note was a pending change note, remove it from the pending change set
                self.pending_change_set.remove(&note_commitment);

                // Insert the note into the received set
                self.unspent_set.insert(note_commitment, note.clone());
            }
        }

        // Scan through the list of nullifiers to find those which refer to notes in our unspent set
        // or pending set and move them into the spent set.
        for nullifier in nullifiers {
            // Try to decode the nullifier
            if let Ok(nullifier) = nullifier.as_ref().try_into() {
                // Try to find the corresponding note commitment in the nullifier map
                if let Some(&note_commitment) = self.nullifier_map.get(&nullifier) {
                    // Try to remove the nullifier from the unspent set
                    if let Some(note) = self.unspent_set.remove(&note_commitment) {
                        // Insert the note into the spent set
                        self.spent_set.insert(note_commitment, note);
                        tracing::debug!(
                            ?nullifier,
                            "found nullifier for unspent note: marking it as spent"
                        )
                    } else if let Some(note) = self.pending_set.remove(&note_commitment) {
                        // Insert the note into the spent set
                        self.spent_set.insert(note_commitment, note);
                        tracing::debug!(
                            ?nullifier,
                            "found nullifier for pending note: marking it as spent"
                        )
                    } else if let Some(note) = self.pending_change_set.remove(&note_commitment) {
                        // Insert the note into the spent set
                        self.spent_set.insert(note_commitment, note);
                        tracing::debug!(
                            ?nullifier,
                            "found nullifier for pending change note: marking it as spent"
                        )
                    } else if self.spent_set.contains_key(&note_commitment) {
                        // If the nullifier is already in the spent set, it means we've already
                        // processed this note and it's spent
                        tracing::debug!(?nullifier, "found nullifier for already-spent note")
                    } else {
                        // This should never happen, because it would indicate that we either failed
                        // to update the spent set after removing a note from the unspent set, or we
                        // never inserted a note into the unspent set after tracking its nullifier
                        tracing::error!(
                            ?nullifier,
                            "found known nullifier but note is not in unspent set or spent set"
                        )
                    }
                } else {
                    // This happens all the time, but if you really want to see every nullifier,
                    // look at trace output
                    tracing::trace!(?nullifier, "found unknown nullifier while scanning");
                }
            } else {
                // This should never happen with a correct server
                tracing::warn!("invalid nullifier in received compact block");
            }
        }

        // Remember that we've scanned this block & we're ready for the next one.
        self.last_block_height = Some(height);
        tracing::debug!(self.last_block_height, "finished scanning block");

        Ok(())
    }
}

mod serde_helpers {
    use super::*;

    use serde_with::serde_as;

    #[serde_as]
    #[derive(Serialize, Deserialize)]
    pub struct ClientStateHelper {
        last_block_height: Option<u32>,
        #[serde_as(as = "serde_with::hex::Hex")]
        note_commitment_tree: Vec<u8>,
        nullifier_map: Vec<(String, String)>,
        unspent_set: Vec<(String, String)>,
        pending_set: Vec<(String, String)>,
        pending_change_set: Vec<(String, String)>,
        pending_timeouts: Vec<(SystemTime, Vec<PendingCommitmentHelper>)>,
        spent_set: Vec<(String, String)>,
        transactions: Vec<(String, String)>,
        asset_registry: Vec<(String, String)>,
        wallet: Wallet,
    }

    #[serde_as]
    #[derive(Serialize, Deserialize)]
    pub enum PendingCommitmentHelper {
        #[serde_as(as = "serde_with::hex::Hex")]
        Change(String),
        #[serde_as(as = "serde_with::hex::Hex")]
        Spend(String),
    }

    impl From<ClientState> for ClientStateHelper {
        fn from(state: ClientState) -> Self {
            Self {
                wallet: state.wallet,
                last_block_height: state.last_block_height,
                note_commitment_tree: bincode::serialize(&state.note_commitment_tree).unwrap(),
                nullifier_map: state
                    .nullifier_map
                    .iter()
                    .map(|(nullifier, commitment)| {
                        (
                            hex::encode(nullifier.0.to_bytes()),
                            hex::encode(commitment.0.to_bytes()),
                        )
                    })
                    .collect(),
                unspent_set: state
                    .unspent_set
                    .iter()
                    .map(|(commitment, note)| {
                        (
                            hex::encode(commitment.0.to_bytes()),
                            hex::encode(note.to_bytes()),
                        )
                    })
                    .collect(),
                pending_timeouts: state
                    .pending_timeouts
                    .into_iter()
                    .map(|(timeout, commitments)| {
                        (
                            timeout,
                            commitments
                                .into_iter()
                                .map(|commitment| match commitment {
                                    PendingCommitment::Change(commitment) => {
                                        PendingCommitmentHelper::Change(hex::encode(
                                            commitment.0.to_bytes(),
                                        ))
                                    }
                                    PendingCommitment::Spend(commitment) => {
                                        PendingCommitmentHelper::Spend(hex::encode(
                                            commitment.0.to_bytes(),
                                        ))
                                    }
                                })
                                .collect(),
                        )
                    })
                    .collect(),
                pending_set: state
                    .pending_set
                    .iter()
                    .map(|(commitment, note)| {
                        (
                            hex::encode(commitment.0.to_bytes()),
                            hex::encode(note.to_bytes()),
                        )
                    })
                    .collect(),
                pending_change_set: state
                    .pending_change_set
                    .iter()
                    .map(|(commitment, note)| {
                        (
                            hex::encode(commitment.0.to_bytes()),
                            hex::encode(note.to_bytes()),
                        )
                    })
                    .collect(),
                spent_set: state
                    .spent_set
                    .iter()
                    .map(|(commitment, note)| {
                        (
                            hex::encode(commitment.0.to_bytes()),
                            hex::encode(note.to_bytes()),
                        )
                    })
                    .collect(),
                asset_registry: state
                    .asset_registry
                    .iter()
                    .map(|(id, denom)| (hex::encode(id.to_bytes()), denom.clone()))
                    .collect(),
                // TODO: serialize full transactions
                transactions: vec![],
            }
        }
    }

    impl TryFrom<ClientStateHelper> for ClientState {
        type Error = anyhow::Error;
        fn try_from(state: ClientStateHelper) -> Result<Self, Self::Error> {
            let mut nullifier_map = BTreeMap::new();

            for (nullifier, commitment) in state.nullifier_map.into_iter() {
                nullifier_map.insert(
                    hex::decode(nullifier)?.as_slice().try_into()?,
                    hex::decode(commitment)?.as_slice().try_into()?,
                );
            }

            let mut unspent_set = BTreeMap::new();
            for (commitment, note) in state.unspent_set.into_iter() {
                unspent_set.insert(
                    hex::decode(commitment)?.as_slice().try_into()?,
                    hex::decode(note)?.as_slice().try_into()?,
                );
            }

            let mut pending_set = BTreeMap::new();
            for (commitment, note) in state.pending_set.into_iter() {
                pending_set.insert(
                    hex::decode(commitment)?.as_slice().try_into()?,
                    hex::decode(note)?.as_slice().try_into()?,
                );
            }

            let mut pending_change_set = BTreeMap::new();
            for (commitment, note) in state.pending_change_set.into_iter() {
                pending_change_set.insert(
                    hex::decode(commitment)?.as_slice().try_into()?,
                    hex::decode(note)?.as_slice().try_into()?,
                );
            }

            let mut spent_set = BTreeMap::new();
            for (commitment, note) in state.spent_set.into_iter() {
                spent_set.insert(
                    hex::decode(commitment)?.as_slice().try_into()?,
                    hex::decode(note)?.as_slice().try_into()?,
                );
            }

            let mut asset_registry = BTreeMap::new();
            for (id, denom) in state.asset_registry.into_iter() {
                asset_registry.insert(hex::decode(id)?.try_into()?, denom);
            }

            let now = SystemTime::now();
            let mut pending_timeouts = BTreeMap::new();
            for (timeout, commitments) in state.pending_timeouts.into_iter() {
                pending_timeouts.insert(timeout, {
                    // Build up a vector of *unexpired* pending commitments, deleting those which
                    // expire from their corresponding sets (just allocated above) as we go
                    let mut result = Vec::with_capacity(commitments.len());
                    for commitment in commitments {
                        match commitment {
                            PendingCommitmentHelper::Change(commitment) => {
                                let commitment = hex::decode(commitment)?.as_slice().try_into()?;
                                if now > timeout {
                                    // If timeout expired, drop this pending change note -- this is
                                    // permissible, because dropping a pending change note just
                                    // means we forget about an output, not an input
                                    pending_change_set.remove(&commitment);
                                } else {
                                    // If not, keep track of this timeout still
                                    result.push(PendingCommitment::Change(commitment));
                                }
                            }
                            PendingCommitmentHelper::Spend(commitment) => {
                                let commitment = hex::decode(commitment)?.as_slice().try_into()?;
                                if now > timeout {
                                    // IMPORTANT: If timeout expired, move this pending spend note
                                    // back to the unspent note set -- if we forget to do this,
                                    // we'll permanently lose track of this note until we reset the wallet
                                    if let Some(note) = pending_set.remove(&commitment) {
                                        unspent_set.insert(commitment, note);
                                    }
                                } else {
                                    // If not, keep track of this timeout still
                                    result.push(PendingCommitment::Spend(commitment));
                                }
                            }
                        }
                    }
                    result
                });
            }

            Ok(Self {
                wallet: state.wallet,
                last_block_height: state.last_block_height,
                note_commitment_tree: bincode::deserialize(&state.note_commitment_tree)?,
                nullifier_map,
                unspent_set,
                pending_set,
                pending_change_set,
                pending_timeouts,
                spent_set,
                asset_registry,
                // TODO: serialize full transactions
                transactions: Default::default(),
            })
        }
    }
}
