use std::collections::BTreeMap;

use penumbra_crypto::{asset, memo::MemoPlaintext, merkle::TreeExt, Address, Note, Value};
use penumbra_stake::{IdentityKey, RateData, STAKING_TOKEN_ASSET_ID};
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
        Send {
            dest_address: Box<Address>,
            value: Value,
            memo: String,
        },
        Fee {
            amount: u64,
        },
        DelegateOrUndelegate {
            flow: DelegateFlow,
            rate_data: RateData,
        },
    }

    #[derive(Debug, Clone, Copy)]
    pub(super) enum DelegateFlow {
        Delegate { unbonded_amount: u64 },
        Undelegate { delegation_amount: u64 },
    }

    impl DelegateFlow {
        pub fn amount(&self) -> u64 {
            match self {
                DelegateFlow::Delegate { unbonded_amount } => *unbonded_amount,
                DelegateFlow::Undelegate { delegation_amount } => *delegation_amount,
            }
        }

        pub fn is_delegate(&self) -> bool {
            matches!(self, DelegateFlow::Delegate { .. })
        }
    }
}

impl Action {
    /// Create a new send action.
    pub fn send(dest_address: Address, value: Value, memo: String) -> Action {
        Self(action::Inner::Send {
            value,
            dest_address: Box::new(dest_address),
            memo,
        })
    }

    /// Create a new fee action.
    pub fn fee(fee: u64) -> Action {
        Self(action::Inner::Fee { amount: fee })
    }

    /// Create a new delegate action.
    pub fn delegate(rate_data: RateData, unbonded_amount: u64) -> Action {
        Self(action::Inner::DelegateOrUndelegate {
            flow: action::DelegateFlow::Delegate { unbonded_amount },
            rate_data,
        })
    }

    /// Create a new undelegate action.
    pub fn undelegate(rate_data: RateData, delegation_amount: u64) -> Action {
        Self(action::Inner::DelegateOrUndelegate {
            flow: action::DelegateFlow::Undelegate { delegation_amount },
            rate_data,
        })
    }
}

/// The remaining actions to perform after a transaction is confirmed.
///
/// Use [`super::ClientState::continue_with_remainder`] to process this into another [`Transaction`]
/// after the first transaction is confirmed, and iterate until there is no more [`Remainder`].
#[derive(derivative::Derivative, Clone, Debug, Default)]
pub struct Continuation {
    spends: BTreeMap<asset::Id, u64>,
    outputs: BTreeMap<asset::Id, u64>,
    delegations: BTreeMap<IdentityKey, u64>,
    undelegations: BTreeMap<IdentityKey, u64>,
    rate_data: BTreeMap<IdentityKey, RateData>,
    fee: u64,
    output_contents: Vec<(Address, Value, MemoPlaintext)>,
}

