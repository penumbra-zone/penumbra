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

/// Result of formatting the tally for a particular asset.
struct FormattedTally {
    total: String,
    available: String,
    submitted_spend: String,
    submitted_change: String,
}

// Format a tally of notes as a set of strings.
///
/// This assumes that the notes are all of the same denomination, and it is called below only
// in the places where they are.
fn tally_format_notes<'a>(
    denom: &Denom,
    cache: &asset::Cache,
    notes: impl IntoIterator<Item = UnspentNote<'a>>,
) -> FormattedTally {
    // Tally each of the kinds of note:
    let mut unspent = 0;
    let mut submitted_spend = 0;
    let mut submitted_change = 0;

    for note in notes {
        *match note {
            UnspentNote::Ready(_) => &mut unspent,
            UnspentNote::SubmittedSpend(_) => &mut submitted_spend,
            UnspentNote::SubmittedChange(_) => &mut submitted_change,
        } += note.as_ref().amount();
    }

    // The amount spent is the difference between submitted spend and submitted change:
    let net_submitted_spend = submitted_spend - submitted_change;

    // Convert the results to denominations:
    let submitted_change = denom.value(submitted_change);
    let net_submitted_spend = denom.value(net_submitted_spend);

    let submitted_change_string = if submitted_change.amount > 0 {
        format!("+{} (change)", submitted_change.try_format(cache).unwrap())
    } else {
        "".to_string()
    };

    let submitted_spend_string = if net_submitted_spend.amount > 0 {
        format!(
            "-{} (spend)",
            net_submitted_spend.try_format(cache).unwrap()
        )
    } else {
        "".to_string()
    };

    // The total amount, disregarding submitted transactions:
    let total = denom.value(submitted_change.amount + unspent);

    // The amount available to spend:
    let available = denom.value(unspent);

    FormattedTally {
        total: total.try_format(cache).unwrap(),
        available: available.try_format(cache).unwrap(),
        submitted_change: submitted_change_string,
        submitted_spend: submitted_spend_string,
    }
}

impl BalanceCmd {
    pub fn needs_sync(&self) -> bool {
        !self.offline
    }

    pub fn exec(&self, state: &ClientState) -> Result<()> {
        // Initialize the table
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);
        let mut print_submitted_column = false; // This will become true if there are any submitted transactions
        let mut headers;

        if self.by_address {
            for (address_id, by_denom) in state.unspent_notes_by_address_and_denom().into_iter() {
                let (mut label, _) = state.wallet().address_by_index(address_id as usize)?;
                for (denom, notes) in by_denom.into_iter() {
                    let tally = tally_format_notes(&denom, state.asset_cache(), notes);
                    let mut row = vec![label.clone(), tally.total];
                    if !tally.submitted_change.is_empty() || !tally.submitted_spend.is_empty() {
                        print_submitted_column = true;
                        row.push(tally.available);
                        row.push(tally.submitted_change);
                        row.push(tally.submitted_spend);
                    }
                    table.add_row(row);

                    // Only display the label on the first row
                    label = String::default();
                }
            }

            // Set up headers for the table (a "Submitted" column will be added if there are any
            // submitted transactions)
            headers = vec!["Address", "Total"];
        } else {
            for (denom, by_address) in state.unspent_notes_by_denom_and_address().into_iter() {
                let tally = tally_format_notes(
                    &denom,
                    state.asset_cache(),
                    by_address.into_values().flatten(),
                );
                let mut row = vec![tally.total];
                if !tally.submitted_change.is_empty() || !tally.submitted_spend.is_empty() {
                    print_submitted_column = true;
                    row.push(tally.available);
                    row.push(tally.submitted_change);
                    row.push(tally.submitted_spend);
                }
                table.add_row(row);
            }

            // Set up headers for the table (a "Submitted" column will be added if there are any
            // submitted transactions)
            headers = vec!["Total"];
        }

        // Add an "Available" and "Submitted" column if there are any submitted transactions
        if print_submitted_column {
            headers.push("Available");
            headers.push("Submitted");
        }
        table.set_header(headers);
        println!("{}", table);

        Ok(())
    }
}
