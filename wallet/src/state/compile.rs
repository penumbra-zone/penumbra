use std::collections::HashMap;

use penumbra_crypto::{asset::Denom, memo::MemoPlaintext, merkle::TreeExt, Address, Note, Value};
use penumbra_stake::STAKING_TOKEN_DENOM;
use penumbra_transaction::Transaction;
use rand_core::{CryptoRng, RngCore};
use tracing::instrument;

/// The abstract description of an action performed by the wallet user.
#[derive(derivative::Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub(super) struct Action(action::Inner);

mod action {
    use super::*;

    /// The abstract description of an action performed by the wallet user (internal enum).
    #[derive(Debug, Clone)]
    pub(super) enum Inner {
        Spend {
            value: Value,
        },
        Fee {
            /// Fee in upenumbra units.
            amount: u64,
        },
        Output {
            dest_address: Box<Address>,
            value: Value,
            memo: String,
        },
    }
}

impl Action {
    /// Create a new spend action.
    pub fn spend(value: Value) -> Action {
        Self(action::Inner::Spend { value })
    }

    /// Create a new fee action.
    pub fn fee(fee: u64) -> Action {
        Self(action::Inner::Fee { amount: fee })
    }

    /// Create a new output action.
    pub fn output(dest_address: Address, value: Value, memo: String) -> Action {
        Self(action::Inner::Output {
            dest_address: Box::new(dest_address),
            value,
            memo,
        })
    }
}

/// The remaining actions to perform after a transaction is confirmed.
///
/// Use [`super::ClientState::continue_with_remainder`] to process this into another [`Transaction`]
/// after the first transaction is confirmed, and iterate until there is no more [`Remainder`].
#[derive(derivative::Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub struct Continuation {
    actions: Vec<Action>,
    #[derivative(Debug = "ignore")]
    source_address: Option<u64>,
}