impl super::ClientState {
    /// Build a transaction (and possible remainder) from a remainder of a previous transaction.
    pub fn continue_with<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        source_address: Option<u64>,
        continuation: Continuation,
    ) -> anyhow::Result<Transaction> {
        let (transaction, continuations) =
            self.compile_with_continuation(rng, source_address, continuation, Vec::new())?;
        if continuations.is_empty() {
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
    pub(super) fn compile_transaction<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        source_address: Option<u64>,
        actions: Vec<Action>,
    ) -> anyhow::Result<(Transaction, Vec<Continuation>)> {
        self.compile_with_continuation(rng, source_address, Continuation::default(), actions)
    }

    /// General helper function that implements both [`continue_with`] and [`compile_transaction`].
    ///
    /// This takes a list of actions and a suspended state of "actions yet to do" (already collected
    /// into separated tallies), and produces a single transaction that should be performed now, and
    /// a list of continuations which should be performed in sequence later.
    ///
    /// In the case where this is called in `compile_transaction`, the continuation is empty. In the
    /// case where this is called in `coniinue_with`, the returned `Vec` of continuations is
    /// expected to be empty, because it should never be the case that a `Continuation` is such that
    /// it must be turned into multiple transactions.
    #[instrument(skip(self, rng, actions))]
    fn compile_with_continuation<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        source_address: Option<u64>,
        continuation: Continuation,
        actions: Vec<Action>,
    ) -> anyhow::Result<(Transaction, Vec<Continuation>)> {
        let self_address = self.wallet.address_by_index(source_address.unwrap_or(0))?.1;
        let mut total = continuation;

        for Action(action) in actions {
            use action::Inner::*;

            match action {
                Fee { amount } => {
                    if amount > 0 {
                        tracing::trace!(?amount, "adding fee to transaction");
                        total.fee += amount;
                    }
                }
                Send {
                    dest_address,
                    value,
                    memo,
                } => {
                    if value.amount > 0 {
                        tracing::trace!(?value, "adding spend to transaction");
                        *total.spends.entry(value.asset_id).or_insert(0) += value.amount;

                        // Track the output of this send
                        tracing::trace!(?value, "adding output to transaction");
                        *total.outputs.entry(value.asset_id).or_insert(0) += value.amount;

                        // Collect the contents of the output
                        let memo = memo.try_into()?;
                        total.output_contents.push((*dest_address, value, memo));
                    }
                }
                DelegateOrUndelegate { flow, rate_data } => {
                    if flow.amount() > 0 {
                        // Compute the output value of this (un)delegation
                        let delegation_token = rate_data.identity_key.delegation_token();
                        let (input_value, output_value) = if flow.is_delegate() {
                            // When delegating:
                            (
                                // Input an amount of the staking token
                                Value {
                                    amount: flow.amount(),
                                    asset_id: *STAKING_TOKEN_ASSET_ID,
                                },
                                // Output an amount of the delegation token computed by the rate data
                                Value {
                                    amount: rate_data.delegation_amount(flow.amount()),
                                    asset_id: delegation_token.id(),
                                },
                            )
                        } else {
                            (
                                // Input an amount of the delegation token
                                Value {
                                    amount: flow.amount(),
                                    asset_id: delegation_token.id(),
                                },
                                // Output an amount of the staking token computed by the rate data
                                Value {
                                    amount: rate_data.unbonded_amount(flow.amount()),
                                    asset_id: *STAKING_TOKEN_ASSET_ID,
                                },
                            )
                        };

                        tracing::trace!(?input_value, "adding (un)delegation spend to transaction");
                        *total.spends.entry(input_value.asset_id).or_insert(0) +=
                            input_value.amount;

                        // Track the output of this send
                        tracing::trace!(
                            ?output_value,
                            "adding (un)delegation output to transaction"
                        );
                        *total.outputs.entry(output_value.asset_id).or_insert(0) +=
                            output_value.amount;

                        // Collect the contents of the output
                        total.output_contents.push((
                            self_address,
                            output_value,
                            Default::default(),
                        ));

                        // Keep track of this (un)delegation in the total (un)delegations
                        *(if flow.is_delegate() {
                            &mut total.delegations
                        } else {
                            &mut total.undelegations
                        })
                        .entry(rate_data.identity_key.clone())
                        .or_insert(0) += flow.amount();

                        // Accumulate the rate data herein discovered, or panic if it mismatches
                        // what we already know.
                        total
                            .rate_data
                            .entry(rate_data.identity_key.clone())
                            .and_modify(|current_rate_data| {
                                if *current_rate_data != rate_data {
                                    panic!(
                                        "mismatched rate data for identity key {}",
                                        rate_data.identity_key
                                    )
                                }
                            })
                            .or_insert(rate_data);
                    }
                }
            }
        }

        tracing::debug!(?total.outputs, "collected total outputs");

        // Add the fee to the total spends
        *total.spends.entry(*STAKING_TOKEN_ASSET_ID).or_insert(0) += total.fee;

        tracing::debug!(?total.spends, "collected total specified spends");

        // Collect the notes for all the spends
        let mut spend_notes = BTreeMap::<asset::Id, Vec<Note>>::new();
        for (asset_id, amount) in total.spends {
            // Get the notes to spend for this denomination
            let notes = self.notes_to_spend(rng, amount, asset_id, source_address)?;
            spend_notes.insert(asset_id, notes);
        }

        tracing::debug!(
            total_notes = ?{
                let mut total_notes = BTreeMap::<asset::Id, u64>::new();
                for (asset_id, notes) in spend_notes.iter() {
                    for note in notes {
                        *total_notes.entry(*asset_id).or_insert(0) += note.amount();
                    }
                }
                total_notes
            },
            "collected concrete notes to spend"
        );

        // Check that the total spend value is less than the total output value and compute total change
        let mut total_change = BTreeMap::<asset::Id, u64>::new();
        for (asset_id, notes) in spend_notes.iter() {
            total_change.insert(*asset_id, notes.iter().map(|n| n.amount()).sum());
        }
        // Subtract the output from the spend amount to get the total change
        for (asset_id, output) in total.outputs {
            let change = total_change.entry(asset_id).or_insert(0);
            *change = change.checked_sub(output).ok_or_else(|| {
                anyhow::anyhow!(
                    "not enough spent to cover outputs for asset id {}",
                    asset_id
                )
            })?;
        }

        tracing::debug!(?total_change, "collected total change");

        // Use the transaction builder to build the transaction
        let mut builder = Transaction::build();
        builder.set_chain_id(self.chain_id()?);

        // Set the fee
        builder.set_fee(total.fee);

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
        for (destination, value, memo) in total.output_contents {
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
        for (asset_id, amount) in total_change {
            if amount > 0 {
                tracing::trace!(?amount, "adding change output to builder");
                // We register the change note so the wallet UI can display it nicely
                self.register_change(builder.add_output_producing_note(
                    rng,
                    &self.wallet.address_by_index(source_address.unwrap_or(0))?.1,
                    Value { amount, asset_id },
                    MemoPlaintext::default(),
                    self.wallet.outgoing_viewing_key(),
                ));
            }
        }

        // TODO: add delegations and undelegations after doing grouping
        // TODO: handle automatic sweeping
        // TODO: handle splitting undelegation into break-change / undelegate
        // TODO: handle dummy notes and spends, unconditional change output

        tracing::debug!("finalizing transaction");
        let transaction = builder.finalize(rng, self.note_commitment_tree.root2())?;

        Ok((transaction, vec![]))
    }
}
