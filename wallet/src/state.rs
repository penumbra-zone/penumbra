use std::{
    collections::{BTreeMap, HashMap},
    mem,
    time::{Duration, SystemTime},
};

use anyhow::{anyhow, Context};
use penumbra_chain::params::ChainParams;
use penumbra_crypto::{
    asset::{self, Denom},
    memo,
    merkle::{Frontier, NoteCommitmentTree, Tree, TreeExt},
    note, Address, FieldExt, Note, Nullifier, Value,
};
use penumbra_proto::light_wallet::{CompactBlock, StateFragment};
use penumbra_stake::{RateData, ValidatorDefinition, STAKING_TOKEN_ASSET_ID, STAKING_TOKEN_DENOM};
use penumbra_transaction::Transaction;
use rand::seq::SliceRandom;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::Wallet;

const MAX_MERKLE_CHECKPOINTS_CLIENT: usize = 10;

/// The time after which a locally cached submitted transaction is considered to have failed.
const SUBMITTED_TRANSACTION_TIMEOUT: Duration = Duration::from_secs(60);

/// State about the chain and our transactions.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    try_from = "serde_helpers::ClientStateHelper",
    into = "serde_helpers::ClientStateHelper"
)]
pub struct ClientState {
    /// The last block height we've scanned to, if any.
    last_block_height: Option<u64>,
    /// Note commitment tree.
    note_commitment_tree: NoteCommitmentTree,
    /// Our nullifiers and the notes they correspond to.
    nullifier_map: BTreeMap<Nullifier, note::Commitment>,
    /// Notes that we have received.
    unspent_set: BTreeMap<note::Commitment, Note>,
    /// Notes that we have spent but which have not yet been confirmed on-chain.
    submitted_spend_set: BTreeMap<note::Commitment, (SystemTime, Note)>,
    /// Notes that we anticipate receiving on-chain as change but which have not yet been confirmed.
    submitted_change_set: BTreeMap<note::Commitment, (SystemTime, Note)>,
    /// Notes that we have spent.
    spent_set: BTreeMap<note::Commitment, Note>,
    /// Map of note commitment to full transaction data for transactions we have visibility into.
    transactions: BTreeMap<note::Commitment, Option<Vec<u8>>>,
    /// Map of asset IDs to (raw) asset denominations.
    asset_cache: asset::Cache,
    /// Key material.
    wallet: Wallet,
    /// Global chain parameters. May not have been fetched yet.
    chain_params: Option<ChainParams>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SubmittedNoteCommitment {
    Change(note::Commitment),
    Spend(note::Commitment),
}

#[derive(Clone, Debug)]
/// A note which has not yet been confirmed on the chain as spent.
pub enum UnspentNote<'a> {
    /// A note which is ours to spend immediately: neither a submitted spent note waiting for
    /// confirmation nor a submitted change output waiting for confirmation.
    Ready(&'a Note),
    /// A note which we have submitted in a spend transaction but which has not yet been
    /// confirmed on the chain (so if the transaction is rejected, we may get it back again).
    SubmittedSpend(&'a Note),
    /// A note which resulted as predicted change from a spend transaction, but which has not
    /// yet been confirmed on the chain (so we cannot spend it yet).
    SubmittedChange(&'a Note),
}

impl<'a> UnspentNote<'a> {
    /// Returns the underlying note if it is [`UnspentNote::Ready`].
    pub fn as_ready(&self) -> Option<&'a Note> {
        match self {
            UnspentNote::Ready(note) => Some(note),
            _ => None,
        }
    }
}

impl AsRef<Note> for UnspentNote<'_> {
    fn as_ref(&self) -> &Note {
        match self {
            UnspentNote::Ready(note) => note,
            UnspentNote::SubmittedSpend(note) => note,
            UnspentNote::SubmittedChange(note) => note,
        }
    }
}

impl ClientState {
    pub fn new(wallet: Wallet) -> Self {
        Self {
            last_block_height: None,
            note_commitment_tree: NoteCommitmentTree::new(MAX_MERKLE_CHECKPOINTS_CLIENT),
            nullifier_map: BTreeMap::new(),
            unspent_set: BTreeMap::new(),
            submitted_spend_set: BTreeMap::new(),
            submitted_change_set: BTreeMap::new(),
            spent_set: BTreeMap::new(),
            transactions: BTreeMap::new(),
            asset_cache: Default::default(),
            wallet,
            chain_params: None,
        }
    }