impl super::ClientState {
    /// Build a transaction (and possible remainder) from a remainder of a previous transaction.
    pub fn continue_with<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        Continuation {
            source_address,
            actions,
        }: Continuation,
    ) -> anyhow::Result<Transaction> {
        let (transaction, remainder) = self.compile_transaction(rng, source_address, actions)?;
        if remainder.is_empty() {
            Ok(transaction)
        } else {
            panic!("internal error: compiling remainder resulted in additional remainder");
        }
    }

    /// Compile a list of abstract actions into a concrete transaction and an optional list of
    /// actions yet to perform (the remainder).
    ///
    /// This allows certain notionally single actions (such as undelegation, or sending large
    /// amounts that would require sweeping) to be broken up into steps, each of which can be
    /// executed independently (and must be, because each cannot be fully built until the previous
    /// has been confirmed).
    #[instrument(skip(self, rng, actions))]
    pub(super) fn compile_transaction<'a, R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        source_address: Option<u64>,
        actions: Vec<Action>,
    ) -> anyhow::Result<(Transaction, Vec<Continuation>)> {
        let mut total_spends = HashMap::<Denom, u64>::new();
        let mut total_outputs = HashMap::<Denom, u64>::new();
        let mut outputs = Vec::<(Address, Value, MemoPlaintext)>::new();
        let mut fee = 0;

        for Action(action) in actions {
            use action::Inner::*;
            match action {
                Fee { amount } => {
                    if amount > 0 {
                        tracing::trace!(?fee, "adding fee to transaction");
                        fee += amount;
                    }
                }
                Spend { value } => {
                    if value.amount > 0 {
                        // Keep track of this spend in the total spends
                        let denom = self
                            .asset_cache()
                            .get(&value.asset_id)
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "unknown denomination for asset id {}",
                                    value.asset_id
                                )
                            })?
                            .clone();

                        tracing::trace!(
                            ?denom,
                            amount = value.amount,
                            "adding spend to transaction"
                        );
                        *total_spends.entry(denom).or_insert(0) += value.amount;
                    }
                }
                Output {
                    dest_address,
                    value,
                    memo,
                } => {
                    if value.amount > 0 {
                        // Keep track of this output in the total outputs
                        let denom = self
                            .asset_cache()
                            .get(&value.asset_id)
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "unknown denomination for asset id {}",
                                    &value.asset_id
                                )
                            })?
                            .clone();

                        tracing::trace!(?denom, amount = ?value.amount, "adding output to transaction");
                        *total_outputs.entry(denom).or_insert(0) += value.amount;

                        // Collect the contents of the output
                        let memo = memo.try_into()?;
                        outputs.push((*dest_address, value, memo));
                    }
                }
            }
        }

        tracing::debug!(?total_outputs, "collected total outputs");

        // Add the fee to the total spends
        *total_spends.entry(STAKING_TOKEN_DENOM.clone()).or_insert(0) += fee;

        tracing::debug!(?total_spends, "collected total specified spends");

        // Collect the notes for all the spends
        let mut spend_notes = HashMap::<Denom, Vec<Note>>::new();
        for (denom, amount) in total_spends {
            // Get the notes to spend for this denomination
            let notes = self.notes_to_spend(rng, amount, &denom, source_address)?;
            spend_notes.insert(denom, notes);
        }

        tracing::debug!(
            total_notes = ?{
                let mut total_notes = HashMap::<Denom, u64>::new();
                for (denom, notes) in spend_notes.iter() {
                    for note in notes {
                        *total_notes.entry(denom.clone()).or_insert(0) += note.amount();
                    }
                }
                total_notes
            },
            "collected concrete notes to spend"
        );

        // Check that the total spend value is less than the total output value and compute total change
        let mut total_change = HashMap::<Denom, u64>::new();
        for (denom, notes) in spend_notes.iter() {
            total_change.insert(denom.clone(), notes.iter().map(|n| n.amount()).sum());
        }
        // Subtract the output from the spend amount to get the total change
        for (denom, output) in total_outputs {
            let change = total_change.entry(denom.clone()).or_insert(0);
            *change = change.checked_sub(output).ok_or_else(|| {
                anyhow::anyhow!(
                    "not enough spent to cover outputs for denomination {}",
                    denom
                )
            })?;
        }

        tracing::debug!(?total_change, "collected total change");

        // Use the transaction builder to build the transaction
        let mut builder = Transaction::build();
        builder.set_chain_id(self.chain_id()?);

        // Set the fee
        builder.set_fee(fee);

        // Add all the spends
        for notes in spend_notes.into_values() {
            for note in notes {
                tracing::trace!(value = ?note.value(), "adding spend note to builder");
                builder.add_spend(
                    rng,
                    &self.note_commitment_tree,
                    self.wallet.spend_key(),
                    note,
                )?;
            }
        }

        // Add all the intially specified outputs
        for (destination, value, memo) in outputs {
            tracing::trace!(
                value = ?value,
                memo = ?memo,
                "adding specified output to builder"
            );
            builder.add_output(
                rng,
                &destination,
                value,
                memo,
                self.wallet.outgoing_viewing_key(),
            );
        }

        // Add the change outputs
        for (denom, amount) in total_change {
            if amount > 0 {
                tracing::trace!(?amount, "adding change output to builder");
                // We register the change note so the wallet UI can display it nicely
                self.register_change(builder.add_output_producing_note(
                    rng,
                    &self.wallet.address_by_index(source_address.unwrap_or(0))?.1,
                    Value {
                        amount,
                        asset_id: denom.into(),
                    },
                    MemoPlaintext::default(),
                    self.wallet.outgoing_viewing_key(),
                ));
            }
        }

        // TODO: handle automatic sweeping
        // TODO: handle splitting undelegation into break-change / undelegate
        // TODO: handle dummy notes and spends, unconditional change output

        tracing::debug!("finalizing transaction");
        let transaction = builder.finalize(rng, self.note_commitment_tree.root2())?;

        Ok((transaction, vec![]))
    }
}
