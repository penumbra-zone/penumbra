use std::collections::BTreeMap;

use anyhow::Result;
use comfy_table::{presets, Table};
use penumbra_crypto::{asset::Cache, keys::AddressIndex, FullViewingKey, Value};
use penumbra_view::ViewClient;
#[derive(Debug, clap::Args)]
pub struct BalanceCmd {
    /// If set, breaks down balances by address.
    #[clap(short, long)]
    pub by_address: bool,
    #[clap(long)]
    /// If set, prints the value of each note individually.
    pub by_note: bool,
}

impl BalanceCmd {
    pub fn offline(&self) -> bool {
        false
    }

    pub async fn exec<V: ViewClient>(&self, fvk: &FullViewingKey, view: &mut V) -> Result<()> {
        let asset_cache = view.assets().await?;

        // Initialize the table
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);

        let rows: Vec<(Option<AddressIndex>, Value, Option<u64>)> = if self.by_note {
            let notes = view.unspent_notes_by_address_and_asset(fvk.hash()).await?;
            let quarantined_notes = view
                .quarantined_notes_by_address_and_asset(fvk.hash())
                .await?;

            notes
                .iter()
                .flat_map(|(index, notes_by_asset)| {
                    // Include each note individually:
                    notes_by_asset.iter().flat_map(|(asset, notes)| {
                        notes
                            .iter()
                            .map(|record| (Some(*index), asset.value(record.note.amount()), None))
                    })
                })
                .chain(
                    quarantined_notes
                        .iter()
                        .flat_map(|(index, notes_by_asset)| {
                            // Include each note individually:
                            notes_by_asset.iter().flat_map(|(asset, notes)| {
                                notes.iter().map(|record| {
                                    (
                                        Some(*index),
                                        asset.value(record.note.amount()),
                                        Some(record.unbonding_epoch),
                                    )
                                })
                            })
                        }),
                )
                .collect()
        } else if self.by_address {
            let notes = view.unspent_notes_by_address_and_asset(fvk.hash()).await?;
            let quarantined_notes = view
                .quarantined_notes_by_address_and_asset(fvk.hash())
                .await?;

            // `Option<u64>` indicates the unbonding epoch, if any, for a quarantined note
            notes
                .iter()
                .flat_map(|(index, notes_by_asset)| {
                    // Sum the notes for each asset:
                    notes_by_asset.iter().map(|(asset, notes)| {
                        let sum: u64 = notes
                            .iter()
                            .map(|record| u64::from(record.note.amount()))
                            .sum();
                        (Some(*index), asset.value(sum.into()), None)
                    })
                })
                .chain(
                    quarantined_notes
                        .iter()
                        .flat_map(|(index, notes_by_asset)| {
                            // Sum the notes for each asset, separating them by unbonding epoch:
                            notes_by_asset.iter().flat_map(|(asset, records)| {
                                let mut sums_by_unbonding_epoch = BTreeMap::<u64, u64>::new();
                                for record in records {
                                    let unbonding_epoch = record.unbonding_epoch;
                                    *sums_by_unbonding_epoch.entry(unbonding_epoch).or_default() +=
                                        u64::from(record.note.amount());
                                }
                                sums_by_unbonding_epoch
                                    .into_iter()
                                    .map(|(unbonding_epoch, sum)| {
                                        (
                                            Some(*index),
                                            asset.value(sum.into()),
                                            Some(unbonding_epoch),
                                        )
                                    })
                            })
                        }),
                )
                .collect()
        } else {
            let notes = view.unspent_notes_by_asset_and_address(fvk.hash()).await?;
            let quarantined_notes = view
                .quarantined_notes_by_asset_and_address(fvk.hash())
                .await?;

            notes
                .iter()
                .map(|(asset, notes)| {
                    // Sum the notes for each index:
                    let sum: u64 = notes
                        .values()
                        .flat_map(|records| {
                            records.iter().map(|record| u64::from(record.note.amount()))
                        })
                        .sum();
                    (None, asset.value(sum.into()), None)
                })
                .chain(quarantined_notes.iter().flat_map(|(asset, records)| {
                    // Sum the notes for each index, separating them by unbonding epoch:
                    let mut sums_by_unbonding_epoch = BTreeMap::<u64, u64>::new();
                    for records in records.values() {
                        for record in records {
                            let unbonding_epoch = record.unbonding_epoch;
                            *sums_by_unbonding_epoch.entry(unbonding_epoch).or_default() +=
                                u64::from(record.note.amount());
                        }
                    }
                    sums_by_unbonding_epoch
                        .into_iter()
                        .map(|(unbonding_epoch, sum)| {
                            (None, asset.value(sum.into()), Some(unbonding_epoch))
                        })
                }))
                .collect()
        };