    /// Returns a reference to the note commitment tree.
    pub fn note_commitment_tree(&self) -> &NoteCommitmentTree {
        &self.note_commitment_tree
    }

    /// Returns a reference to the client state's asset cache.
    pub fn asset_cache(&self) -> &asset::Cache {
        &self.asset_cache
    }

    /// Returns a mutable reference to the client state's asset cache.
    pub fn asset_cache_mut(&mut self) -> &mut asset::Cache {
        &mut self.asset_cache
    }

    /// Returns the wallet the state is tracking.
    pub fn wallet(&self) -> &Wallet {
        &self.wallet
    }

    /// Returns the global chain parameters.
    pub fn chain_params(&self) -> Option<&ChainParams> {
        self.chain_params.as_ref()
    }

    /// Returns a mutable reference to the global chain parameters.
    pub fn chain_params_mut(&mut self) -> &mut Option<ChainParams> {
        &mut self.chain_params
    }

    /// Returns a mutable reference to the wallet the state is tracking.
    pub fn wallet_mut(&mut self) -> &mut Wallet {
        &mut self.wallet
    }

    /// Register a change note.
    ///
    /// This is a note we create, sent to ourselves, with the "change" from a
    /// transaction.  Tracking these notes allows the wallet UI to display
    /// self-addressed value to users, so they're not surprised that their
    /// wallet suddenly has much less than they expected (e.g., when they split
    /// a large note into a small output + change).
    ///
    /// This registration is temporary; if the note is not observed on-chain
    /// before some timeout, it will be forgotten.
    pub fn register_change(&mut self, note: Note) {
        let timeout = SystemTime::now() + SUBMITTED_TRANSACTION_TIMEOUT;
        let commitment = note.commit();

        tracing::debug!(?commitment, value = ?note.value(), "adding note to submitted change set");
        self.submitted_change_set
            .insert(commitment, (timeout, note));
    }

    /// Register a note as spent.
    ///
    /// This marks the note as having been spent (pending confirmation) by the
    /// chain.  Tracking these notes allows the wallet to not accidentally
    /// attempt to double-spend a note, just because the first transaction that
    /// spent it hasn't been finalized yet.
    ///
    /// This registration is temporary; if the spend is not observed on-chain
    /// before some timeout, it will be forgotten, and the note marked as unspent again.
    pub fn register_spend(&mut self, note: &Note) {
        let commitment = note.commit();
        tracing::debug!(?commitment, value = ?note.value(), "moving note from unspent set to submitted spend set");
        let note = self.unspent_set.remove(&commitment).unwrap();
        let timeout = SystemTime::now() + SUBMITTED_TRANSACTION_TIMEOUT;
        self.submitted_spend_set.insert(commitment, (timeout, note));
    }

    /// Returns a list of notes to spend to release (at least) the provided
    /// value.
    ///
    /// The returned notes are removed from the unspent set and marked as having
    /// been spent (pending confirmation) by the chain.
    ///
    /// If `source_address` is `Some`, restrict to only the notes sent to that
    /// address.
    pub fn notes_to_spend<R: CryptoRng + RngCore>(
        &mut self,
        rng: &mut R,
        amount: u64,
        denom: &Denom,
        source_address: Option<u64>,
    ) -> Result<Vec<Note>, anyhow::Error> {
        let mut notes_by_address = self
            .unspent_notes_by_denom_and_address()
            .remove(denom)
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
            // A note is only spendable if it has been confirmed on chain to us (change outputs
            // cannot be spent yet because they do not have a position):
            if let UnspentNote::Ready(note) = note {
                notes_to_spend.push(note.clone());
                total_spend_value += note.amount();

                if total_spend_value >= amount {
                    break;
                }
            }
        }

        if total_spend_value >= amount {
            // Before returning the notes to the caller, mark them as having been
            // spent.  (If the caller does not spend them, or the tx fails, etc.,
            // this state will be erased after the timeout).
            for note in &notes_to_spend {
                self.register_spend(note);
            }

            Ok(notes_to_spend)
        } else {
            Err(anyhow::anyhow!(
                "not enough available notes for requested spend"
            ))
        }
    }

    /// Returns the chain id, if the chain parameters are set.
    pub fn chain_id(&self) -> Option<String> {
        self.chain_params().map(|p| p.chain_id.clone())
    }

