use std::collections::BTreeMap;

use penumbra_crypto::{asset, memo::MemoPlaintext, merkle::TreeExt, Address, Note, Value};
use penumbra_stake::{IdentityKey, RateData, STAKING_TOKEN_ASSET_ID};
use penumbra_transaction::Transaction;
use rand_core::{CryptoRng, RngCore};
use tracing::instrument;

/// The abstract description of an action performed by the wallet user.
#[derive(derivative::Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub(super) struct ActionDescription(action::Inner);

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

    /// An amount of a delegation or undelegation, tagged by which it is.
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

impl ActionDescription {
    /// Create a new send action.
    pub fn send(dest_address: Address, value: Value, memo: String) -> ActionDescription {
        Self(action::Inner::Send {
            value,
            dest_address: Box::new(dest_address),
            memo,
        })
    }

    /// Create a new fee action.
    pub fn fee(fee: u64) -> ActionDescription {
        Self(action::Inner::Fee { amount: fee })
    }

    /// Create a new delegate action.
    pub fn delegate(rate_data: RateData, unbonded_amount: u64) -> ActionDescription {
        Self(action::Inner::DelegateOrUndelegate {
            flow: action::DelegateFlow::Delegate { unbonded_amount },
            rate_data,
        })
    }

    /// Create a new undelegate action.
    pub fn undelegate(rate_data: RateData, delegation_amount: u64) -> ActionDescription {
        Self(action::Inner::DelegateOrUndelegate {
            flow: action::DelegateFlow::Undelegate { delegation_amount },
            rate_data,
        })
    }
}

