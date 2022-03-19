use anyhow::{anyhow, Result};
use penumbra_crypto::{memo, Value};
use penumbra_transaction::Transaction;
use rand_core::OsRng;
use structopt::StructOpt;

use crate::{ClientStateFile, Opt};

#[derive(Debug, StructOpt)]
pub enum TxCmd {
    /// Send transaction to the node.
    Send {
        /// The destination address to send funds to.
        #[structopt(long)]
        to: String,
        /// The amounts to send, written as typed values 1.87penumbra, 12cubes, etc.
        values: Vec<String>,
        /// The transaction fee (paid in upenumbra).
        #[structopt(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[structopt(long)]
        source: Option<u64>,
        /// Optional. Set the transaction's memo field to the provided text.
        #[structopt(long)]
        memo: Option<String>,
    },
    /// Sweeps small notes of the same denomination into a few larger notes.
    ///
    /// Since Penumbra transactions reveal their arity (how many spends,
    /// outputs, etc), but transactions are unlinkable from each other, it is
    /// slightly preferable to sweep small notes into larger ones in an isolated
    /// "sweep" transaction, rather than at the point that they should be spent.
    ///
    /// Currently, only zero-fee sweep transactions are implemented.
    Sweep,
}

impl TxCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn needs_sync(&self) -> bool {
        match self {
            TxCmd::Send { .. } => true,
            TxCmd::Sweep { .. } => true,
        }
    }

    pub async fn exec(&self, opt: &Opt, state: &mut ClientStateFile) -> Result<()> {
        match self {
            TxCmd::Send {
                values,
                to,
                fee,
                source: from,
                memo,
            } => {
                // Parse all of the values provided.
                let values = values
                    .iter()
                    .map(|v| v.parse())
                    .collect::<Result<Vec<Value>, _>>()?;
                let to = to
                    .parse()
                    .map_err(|_| anyhow::anyhow!("address is invalid"))?;

                let transaction =
                    state.build_send(&mut OsRng, &values, *fee, to, *from, memo.clone())?;

                opt.submit_transaction(&transaction).await?;
                // Only commit the state if the transaction was submitted
                // successfully, so that we don't store pending notes that will
                // never appear on-chain.
                state.commit()?;
            }
            TxCmd::Sweep => {
                sweep(opt, state).await?;
            }
        }
        Ok(())
    }
}

// This code is done outside of the client state as a test case for whether it's
// possible to use that interface to implement bespoke note handling.
//
// For each (account, denom) pair, we do a parallel, SWEEP_COUNT-to-1 sweep of
// as many of that denom's notes as possible.  This command can be run multiple
// times, and tells the user about the results of sweeping.
//
// The original implementation counted an estimate of the number of notes
// following the sweep and used it to decide whether to recurse, but doing so
// requires async recursion, and working that out didn't seem like a great
// effort/benefit tradeoff.
async fn sweep(opt: &Opt, state: &mut ClientStateFile) -> Result<()> {
    const SWEEP_COUNT: usize = 8;
    let mut transactions = Vec::new();
    // The UnspentNote struct owns a borrow of a note, preventing use of
    // any mutable methods on the ClientState, so we have to accumulate
    // changes to be applied later.
    let mut spent_notes = Vec::new();
    let mut change_notes = Vec::new();
    let unspent = state.unspent_notes_by_address_and_denom();
    for (id, label, addr) in state.wallet().addresses() {
        if unspent.get(&(id as u64)).is_none() {
            continue;
        }
        tracing::info!(?id, ?label, "processing address");
        for (denom, notes) in unspent.get(&(id as u64)).unwrap().iter() {
            // Extract only the ready notes of this denomination.
            let mut notes = notes
                .iter()
                .filter_map(|n| n.as_ready())
                .collect::<Vec<_>>();
            // Sort notes by amount, ascending, so the biggest notes are at the end...
            notes.sort_by(|a, b| a.value().amount.cmp(&b.value().amount));
            // ... so that when we use chunks_exact, we get SWEEP_COUNT sized
            // chunks, ignoring the biggest notes in the remainder.
            for group in notes.chunks_exact(SWEEP_COUNT) {
                tracing::info!(?denom, "building sweep transaction");
                let mut tx_builder =
                    Transaction::build_with_root(state.note_commitment_tree().root());
                tx_builder.set_fee(0).set_chain_id(
                    state
                        .chain_id()
                        .ok_or_else(|| anyhow!("missing chain_id"))?,
                );

                for note in group {
                    tx_builder.add_spend(
                        &mut OsRng,
                        state.note_commitment_tree(),
                        state.wallet().spend_key(),
                        (*note).clone(),
                    )?;
                    spent_notes.push((*note).clone());
                }
                let change = tx_builder.add_output_producing_note(
                    &mut OsRng,
                    &addr,
                    Value {
                        amount: group.iter().map(|n| n.amount()).sum(),
                        asset_id: denom.id(),
                    },
                    memo::MemoPlaintext([0u8; 512]),
                    state.wallet().outgoing_viewing_key(),
                );
                change_notes.push(change);

                transactions.push(tx_builder.finalize(&mut OsRng).map_err(|err| {
                    anyhow::anyhow!("error during transaction finalization: {}", err)
                })?);
            }
        }
    }

    // Now drop `unspent` so that we can mutate the client state.
    std::mem::drop(unspent);

    let num_sweeps = transactions.len();
    tracing::info!(num_sweeps, "submitting sweeps");
    for transaction in transactions {
        opt.submit_transaction_unconfirmed(&transaction).await?;
    }
    for spend in spent_notes {
        state.register_spend(&spend);
    }
    for change in change_notes {
        state.register_change(change);
    }

    // Print a message to the user, so they can find out what we did.
    if num_sweeps > 0 {
        println!(
            "swept {} notes into {} new outputs; rerun to sweep further",
            num_sweeps * SWEEP_COUNT,
            num_sweeps,
        );
    } else {
        println!("finished sweeping");
        // Terminate with a non-zero exit code so it's easy to script
        // sweeping in a loop
        std::process::exit(9);
    }

    Ok(())
}