    /// Generate a new transaction delegating stake
    #[instrument(skip(self, rng, rate_data))]
    pub fn build_delegate<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        rate_data: RateData,
        unbonded_amount: u64,
        fee: u64,
        source_address: Option<u64>,
    ) -> Result<Transaction, anyhow::Error> {
        // If the source address is set, send the delegation tokens to the same
        // address; otherwise, send them to the default address.
        let (_label, self_address) = self
            .wallet()
            .address_by_index(source_address.unwrap_or(0) as usize)?;

        let mut tx_builder = Transaction::build_with_root(self.note_commitment_tree.root2());

        tx_builder
            .set_fee(fee)
            .set_chain_id(self.chain_id().ok_or_else(|| anyhow!("missing chain_id"))?)
            .add_delegation(&rate_data, unbonded_amount);

        let spend_amount = unbonded_amount + fee;
        let mut spent_amount = 0;

        for note in self.notes_to_spend(rng, spend_amount, &*STAKING_TOKEN_DENOM, source_address)? {
            spent_amount += note.amount();
            tx_builder.add_spend(
                rng,
                &self.note_commitment_tree,
                self.wallet.spend_key(),
                note,
            )?;
        }

        let delegation_note = tx_builder.add_output_producing_note(
            rng,
            &self_address,
            Value {
                amount: rate_data.delegation_amount(unbonded_amount),
                asset_id: rate_data.identity_key.delegation_token().id(),
            },
            memo::MemoPlaintext([0u8; memo::MEMO_LEN_BYTES]),
            self.wallet.outgoing_viewing_key(),
        );

        let change_amount = spent_amount - spend_amount;
        // TODO: support dummy notes, and produce a change output unconditionally.
        // let change_note = if change_amount > 0 { ... } else { /* dummy note */}
        if change_amount > 0 {
            let change_note = tx_builder.add_output_producing_note(
                rng,
                &self_address,
                Value {
                    amount: change_amount,
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                },
                memo::MemoPlaintext([0u8; memo::MEMO_LEN_BYTES]),
                self.wallet.outgoing_viewing_key(),
            );
            self.register_change(change_note);
        }

        self.register_change(delegation_note);

        tx_builder.finalize(rng).map_err(Into::into)
    }

    /// Generate a new transaction delegating stake
    #[instrument(skip(self, rng))]
    pub fn build_undelegate<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        rate_data: RateData,
        delegation_amount: u64,
        fee: u64,
        source_address: Option<u64>,
    ) -> Result<Transaction, anyhow::Error> {
        // If the source address is set, send the delegation tokens to the same
        // address; otherwise, send them to the default address.
        let (_label, self_address) = self
            .wallet()
            .address_by_index(source_address.unwrap_or(0) as usize)?;

        let mut tx_builder = Transaction::build_with_root(self.note_commitment_tree.root2());

        tx_builder
            .set_fee(fee)
            .set_chain_id(self.chain_id().ok_or_else(|| anyhow!("missing chain_id"))?)
            .add_undelegation(&rate_data, delegation_amount);

        // Because the outputs of an undelegation are quarantined, we want to
        // avoid any unnecessary change outputs, so we pay fees out of the
        // unbonded amount.
        let unbonded_amount = rate_data.unbonded_amount(delegation_amount);
        let output_amount = unbonded_amount.checked_sub(fee).ok_or_else(|| {
            anyhow::anyhow!(
                "unbonded amount {} from delegation amount {} is insufficient to pay fees {}",
                unbonded_amount,
                delegation_amount,
                fee
            )
        })?;

        let delegation_denom = rate_data.identity_key.delegation_token().denom();

        // XXX if the undelegation is for less than their total amount of delegation tokens,
        // all of their remaining delegation tokens will also be quarantined.
        // this sucks lmao
        let mut spent_amount = 0;

        for note in
            self.notes_to_spend(rng, delegation_amount, &delegation_denom, source_address)?
        {
            spent_amount += note.amount();
            tx_builder.add_spend(
                rng,
                &self.note_commitment_tree,
                self.wallet.spend_key(),
                note,
            )?;
        }

        let output_note = tx_builder.add_output_producing_note(
            rng,
            &self_address,
            Value {
                amount: output_amount,
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
            memo::MemoPlaintext([0u8; memo::MEMO_LEN_BYTES]),
            self.wallet.outgoing_viewing_key(),
        );

        let change_amount = spent_amount - delegation_amount;
        // TODO: support dummy notes, and produce a change output unconditionally.
        // let change_note = if change_amount > 0 { ... } else { /* dummy note */}
        if change_amount > 0 {
            let change_note = tx_builder.add_output_producing_note(
                rng,
                &self_address,
                Value {
                    amount: change_amount,
                    asset_id: delegation_denom.id(),
                },
                memo::MemoPlaintext([0u8; memo::MEMO_LEN_BYTES]),
                self.wallet.outgoing_viewing_key(),
            );
            self.register_change(change_note);
        }

        self.register_change(output_note);

        tx_builder.finalize(rng).map_err(Into::into)
    }

    /// Generate a new transaction uploading a validator definition.
    #[instrument(skip(self, rng))]
    pub fn build_validator_definition<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        new_validator: ValidatorDefinition,
        fee: u64,
        source_address: Option<u64>,
    ) -> Result<Transaction, anyhow::Error> {
        let mut tx_builder = Transaction::build_with_root(self.note_commitment_tree.root2());

        tx_builder
            .set_fee(fee)
            .set_chain_id(self.chain_id().ok_or_else(|| anyhow!("missing chain_id"))?);

        // Add the Validator to the tx_builder.
        tx_builder.add_validator_definition(new_validator);

        // If there are any fees, they need to be spent.
        let mut value_to_spend = HashMap::<Denom, u64>::new();
        if fee > 0 {
            *value_to_spend
                .entry(STAKING_TOKEN_DENOM.clone())
                .or_default() += fee;
        }

        for (denom, amount) in value_to_spend {
            // Only produce an output if the amount is greater than zero
            if amount == 0 {
                continue;
            }

            // Select a list of notes that provides at least the required amount.
            let notes: Vec<Note> = self.notes_to_spend(rng, amount, &denom, source_address)?;
            let change_address = self
                .wallet
                .change_address(notes.last().expect("spent at least one note"))?;
            let spent: u64 = notes.iter().map(|note| note.amount()).sum();

            // Spend each of the notes we selected.
            for note in notes {
                tx_builder.add_spend(
                    rng,
                    &self.note_commitment_tree,
                    // The active wallet pays the fees for all the validators it is defining.
                    self.wallet.spend_key(),
                    note,
                )?;
            }

            // Find out how much change we have and whether to add a change output.
            let change = spent - amount;
            if change > 0 {
                // xx: add memo handling
                let memo = memo::MemoPlaintext([0u8; 512]);
                let note = tx_builder.add_output_producing_note(
                    rng,
                    &change_address,
                    Value {
                        amount: change,
                        asset_id: denom.id(),
                    },
                    memo,
                    self.wallet.outgoing_viewing_key(),
                );

                self.register_change(note);
            }
        }

        let transaction = tx_builder
            .finalize(rng)
            .map_err(|err| anyhow::anyhow!("error during transaction finalization: {}", err))?;

        Ok(transaction)
    }

    /// Generate a new transaction sending value to `dest_address`.
    #[instrument(skip(self, rng))]
    pub fn build_send<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        values: &[Value],
        fee: u64,
        dest_address: Address,
        source_address: Option<u64>,
        tx_memo: Option<String>,
    ) -> Result<Transaction, anyhow::Error> {
        let mut tx_builder = Transaction::build_with_root(self.note_commitment_tree.root2());

        tx_builder
            .set_fee(fee)
            .set_chain_id(self.chain_id().ok_or_else(|| anyhow!("missing chain_id"))?);

        let mut output_value = HashMap::<Denom, u64>::new();
        for Value { amount, asset_id } in values {
            let denom = self
                .asset_cache()
                .get(asset_id)
                .ok_or_else(|| anyhow::anyhow!("unknown denomination for asset id {}", asset_id))?;
            output_value.insert(denom.clone(), *amount);
        }

        for (denom, amount) in &output_value {
            let memo: memo::MemoPlaintext = match tx_memo {
                Some(ref input_memo) => input_memo.clone().try_into()?,
                None => memo::MemoPlaintext([0u8; memo::MEMO_LEN_BYTES]),
            };
            tx_builder.add_output(
                rng,
                &dest_address,
                Value {
                    amount: *amount,
                    asset_id: denom.id(),
                },
                memo,
                self.wallet.outgoing_viewing_key(),
            );
        }

        // The value we need to spend is the output value, plus fees.
        let mut value_to_spend = output_value;
        if fee > 0 {
            *value_to_spend
                .entry(STAKING_TOKEN_DENOM.clone())
                .or_default() += fee;
        }

        for (denom, amount) in value_to_spend {
            // Only produce an output if the amount is greater than zero
            if amount == 0 {
                continue;
            }

            // Select a list of notes that provides at least the required amount.
            let notes: Vec<Note> = self.notes_to_spend(rng, amount, &denom, source_address)?;
            let change_address = self
                .wallet
                .change_address(notes.last().expect("spent at least one note"))?;
            let spent: u64 = notes.iter().map(|note| note.amount()).sum();

            // Spend each of the notes we selected.
            for note in notes {
                tx_builder.add_spend(
                    rng,
                    &self.note_commitment_tree,
                    self.wallet.spend_key(),
                    note,
                )?;
            }

            // Find out how much change we have and whether to add a change output.
            let change = spent - amount;
            if change > 0 {
                // xx: add memo handling
                let memo = memo::MemoPlaintext([0u8; 512]);
                let note = tx_builder.add_output_producing_note(
                    rng,
                    &change_address,
                    Value {
                        amount: change,
                        asset_id: denom.id(),
                    },
                    memo,
                    self.wallet.outgoing_viewing_key(),
                );

                self.register_change(note);
            }
        }

        let transaction = tx_builder
            .finalize(rng)
            .map_err(|err| anyhow::anyhow!("error during transaction finalization: {}", err))?;

        Ok(transaction)
    }

    /// Returns an iterator over unspent `(address_id, denom, note)` triples.
    ///
    /// Notes are [`UnspentNote`]s, which describe whether the note is ready to spend, part of a
    /// submitted output, or part of submitted change expected to be received.
    pub fn unspent_notes(&self) -> impl Iterator<Item = (u64, Denom, UnspentNote)> + '_ {
        self.unspent_set
            .values()
            .map(UnspentNote::Ready)
            .chain(
                self.submitted_spend_set
                    .values()
                    .map(|(_, note)| UnspentNote::SubmittedSpend(note)),
            )
            .chain(
                self.submitted_change_set
                    .values()
                    .map(|(_, note)| UnspentNote::SubmittedChange(note)),
            )
            .map(|note| {
                // Any notes we have in the unspent set we will have the corresponding denominations
                // for since the notes and asset registry are both part of the sync.
                let denom = self
                    .asset_cache
                    .get(&note.as_ref().asset_id())
                    .expect("all asset IDs should have denominations stored locally")
                    .clone();

                let index: u64 = self
                    .wallet()
                    .incoming_viewing_key()
                    .index_for_diversifier(&note.as_ref().diversifier())
                    .try_into()
                    .expect("diversifiers created by `pcli` are well-formed");

                (index, denom, note)
            })
    }

    /// Returns unspent notes, grouped by address index and then by denomination.
    pub fn unspent_notes_by_address_and_denom(
        &self,
    ) -> BTreeMap<u64, BTreeMap<Denom, Vec<UnspentNote>>> {
        let mut notemap = BTreeMap::default();

        for (index, denom, note) in self.unspent_notes() {
            notemap
                .entry(index)
                .or_insert_with(BTreeMap::default)
                .entry(denom)
                .or_insert_with(Vec::default)
                .push(note.clone());
        }

        notemap
    }

    /// Returns unspent notes, grouped by denomination and then by address index.
    pub fn unspent_notes_by_denom_and_address(
        &self,
    ) -> BTreeMap<Denom, BTreeMap<u64, Vec<UnspentNote>>> {
        let mut notemap = BTreeMap::default();

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

    /// Returns the last block height the client state has synced up to, if any.
    pub fn last_block_height(&self) -> Option<u64> {
        self.last_block_height
    }

    /// Remove all submitted spends and change whose timeouts have expired, dropping submitted change
    /// and returning submitted spends to the unspent set.
    #[instrument(
        skip(self),
        fields(
            submitted_spend_set_size = self.submitted_spend_set.len(),
            submitted_change_set_size = self.submitted_change_set.len(),
        ),
    )]
    pub fn prune_timeouts(&mut self) {
        let now = SystemTime::now();

        // Pull out the submitted sets and set them in `self` to the empty map
        let submitted_spend_set = mem::take(&mut self.submitted_spend_set);
        let submitted_change_set = mem::take(&mut self.submitted_change_set);

        // Iterate over submitted spends and put back into the unspent set any whose timeouts have
        // already expired
        for (note_commitment, (timeout, note)) in submitted_spend_set {
            if now > timeout {
                // IMPORTANT: we must recover the submitted spend note or else we can't ever spend
                // it without resetting and resyncing the wallet entirely
                if self.spent_set.contains_key(&note_commitment) {
                    tracing::debug!(
                        value = ?note.value(),
                        "timeout expired for submitted note already in spent set"
                    )
                } else {
                    tracing::debug!(
                        value = ?note.value(),
                        "timeout expired without confirmation for submitted note, putting it back into the unspent set"
                    );
                    self.unspent_set.insert(note_commitment, note);
                }
            } else {
                self.submitted_spend_set
                    .insert(note_commitment, (timeout, note));
            }
        }

        // Iterate over submitted change and **DROP** any whose timeouts have already expired
        for (note_commitment, (timeout, note)) in submitted_change_set {
            if now > timeout {
                // We can drop submitted change notes, because they are outputs of the transaction
                // and therefore we can expect that either the transaction will fail, or we will
                // receive them again later
                tracing::debug!(
                    value = ?note.value(),
                    "timeout expired without confirmation for submitted change, dropping it"
                );
            } else {
                self.submitted_change_set
                    .insert(note_commitment, (timeout, note));
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
            (height, last_height) => {
                return Err(anyhow::anyhow!(
                    "unexpected block height {}, expecting {:?}",
                    height,
                    last_height.map(|x| x + 1)
                ))
            }
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

                // If the note was a submitted change note, remove it from the submitted change set
                if self.submitted_change_set.remove(&note_commitment).is_some() {
                    tracing::debug!(value = ?note.value(), "found submitted change note while scanning, removing it from the submitted change set");
                }

                // Insert the note into the received set
                self.unspent_set.insert(note_commitment, note.clone());
            }
        }

        // Scan through the list of nullifiers to find those which refer to notes in our unspent
        // set, submitted change set, or submitted spend set and move them into the spent set.
        for nullifier in nullifiers {
            // Try to decode the nullifier
            let nullifier = nullifier.as_ref().try_into()?;

            // Try to find the corresponding note commitment in the nullifier map
            if let Some(&note_commitment) = self.nullifier_map.get(&nullifier) {
                // Try to remove the nullifier from the unspent set
                if let Some(note) = self.unspent_set.remove(&note_commitment) {
                    // Insert the note into the spent set
                    tracing::debug!(
                        value = ?note.value(),
                        ?nullifier,
                        "found nullifier for unspent note, marking it as spent"
                    );
                    self.spent_set.insert(note_commitment, note);
                    self.note_commitment_tree.remove_witness(&note_commitment);
                } else if let Some((_, note)) = self.submitted_spend_set.remove(&note_commitment) {
                    // Insert the note into the spent set
                    tracing::debug!(
                        value = ?note.value(),
                        ?nullifier,
                        "found nullifier for submitted spend note, marking it as spent"
                    );
                    self.spent_set.insert(note_commitment, note);
                    self.note_commitment_tree.remove_witness(&note_commitment);
                } else if let Some((_, note)) = self.submitted_change_set.remove(&note_commitment) {
                    // Insert the note into the spent set
                    tracing::debug!(
                        value = ?note.value(),
                        ?nullifier,
                        "found nullifier for submitted change note, marking it as spent"
                    );
                    self.spent_set.insert(note_commitment, note);
                    self.note_commitment_tree.remove_witness(&note_commitment);
                } else if self.spent_set.contains_key(&note_commitment) {
                    // If the nullifier is already in the spent set, it means we've already
                    // processed this note and it's spent. This should never happen
                    tracing::warn!(
                        ?nullifier,
                        "found nullifier for already-spent note, possibly corrupted state?"
                    )
                }
            } else {
                // This happens all the time, but if you really want to see every nullifier,
                // look at trace output
                tracing::trace!(?nullifier, "found unknown nullifier while scanning");
            }
        }

        // Remember that we've scanned this block & we're ready for the next one.
        self.last_block_height = Some(height);
        tracing::debug!(self.last_block_height, "finished scanning block");

        Ok(())
    }
}