impl super::ClientState {
    /// Build a transaction (and possible remainder) from a remainder of a previous transaction.
    #[instrument(skip(self, rng))]
    pub fn evaluate_description<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        description: TransactionDescription,
        source_address: Option<u64>,
    ) -> anyhow::Result<Transaction> {
        // Use the transaction builder to build the transaction
        let mut builder = Transaction::build();
        builder.set_chain_id(self.chain_id()?);

        // Set the fee
        builder.set_fee(description.fee);

        // Add all the spends
        for note in description.spends {
            tracing::trace!(value = ?note.value(), "adding spend note to builder");
            builder.add_spend(
                rng,
                &self.note_commitment_tree,
                self.wallet.spend_key(),
                note,
            )?;
        }

        // Add all the outputs
        for (asset_id, outputs) in description.outputs {
            for Output {
                destination,
                amount,
                memo,
                is_change,
            } in outputs
            {
                let value = Value { amount, asset_id };
                tracing::trace!(
                    ?value,
                    ?memo,
                    ?is_change,
                    "adding specified output to builder"
                );
                let note = builder.add_output_producing_note(
                    rng,
                    &destination,
                    value,
                    memo,
                    self.wallet.outgoing_viewing_key(),
                );
                if is_change {
                    self.register_change(note);
                }
            }
        }

        // Add all the delegations
        for (rate_data, amount) in description.delegations {
            tracing::trace!(
                ?rate_data,
                ?amount,
                "adding specified delegation to builder"
            );
            builder.add_delegation(&rate_data, amount);
        }

        // Add the undelegation, if any exists
        if let Some((rate_data, amount)) = description.undelegation {
            tracing::trace!(
                ?rate_data,
                ?amount,
                "adding specified undelegation to builder"
            );
            builder.add_undelegation(&rate_data, amount);
        }

        // TODO: handle dummy notes and spends, unconditional change output here

        tracing::debug!("finalizing transaction");
        Ok(builder.finalize(rng, self.note_commitment_tree.root2())?)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransactionDescription {
    spends: Vec<Note>,
    delegations: Vec<(RateData, u64)>,
    undelegation: Option<(RateData, u64)>,
    fee: u64,
    outputs: BTreeMap<asset::Id, Vec<Output>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Output {
    destination: Address,
    amount: u64,
    memo: MemoPlaintext,
    is_change: bool,
}

impl super::ClientState {
    /// Compile a list of abstract actions into a concrete transaction and an optional list of
    /// actions yet to perform (the remainder).
    ///
    /// This allows certain notionally single actions (such as undelegation, or sending large
    /// amounts that would require sweeping) to be broken up into steps, each of which can be
    /// executed independently (and must be, because each cannot be fully built until the previous
    /// has been confirmed).
    #[instrument(skip(self, rng, actions))]
    pub(super) fn compile_transaction<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        source_address: Option<u64>,
        actions: Vec<ActionDescription>,
    ) -> anyhow::Result<Vec<TransactionDescription>> {
        let self_address = self.wallet.address_by_index(source_address.unwrap_or(0))?.1;

        // Compute the tally of the actions first, then work with that to continue compiling
        let mut total = self.tally_transaction(source_address, actions)?;

        // TODO: split per undelegation into separate tallies

        // TODO: process each tally to split it into zero or more sweep/split transaction
        // descriptions followed by the desired action. Recall the invariant that an undelegate
        // transaction should never produce change, we must always do an exact split transaction
        // beforehand. How can we ensure we select the right note in this case, since our note
        // selection is random at present?

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
        for (asset_id, outputs) in total.outputs.iter() {
            let output_total = outputs.iter().map(|x| x.amount).sum();
            let change = total_change.entry(*asset_id).or_insert(0);
            *change = change.checked_sub(output_total).ok_or_else(|| {
                anyhow::anyhow!(
                    "not enough spent to cover outputs for asset id {}",
                    asset_id
                )
            })?;
        }
        tracing::debug!(?total_change, "collected total change");
        // Add change notes to the outputs
        for (asset_id, amount) in total_change {
            total.outputs.entry(asset_id).or_default().push(Output {
                destination: self_address,
                amount,
                memo: Default::default(),
                is_change: true,
            });
        }

        todo!("split into separate transaction descriptions")
    }
}

#[derive(derivative::Derivative, Clone, Debug, Default, PartialEq, Eq)]
struct Tally {
    spends: BTreeMap<asset::Id, u64>,
    delegations: BTreeMap<IdentityKey, (RateData, u64)>,
    undelegations: BTreeMap<IdentityKey, (RateData, u64)>,
    fee: u64,
    outputs: BTreeMap<asset::Id, Vec<Output>>,
}

impl super::ClientState {
    /// Compile a list of action descriptions into a tally of the information contained therein.
    ///
    /// This is a helper function for [`compile_transaction`].
    fn tally_transaction(
        &mut self,
        source_address: Option<u64>,
        actions: Vec<ActionDescription>,
    ) -> anyhow::Result<Tally> {
        let self_address = self.wallet.address_by_index(source_address.unwrap_or(0))?.1;
        let mut total = Tally::default();

        for ActionDescription(action) in actions {
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

                        // Collect the contents of the output
                        tracing::trace!(?value, "adding output to transaction");
                        total
                            .outputs
                            .entry(value.asset_id)
                            .or_default()
                            .push(Output {
                                destination: *dest_address,
                                amount: value.amount,
                                memo: memo.try_into()?,
                                is_change: false,
                            });
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
                            // When undelegating:
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

                        // Collect the contents of the output
                        tracing::trace!(
                            ?output_value,
                            "adding (un)delegation output to transaction"
                        );
                        total
                            .outputs
                            .entry(output_value.asset_id)
                            .or_default()
                            .push(Output {
                                destination: self_address,
                                amount: output_value.amount,
                                memo: Default::default(),
                                is_change: false,
                            });

                        // Keep track of this (un)delegation in the total (un)delegations...

                        // 1. select the appropriate map to update
                        (if flow.is_delegate() {
                            &mut total.delegations
                        } else {
                            &mut total.undelegations
                        })
                        // 2. select the entry for this validatoridentity key
                        .entry(rate_data.identity_key.clone())
                        // 3. panic if the rate data is already there and it differs from the one we have
                        .and_modify(|(extant_rate_data, _)| {
                            if *extant_rate_data != rate_data {
                                panic!("mismatched rate data");
                            }
                        })
                        // 4. insert the rate data and add the amount into the existing amount
                        .or_insert((rate_data, 0))
                        .1 += flow.amount();
                    }
                }
            }
        }

        tracing::debug!(?total.outputs, "collected total outputs");

        // Add the fee to the total spends
        *total.spends.entry(*STAKING_TOKEN_ASSET_ID).or_insert(0) += total.fee;

        tracing::debug!(?total.spends, "collected total specified spends");

        Ok(total)
    }
}