        let (indexed_rows, ephemeral_rows) = combine_ephemeral(rows, self.by_note);

        if self.by_address {
            table.set_header(vec!["Addr Index", "Amount"]);
        } else {
            table.set_header(vec!["Amount"]);
        }

        for row in indexed_rows.iter().chain(ephemeral_rows.iter()) {
            table.add_row(format_row(row, self.by_address, &asset_cache));
        }

        println!("{}", table);

        Ok(())
    }
}

fn format_row(
    row: &(Option<AddressIndex>, Value, Option<u64>),
    by_address: bool,
    asset_cache: &Cache,
) -> Vec<String> {
    let (index, value, quarantined) = row;

    let mut string_row = Vec::with_capacity(2);

    if by_address {
        let index = u128::from(index.expect("--by-address specified, but no index set for note"));
        let index_text = if index < u64::MAX as u128 {
            format!("{}", index)
        } else {
            "Ephemeral".to_string()
        };

        string_row.push(index_text)
    }
    string_row.push(format!(
        "{}{}",
        value.format(&asset_cache),
        if let Some(unbonding_epoch) = quarantined {
            format!(" (unbonding until epoch {})", unbonding_epoch)
        } else {
            "".to_string()
        }
    ));

    string_row
}

/// Split the rows into (indexed, ephemeral) pair with all of the ephemeral notes
/// combined by asset. The AddressIndex is left in to signal the ephemerality to
/// the table parsing. This should be changed when well typed, JSON output is supported
fn combine_ephemeral(
    rows: Vec<(Option<AddressIndex>, Value, Option<u64>)>,
    by_note: bool,
) -> (
    Vec<(Option<AddressIndex>, Value, Option<u64>)>,
    Vec<(Option<AddressIndex>, Value, Option<u64>)>,
) {
    if by_note {
        return (rows, Vec::new());
    }

    // get all ephemeral rows
    let (mut ephemeral_notes, indexed_rows): (Vec<_>, Vec<_>) =
        rows.into_iter().partition(|(index, _, _)| {
            if let Some(index) = index {
                u128::from(*index) > u64::MAX as u128
            } else {
                false
            }
        });

    let ephemeral_rows = if ephemeral_notes.len() <= 1 {
        // Nothing to combine
        ephemeral_notes
    } else {
        // Simulate a `SELECT SUM(note.amount) GROUP BY is_ephemeral` by sorting
        // the notes by asset, and the summing rows together until the asset_id changes
        ephemeral_notes.sort_by(|row1, row2| row1.1.asset_id.cmp(&row2.1.asset_id));
        let mut new_ephemeral_notes = vec![];
        let mut cur_row = ephemeral_notes[0];
        for row in ephemeral_notes.iter().skip(1) {
            if cur_row.1.asset_id == row.1.asset_id {
                cur_row.1.amount = cur_row.1.amount + row.1.amount;
            } else {
                new_ephemeral_notes.push(cur_row);
                cur_row = *row;
            }
        }
        new_ephemeral_notes
    };
    (indexed_rows, ephemeral_rows)
}