mod serde_helpers {
    use serde_with::serde_as;

    use super::*;

    #[serde_as]
    #[derive(Serialize, Deserialize)]
    pub struct ClientStateHelper {
        wallet: Wallet, // this should be at the top to make `wallet reset` faster
        last_block_height: Option<u64>,
        #[serde_as(as = "serde_with::hex::Hex")]
        note_commitment_tree: Vec<u8>,
        nullifier_map: Vec<(String, String)>,
        unspent_set: Vec<(String, String)>,
        #[serde(default, alias = "pending_set")]
        submitted_spend_set: Vec<(String, SystemTime, String)>,
        #[serde(default, alias = "pending_change_set")]
        submitted_change_set: Vec<(String, SystemTime, String)>,
        spent_set: Vec<(String, String)>,
        transactions: Vec<(String, String)>,
        asset_registry: Vec<(asset::Id, String)>,
        chain_params: Option<ChainParams>,
    }

    #[serde_as]
    #[derive(Serialize, Deserialize)]
    pub enum SubmittedNoteCommitmentHelper {
        #[serde_as(as = "serde_with::hex::Hex")]
        Change(String),
        #[serde_as(as = "serde_with::hex::Hex")]
        Spend(String),
    }

    impl From<SubmittedNoteCommitment> for SubmittedNoteCommitmentHelper {
        fn from(submitted_note_commitment: SubmittedNoteCommitment) -> Self {
            match submitted_note_commitment {
                SubmittedNoteCommitment::Change(commitment) => {
                    SubmittedNoteCommitmentHelper::Change(hex::encode(commitment.0.to_bytes()))
                }
                SubmittedNoteCommitment::Spend(commitment) => {
                    SubmittedNoteCommitmentHelper::Spend(hex::encode(commitment.0.to_bytes()))
                }
            }
        }
    }

