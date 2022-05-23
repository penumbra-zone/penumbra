use std::{
    collections::{BTreeMap, HashMap},
    mem,
    time::{Duration, SystemTime},
};

use anyhow::anyhow;
use penumbra_chain::{params::ChainParams, sync::CompactBlock, Epoch};
use penumbra_crypto::{
    asset::{self, Denom},
    memo::MemoPlaintext,
    note, Address, DelegationToken, FieldExt, Note, NotePayload, Nullifier, Value,
    STAKING_TOKEN_ASSET_ID, STAKING_TOKEN_DENOM,
};
use penumbra_stake::{rate::RateData, validator};
use penumbra_tct as tct;
use penumbra_transaction::{
    plan::{ActionPlan, OutputPlan, SpendPlan, TransactionPlan},
    Fee, Transaction, WitnessData,
};
use rand::seq::SliceRandom;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::Wallet;

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
    /// Tiered commitment tree.
    note_commitment_tree: penumbra_tct::Tree,
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
    //transactions: BTreeMap<note::Commitment, Option<Vec<u8>>>,
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
            note_commitment_tree: tct::Tree::new(),
            nullifier_map: BTreeMap::new(),
            unspent_set: BTreeMap::new(),
            submitted_spend_set: BTreeMap::new(),
            submitted_change_set: BTreeMap::new(),
            spent_set: BTreeMap::new(),
            //transactions: BTreeMap::new(),
            asset_cache: Default::default(),
            wallet,
            chain_params: None,
        }
    }

    /// TODO: this will go away with wallet restructuring, where we'll
    /// record the position and return it with note queries
    pub fn position(&self, note: &Note) -> Option<tct::Position> {
        self.note_commitment_tree.position_of(note.commit())
    }

    /// Returns a reference to the note commitment tree.
    pub fn note_commitment_tree(&self) -> &tct::Tree {
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

    pub fn build_transaction<R: RngCore + CryptoRng>(
        &self,
        mut rng: R,
        plan: TransactionPlan,
    ) -> anyhow::Result<Transaction> {
        // Next, authorize the transaction, ...
        let auth_data = plan.authorize(&mut rng, self.wallet.spend_key());

        // ... build the witness data ...
        let witness_data = WitnessData {
            anchor: self.note_commitment_tree.root(),
            note_commitment_proofs: plan
                .spend_plans()
                .map(|spend| {
                    self.note_commitment_tree
                        .witness(spend.note.commit())
                        .ok_or_else(|| anyhow::anyhow!("missing auth path for note commitment"))
                })
                .collect::<Result<_, _>>()?,
        };

        // ... and then build the transaction:
        plan.build(
            &mut rng,
            self.wallet.full_viewing_key(),
            auth_data,
            witness_data,
        )
    }

    /// Generate a new transaction plan delegating stake
    #[instrument(skip(self, rng, rate_data))]
    pub fn plan_delegate<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        rate_data: RateData,
        unbonded_amount: u64,
        fee: u64,
        source_address: Option<u64>,
    ) -> Result<TransactionPlan, anyhow::Error> {
        // If the source address is set, send the delegation tokens to the same
        // address; otherwise, send them to the default address.
        let (_label, self_address) = self
            .wallet()
            .address_by_index(source_address.unwrap_or(0) as usize)?;

        let mut plan = TransactionPlan {
            chain_id: self.chain_id().ok_or_else(|| anyhow!("missing chain_id"))?,
            fee: Fee(fee),
            ..Default::default()
        };

        // Add the delegation action itself:
        plan.actions
            .push(rate_data.build_delegate(unbonded_amount).into());

        // Add an output to ourselves to record the delegation:
        plan.actions.push(
            OutputPlan::new(
                rng,
                Value {
                    amount: rate_data.delegation_amount(unbonded_amount),
                    asset_id: DelegationToken::new(rate_data.identity_key).id(),
                },
                self_address,
                MemoPlaintext::default(),
            )
            .into(),
        );

        // Add the required spends, and track change:
        let spend_amount = unbonded_amount + fee;
        let mut spent_amount = 0;
        for note in self.notes_to_spend(rng, spend_amount, &*STAKING_TOKEN_DENOM, source_address)? {
            spent_amount += note.amount();
            plan.actions
                .push(SpendPlan::new(rng, note.clone(), self.position(&note).unwrap()).into());
        }

        // Add a change note if we have change left over:
        let change_amount = spent_amount - spend_amount;
        // TODO: support dummy notes, and produce a change output unconditionally.
        // let change_note = if change_amount > 0 { ... } else { /* dummy note */}
        if change_amount > 0 {
            plan.actions.push(
                OutputPlan::new(
                    rng,
                    Value {
                        amount: change_amount,
                        asset_id: *STAKING_TOKEN_ASSET_ID,
                    },
                    self_address,
                    MemoPlaintext::default(),
                )
                .into(),
            );
        }

        Ok(plan)
    }

    /// Generate a new transaction plan delegating stake
    #[instrument(skip(self, rng))]
    pub fn plan_undelegate<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        rate_data: RateData,
        delegation_amount: u64,
        fee: u64,
        source_address: Option<u64>,
    ) -> Result<TransactionPlan, anyhow::Error> {
        // If the source address is set, send the delegation tokens to the same
        // address; otherwise, send them to the default address.
        let (_label, self_address) = self
            .wallet()
            .address_by_index(source_address.unwrap_or(0) as usize)?;

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

        let mut plan = TransactionPlan {
            chain_id: self.chain_id().ok_or_else(|| anyhow!("missing chain_id"))?,
            fee: Fee(fee),
            ..Default::default()
        };

        // Add the undelegation action itself:
        plan.actions
            .push(rate_data.build_undelegate(delegation_amount).into());

        // Add an output to ourselves to record the undelegation:
        plan.actions.push(
            OutputPlan::new(
                rng,
                Value {
                    amount: output_amount,
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                },
                self_address,
                MemoPlaintext::default(),
            )
            .into(),
        );

        // Add the required spends, and track change:
        let delegation_denom = DelegationToken::new(rate_data.identity_key).denom();
        let mut spent_amount = 0;
        for note in
            self.notes_to_spend(rng, delegation_amount, &delegation_denom, source_address)?
        {
            spent_amount += note.amount();
            plan.actions
                .push(SpendPlan::new(rng, note.clone(), self.position(&note).unwrap()).into());
        }

        let change_amount = spent_amount - delegation_amount;
        // TODO: support dummy notes, and produce a change output unconditionally.
        // let change_note = if change_amount > 0 { ... } else { /* dummy note */}
        if change_amount > 0 {
            plan.actions.push(
                OutputPlan::new(
                    rng,
                    Value {
                        amount: change_amount,
                        asset_id: delegation_denom.id(),
                    },
                    self_address,
                    MemoPlaintext::default(),
                )
                .into(),
            );
        }

        Ok(plan)
    }

    /// Generate a new transaction uploading a validator definition.
    #[instrument(skip(self, rng))]
    pub fn plan_validator_definition<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        new_validator: validator::Definition,
        fee: u64,
        source_address: Option<u64>,
    ) -> Result<TransactionPlan, anyhow::Error> {
        // If the source address is set, send fee change to the same
        // address; otherwise, send it to the default address.
        let (_label, self_address) = self
            .wallet()
            .address_by_index(source_address.unwrap_or(0) as usize)?;

        let mut plan = TransactionPlan {
            chain_id: self.chain_id().ok_or_else(|| anyhow!("missing chain_id"))?,
            fee: Fee(fee),
            ..Default::default()
        };

        plan.actions
            .push(ActionPlan::ValidatorDefinition(new_validator.into()));

        // Add the required spends, and track change:
        let spend_amount = fee;
        let mut spent_amount = 0;
        for note in self.notes_to_spend(rng, spend_amount, &*STAKING_TOKEN_DENOM, source_address)? {
            spent_amount += note.amount();
            plan.actions
                .push(SpendPlan::new(rng, note.clone(), self.position(&note).unwrap()).into());
        }
        // Add a change note if we have change left over:
        let change_amount = spent_amount - spend_amount;
        // TODO: support dummy notes, and produce a change output unconditionally.
        // let change_note = if change_amount > 0 { ... } else { /* dummy note */}
        if change_amount > 0 {
            plan.actions.push(
                OutputPlan::new(
                    rng,
                    Value {
                        amount: change_amount,
                        asset_id: *STAKING_TOKEN_ASSET_ID,
                    },
                    self_address,
                    MemoPlaintext::default(),
                )
                .into(),
            );
        }

        Ok(plan)
    }

    /// Generate a new transaction sending value to `dest_address`.
    #[instrument(skip(self, rng))]
    pub fn plan_send<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        values: &[Value],
        fee: u64,
        dest_address: Address,
        source_address: Option<u64>,
        tx_memo: Option<String>,
    ) -> Result<TransactionPlan, anyhow::Error> {
        let memo = if let Some(input_memo) = tx_memo {
            input_memo.as_bytes().try_into()?
        } else {
            MemoPlaintext::default()
        };

        let mut plan = TransactionPlan {
            chain_id: self.chain_id().ok_or_else(|| anyhow!("missing chain_id"))?,
            fee: Fee(fee),
            ..Default::default()
        };

        // Track totals of the output values rather than just processing
        // them individually, so we can plan the required spends.
        let mut output_value = HashMap::<Denom, u64>::new();
        for Value { amount, asset_id } in values {
            let denom = self
                .asset_cache()
                .get(asset_id)
                .ok_or_else(|| anyhow::anyhow!("unknown denomination for asset id {}", asset_id))?;
            output_value.insert(denom.clone(), *amount);
        }

        // Add outputs for the funds we want to send:
        for (denom, amount) in &output_value {
            plan.actions.push(
                OutputPlan::new(
                    rng,
                    Value {
                        amount: *amount,
                        asset_id: denom.id(),
                    },
                    dest_address,
                    memo.clone(),
                )
                .into(),
            );
        }

        // The value we need to spend is the output value, plus fees.
        let mut value_to_spend = output_value;
        if fee > 0 {
            *value_to_spend
                .entry(STAKING_TOKEN_DENOM.clone())
                .or_default() += fee;
        }

        // Add the required spends:
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
                plan.actions
                    .push(SpendPlan::new(rng, note.clone(), self.position(&note).unwrap()).into());
            }

            // Find out how much change we have and whether to add a change output.
            let change = spent - amount;
            if change > 0 {
                plan.actions.push(
                    OutputPlan::new(
                        rng,
                        Value {
                            amount: change,
                            asset_id: denom.id(),
                        },
                        change_address,
                        MemoPlaintext::default(),
                    )
                    .into(),
                );
            }
        }

        Ok(plan)
    }

}
