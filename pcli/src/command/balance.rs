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
    #[structopt(long)]
    /// If set, prints the value of each note individually.
    pub by_note: bool,
}

/// Result of formatting the tally for a particular asset.
struct FormattedTally {
    total: String,
    available: String,
    submitted_spend: String,
    submitted_change: String,
    unbonding: String,
}

// Format a tally of notes as a set of strings.
///
/// This assumes that the notes are all of the same denomination, and it is called below only
// in the places where they are.
fn tally_format_notes<'a>(
    denom: &'a Denom,
    cache: &'a asset::Cache,
    notes_groups: impl IntoIterator<Item = impl IntoIterator<Item = UnspentNote<'a>> + 'a> + 'a,
) -> impl IntoIterator<Item = FormattedTally> + 'a {
    notes_groups.into_iter().map(|notes| {
        // Tally each of the kinds of note:
        let mut unspent = 0;
        let mut submitted_spend = 0;
        let mut submitted_change = 0;
        let mut unbonding = 0;

        for note in notes {
            *match note {
                UnspentNote::Ready(_) => &mut unspent,
                UnspentNote::SubmittedSpend(_) => &mut submitted_spend,
                UnspentNote::SubmittedChange(_) => &mut submitted_change,
                UnspentNote::Quarantined { .. } => &mut unbonding,
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

        let unbonding_string = if unbonding > 0 {
            format!("+{unbonding} (unbonding)")
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
            unbonding: unbonding_string,
        }
    })
}

impl BalanceCmd {
    pub fn needs_sync(&self) -> bool {
        !self.offline
    }

    pub fn exec(&self, state: &ClientState) -> Result<()> {
        // Initialize the table
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);
        let mut headers;

        if self.by_address {
            for (address_id, by_denom) in state.unspent_notes_by_address_and_denom().into_iter() {
                let (mut label, _) = state.wallet().address_by_index(address_id as usize)?;
                for (denom, notes) in by_denom.into_iter() {
                    let notes_groups = if self.by_note {
                        notes.into_iter().map(|n| vec![n]).collect()
                    } else {
                        vec![notes]
                    };
                    let tallies = tally_format_notes(&denom, state.asset_cache(), notes_groups);
                    for tally in tallies {
                        let mut row = vec![label.clone(), tally.total];
                        row.push(tally.available);
                        row.push(tally.submitted_change);
                        row.push(tally.submitted_spend);
                        row.push(tally.unbonding);
                        table.add_row(row);

                        // Only display the label on the first row
                        label = String::default();
                    }
                }
            }

            // Set up headers for the table (a "Submitted" column will be added if there are any
            // submitted transactions)
            headers = vec!["Address", "Total"];
        } else {
            for (denom, by_address) in state.unspent_notes_by_denom_and_address().into_iter() {
                let notes = by_address.into_values().flatten();

                let notes_groups = if self.by_note {
                    notes.map(|n| vec![n]).collect()
                } else {
                    vec![notes.collect()]
                };

                let tallies = tally_format_notes(&denom, state.asset_cache(), notes_groups);

                for tally in tallies {
                    let mut row = vec![tally.total];
                    row.push(tally.available);
                    row.push(tally.submitted_change);
                    row.push(tally.submitted_spend);
                    row.push(tally.unbonding);
                    table.add_row(row);
                }
            }

            // Set up headers for the table (a "Submitted" column will be added if there are any
            // submitted transactions)
            headers = vec!["Total"];
        }

        // Set up column headers and print table
        headers.push("Available");
        headers.push("Submitted");
        headers.push("");
        headers.push("Unbonding");
        table.set_header(headers);
        println!("{}", table);

        Ok(())
    }
}