    impl TryFrom<SubmittedNoteCommitmentHelper> for SubmittedNoteCommitment {
        type Error = anyhow::Error;

        fn try_from(
            submitted_note_commitment: SubmittedNoteCommitmentHelper,
        ) -> Result<Self, anyhow::Error> {
            Ok(match submitted_note_commitment {
                SubmittedNoteCommitmentHelper::Change(commitment) => {
                    let commitment = hex::decode(commitment)?.as_slice().try_into()?;
                    SubmittedNoteCommitment::Change(commitment)
                }
                SubmittedNoteCommitmentHelper::Spend(commitment) => {
                    let commitment = hex::decode(commitment)?.as_slice().try_into()?;
                    SubmittedNoteCommitment::Spend(commitment)
                }
            })
        }
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
                submitted_spend_set: state
                    .submitted_spend_set
                    .iter()
                    .map(|(commitment, (timeout, note))| {
                        (
                            hex::encode(commitment.0.to_bytes()),
                            *timeout,
                            hex::encode(note.to_bytes()),
                        )
                    })
                    .collect(),
                submitted_change_set: state
                    .submitted_change_set
                    .iter()
                    .map(|(commitment, (timeout, note))| {
                        (
                            hex::encode(commitment.0.to_bytes()),
                            *timeout,
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
                    .asset_cache
                    .iter()
                    .map(|(id, denom)| (*id, denom.to_string()))
                    .collect(),
                // TODO: serialize full transactions
                transactions: vec![],
                chain_params: state.chain_params,
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

            let mut submitted_spend_set = BTreeMap::new();
            for (commitment, timeout, note) in state.submitted_spend_set.into_iter() {
                submitted_spend_set.insert(
                    hex::decode(commitment)?.as_slice().try_into()?,
                    (timeout, hex::decode(note)?.as_slice().try_into()?),
                );
            }

            let mut submitted_change_set = BTreeMap::new();
            for (commitment, timeout, note) in state.submitted_change_set.into_iter() {
                submitted_change_set.insert(
                    hex::decode(commitment)?.as_slice().try_into()?,
                    (timeout, hex::decode(note)?.as_slice().try_into()?),
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
                asset_registry.insert(id, denom);
            }

            Ok(Self {
                wallet: state.wallet,
                last_block_height: state.last_block_height,
                note_commitment_tree: bincode::deserialize(&state.note_commitment_tree)?,
                nullifier_map,
                unspent_set,
                submitted_spend_set,
                submitted_change_set,
                spent_set,
                asset_cache: asset_registry.try_into()?,
                // TODO: serialize full transactions
                transactions: Default::default(),
                chain_params: state.chain_params,
            })
        }
    }
}
