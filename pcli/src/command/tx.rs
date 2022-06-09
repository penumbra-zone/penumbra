use anyhow::Result;
use penumbra_crypto::{FullViewingKey, Value};
use penumbra_custody::CustodyClient;
use penumbra_view::ViewClient;
use penumbra_wallet::{build_transaction, plan};
use rand_core::OsRng;

use crate::Opt;

#[derive(Debug, clap::Subcommand)]
pub enum TxCmd {
    /// Send transaction to the node.
    Send {
        /// The destination address to send funds to.
        #[clap(long)]
        to: String,
        /// The amounts to send, written as typed values 1.87penumbra, 12cubes, etc.
        values: Vec<String>,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[clap(long)]
        source: Option<u64>,
        /// Optional. Set the transaction's memo field to the provided text.
        #[clap(long)]
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

    pub async fn exec<V: ViewClient, C: CustodyClient>(
        &self,
        opt: &Opt,
        fvk: &FullViewingKey,
        view: &mut V,
        custody: &mut C,
    ) -> Result<()> {
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

                let plan =
                    plan::send(&fvk, view, OsRng, &values, *fee, to, *from, memo.clone()).await?;

                let transaction = build_transaction(fvk, view, custody, OsRng, plan).await?;

                opt.submit_transaction(&transaction).await?;
            }
            TxCmd::Sweep => {
                todo!("port to new API");
                //sweep(opt, state).await?;
            }
        }
        Ok(())
    }
}

// TODO: port to new API
/*

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
                let mut plan = TransactionPlan {
                    chain_id: state
                        .chain_id()
                        .ok_or_else(|| anyhow!("missing chain_id"))?,
                    fee: Fee(0),
                    ..Default::default()
                };

                for note in group {
                    plan.actions.push(
                        SpendPlan::new(&mut OsRng, (**note).clone(), state.position(note).unwrap())
                            .into(),
                    );
                }
                plan.actions.push(
                    OutputPlan::new(
                        &mut OsRng,
                        Value {
                            amount: group.iter().map(|n| n.amount()).sum(),
                            asset_id: denom.id(),
                        },
                        addr,
                        MemoPlaintext::default(),
                    )
                    .into(),
                );

                transactions.push(state.build_transaction(OsRng, plan)?);
            }
        }
    }

    let num_sweeps = transactions.len();
    tracing::info!(num_sweeps, "submitting sweeps");
    for transaction in transactions {
        opt.submit_transaction_unconfirmed(&transaction).await?;
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

 */
