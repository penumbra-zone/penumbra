use std::{borrow::Cow, collections::HashMap};

use penumbra_crypto::{asset::Denom, memo::MemoPlaintext, merkle::TreeExt, Address, Note, Value};
use penumbra_stake::STAKING_TOKEN_DENOM;
use penumbra_transaction::Transaction;
use rand_core::{CryptoRng, RngCore};

/// The abstract description of an action performed by the wallet user.
#[derive(Debug, Clone)]
pub(super) struct Action<'a>(action::Inner<'a>);

mod action {
    use super::*;

    /// The abstract description of an action performed by the wallet user (internal enum).
    #[derive(Debug, Clone)]
    pub(super) enum Inner<'a> {
        Spend {
            value: Cow<'a, Value>,
        },
        Fee {
            /// Fee in upenumbra units.
            amount: u64,
        },
        Output {
            dest_address: &'a Address,
            value: Cow<'a, Value>,
            memo: String,
        },
    }
}

impl<'a> Action<'a> {
    /// Create a new spend action.
    pub fn spend(value: &'a Value) -> Action<'a> {
        Self(action::Inner::Spend {
            value: Cow::Borrowed(value),
        })
    }

    /// Create a new fee action.
    pub fn fee(fee: u64) -> Action<'a> {
        Self(action::Inner::Fee { amount: fee })
    }

    /// Create a new output action.
    pub fn output(dest_address: &'a Address, value: &'a Value, memo: String) -> Action<'a> {
        Self(action::Inner::Output {
            dest_address,
            value: Cow::Borrowed(value),
            memo,
        })
    }
}

/// The remaining actions to perform after a transaction is confirmed.
///
/// Use [`super::ClientState::continue_with_remainder`] to process this into another [`Transaction`]
/// after the first transaction is confirmed, and iterate until there is no more [`Remainder`].
#[derive(Debug, Clone)]
pub struct Remainder<'a> {
    actions: Vec<Action<'a>>,
    source_address: Option<u64>,
}

impl super::ClientState {
    /// Build a transaction (and possible remainder) from a remainder of a previous transaction.
    pub fn continue_with_remainder<'a, R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        Remainder {
            source_address,
            actions,
        }: Remainder<'a>,
    ) -> anyhow::Result<(Transaction, Option<Remainder<'a>>)> {
        self.compile_transaction(rng, source_address, actions)
    }

    /// Compile a list of abstract actions into a concrete transaction and an optional list of
    /// actions yet to perform (the remainder).
    ///
    /// This allows certain notionally single actions (such as undelegation, or sending large
    /// amounts that would require sweeping) to be broken up into steps, each of which can be
    /// executed independently (and must be, because each cannot be fully built until the previous
    /// has been confirmed).
    pub(super) fn compile_transaction<'a, R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        source_address: Option<u64>,
        actions: Vec<Action<'a>>,
    ) -> anyhow::Result<(Transaction, Option<Remainder<'a>>)> {
        let mut total_spends = HashMap::<Denom, u64>::new();
        let mut spend_notes = HashMap::<Denom, Vec<Note>>::new();
        let mut total_outputs = HashMap::<Denom, u64>::new();
        let mut outputs = Vec::<(&Address, Value, MemoPlaintext)>::new();
        let mut fee = 0;

        for Action(action) in actions {
            use action::Inner::*;
            match action {
                Fee { amount } => fee += amount,
                Spend { value } => {
                    let Value { amount, asset_id } = value.as_ref();

                    // Keep track of this spend in the total spends
                    let denom = self
                        .asset_cache()
                        .get(asset_id)
                        .ok_or_else(|| {
                            anyhow::anyhow!("unknown denomination for asset id {}", asset_id)
                        })?
                        .clone();
                    *total_spends.entry(denom).or_insert(0) += amount;
                }
                Output {
                    dest_address,
                    value,
                    memo,
                } => {
                    let Value { amount, asset_id } = value.as_ref();

                    // Keep track of this output in the total outputs
                    let denom = self
                        .asset_cache()
                        .get(asset_id)
                        .ok_or_else(|| {
                            anyhow::anyhow!("unknown denomination for asset id {}", asset_id)
                        })?
                        .clone();
                    *total_outputs.entry(denom).or_insert(0) += amount;

                    // Collect the contents of the output
                    let memo = memo.try_into()?;
                    let value = Value {
                        amount: *amount,
                        asset_id: *asset_id,
                    };
                    outputs.push((dest_address, value, memo));
                }
            }
        }

        // Add the fee to the total spends
        *total_spends.entry(STAKING_TOKEN_DENOM.clone()).or_insert(0) += fee;

        // Collect the notes for all the spends
        for (denom, amount) in total_spends {
            // Get the notes to spend for this denomination
            let notes = self.notes_to_spend(rng, amount, &denom, source_address)?;
            spend_notes.insert(denom, notes);
        }

        // Check that the total spend value is less than the total output value and compute total change
        let mut total_change = HashMap::<Denom, u64>::new();
        for (denom, notes) in spend_notes.iter() {
            total_change.insert(denom.clone(), notes.iter().map(|n| n.amount()).sum());
        }
        // Subtract the output from the spend amount to get the total change
        for (denom, output) in total_outputs {
            total_change
                .entry(denom.clone())
                .or_insert(0)
                .checked_sub(output)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "not enough spent to cover outputs for denomination {}",
                        denom
                    )
                })?;
        }

        // Use the transaction builder to build the transaction
        let mut builder = Transaction::build_with_root(self.note_commitment_tree.root2());
        builder.set_chain_id(self.chain_id()?);

        // Set the fee
        builder.set_fee(fee);

        // Add all the spends
        for notes in spend_notes.into_values() {
            for note in notes {
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
            builder.add_output(
                rng,
                destination,
                value,
                memo,
                self.wallet.outgoing_viewing_key(),
            );
        }

        // Add the change outputs
        for (denom, amount) in total_change {
            if amount > 0 {
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

        Ok((builder.finalize(rng)?, None))
    }
}
