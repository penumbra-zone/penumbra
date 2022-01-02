use anyhow::Result;
use comfy_table::{presets, Table};
use penumbra_crypto::asset::{self, Denom};
use penumbra_wallet::{ClientState, UnspentNote};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct BalanceCmd {
    /// If set, breaks down balances by address.
    #[structopt(short, long)]
    pub by_address: bool,
    #[structopt(long)]
    /// If set, does not attempt to synchronize the wallet before printing the balance.
    pub offline: bool,
}

// Format a tally of notes as three strings: total, unspent, and pending spend. This
// assumes that the notes are all of the same denomination, and it is called below only
// in the places where they are.
fn tally_format_notes<'a>(
    denom: &Denom,
    cache: &asset::Cache,
    notes: impl IntoIterator<Item = UnspentNote<'a>>,
) -> (String, String, String, String) {
    // Tally each of the kinds of note:
    let mut unspent = 0;
    let mut pending = 0;
    let mut pending_change = 0;

    for note in notes {
        *match note {
            UnspentNote::Ready(_) => &mut unspent,
            UnspentNote::PendingSpend(_) => &mut pending,
            UnspentNote::PendingChange(_) => &mut pending_change,
        } += note.as_ref().amount();
    }

    // The amount spent is the difference between pending and pending change:
    let pending_spend = pending - pending_change;

    let pending_change = denom.value(pending_change);
    let pending_spend = denom.value(pending_spend);

    let pending_change_string = if pending_change.amount > 0 {
        format!("+{} (change)", pending_change.try_format(cache).unwrap())
    } else {
        "".to_string()
    };

    let pending_spend_string = if pending_spend.amount > 0 {
        format!("-{} (spend)", pending_spend.try_format(cache).unwrap())
    } else {
        "".to_string()
    };

    // The total amount, disregarding pending transactions:
    let total = denom.value(pending_change.amount + unspent);
    // The amount available to spend:
    let available = denom.value(unspent);

    (
        total.try_format(cache).unwrap(),
        available.try_format(cache).unwrap(),
        pending_change_string,
        pending_spend_string,
    )
}

impl BalanceCmd {
    pub fn needs_sync(&self) -> bool {
        !self.offline
    }

    pub fn exec(&self, state: &ClientState) -> Result<()> {
        // Initialize the table
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);
        let mut print_pending_column = false; // This will become true if there are any pending transactions
        let mut headers;

        if self.by_address {
            for (address_id, by_denom) in state.unspent_notes_by_address_and_denom().into_iter() {
                let (mut label, _) = state.wallet().address_by_index(address_id as usize)?;
                for (denom, notes) in by_denom.into_iter() {
                    let (total, available, pending_change, pending_spend) =
                        tally_format_notes(&denom, state.asset_cache(), notes);
                    let mut row = vec![label.clone(), total];
                    if !pending_change.is_empty() || !pending_spend.is_empty() {
                        print_pending_column = true;
                        row.push(available);
                        row.push(pending_change);
                        row.push(pending_spend);
                    }
                    table.add_row(row);

                    // Only display the label on the first row
                    label = String::default();
                }
            }

            // Set up headers for the table (a "Pending" column will be added if there are any
            // pending transactions)
            headers = vec!["Address", "Total"];
        } else {
            for (denom, by_address) in state.unspent_notes_by_denom_and_address().into_iter() {
                let (total, available, pending_change, pending_spend) = tally_format_notes(
                    &denom,
                    state.asset_cache(),
                    by_address.into_values().flatten(),
                );
                let mut row = vec![total];
                if !pending_change.is_empty() || !pending_spend.is_empty() {
                    print_pending_column = true;
                    row.push(available);
                    row.push(pending_change);
                    row.push(pending_spend);
                }
                table.add_row(row);
            }

            // Set up headers for the table (a "Pending" column will be added if there are any
            // pending transactions)
            headers = vec!["Total"];
        }

        // Add an "Available" and "Pending" column if there are any pending transactions
        if print_pending_column {
            headers.push("Available");
            headers.push("Pending");
        }
        table.set_header(headers);
        println!("{}", table);

        Ok(())
    }
}
