use std::collections::BTreeMap;

use anyhow::Result;
use comfy_table::{presets, Table};
use penumbra_crypto::{keys::AddressIndex, FullViewingKey, Value};
use penumbra_view::ViewClient;
#[derive(Debug, clap::Args)]
pub struct BalanceCmd {
    /// If set, breaks down balances by address.
    #[clap(short, long)]
    pub by_address: bool,
    #[clap(long)]
    /// If set, does not attempt to synchronize the wallet before printing the balance.
    pub offline: bool,
    #[clap(long)]
    /// If set, prints the value of each note individually.
    pub by_note: bool,
}

impl BalanceCmd {
    pub fn needs_sync(&self) -> bool {
        !self.offline
    }

    pub async fn exec<V: ViewClient>(&self, fvk: &FullViewingKey, view: &mut V) -> Result<()> {
        let asset_cache = view.assets().await?;

        // Initialize the table
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);

        if self.by_address {
            let notes = view.unspent_notes_by_address_and_asset(fvk.hash()).await?;
            let quarantined_notes = view
                .quarantined_notes_by_address_and_asset(fvk.hash())
                .await?;

            // `Option<u64>` indicates the unbonding epoch, if any, for a quarantined note
            let rows: Vec<(AddressIndex, Value, Option<u64>)> = if self.by_note {
                notes
                    .iter()
                    .flat_map(|(index, notes_by_asset)| {
                        // Include each note individually:
                        notes_by_asset.iter().flat_map(|(asset, notes)| {
                            notes
                                .iter()
                                .map(|record| (*index, asset.value(record.note.amount()), None))
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
                                            *index,
                                            asset.value(record.note.amount()),
                                            Some(record.unbonding_epoch),
                                        )
                                    })
                                })
                            }),
                    )
                    .collect()
            } else {
                notes
                    .iter()
                    .flat_map(|(index, notes_by_asset)| {
                        // Sum the notes for each asset:
                        notes_by_asset.iter().map(|(asset, notes)| {
                            let sum: u64 = notes
                                .iter()
                                .map(|record| u64::from(record.note.amount()))
                                .sum();
                            (*index, asset.value(sum.into()), None)
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
                                        *sums_by_unbonding_epoch
                                            .entry(unbonding_epoch)
                                            .or_default() += u64::from(record.note.amount());
                                    }
                                    sums_by_unbonding_epoch.into_iter().map(
                                        |(unbonding_epoch, sum)| {
                                            (*index, asset.value(sum.into()), Some(unbonding_epoch))
                                        },
                                    )
                                })
                            }),
                    )
                    .collect()
            };

            table.set_header(vec!["Addr Index", "Amount"]);
            for (index, value, quarantined) in rows {
                table.add_row(vec![
                    format!("{}", u128::from(index)),
                    format!(
                        "{}{}",
                        value.format(&asset_cache),
                        if let Some(unbonding_epoch) = quarantined {
                            format!(" (unbonding until epoch {})", unbonding_epoch)
                        } else {
                            "".to_string()
                        }
                    ),
                ]);
            }
        } else {
            let notes = view.unspent_notes_by_asset_and_address(fvk.hash()).await?;
            let quarantined_notes = view
                .quarantined_notes_by_asset_and_address(fvk.hash())
                .await?;

            let rows: Vec<(Value, Option<u64>)> = if self.by_note {
                notes
                    .iter()
                    .flat_map(|(asset, notes)| {
                        // Include each note individually:
                        notes.iter().flat_map(|(_index, notes)| {
                            notes
                                .iter()
                                .map(|record| (asset.value(record.note.amount()), None))
                        })
                    })
                    .chain(quarantined_notes.iter().flat_map(|(asset, notes)| {
                        // Include each note individually:
                        notes.iter().flat_map(|(_index, notes)| {
                            notes.iter().map(|record| {
                                (
                                    asset.value(record.note.amount()),
                                    Some(record.unbonding_epoch),
                                )
                            })
                        })
                    }))
                    .collect()
            } else {
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
                        (asset.value(sum.into()), None)
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
                                (asset.value(sum.into()), Some(unbonding_epoch))
                            })
                    }))
                    .collect()
            };
            table.set_header(vec!["Amount"]);
            for (value, quarantined) in rows {
                table.add_row(vec![format!(
                    "{}{}",
                    value.format(&asset_cache),
                    if let Some(unbonding_epoch) = quarantined {
                        format!(" (unbonding until epoch {})", unbonding_epoch)
                    } else {
                        "".to_string()
                    }
                )]);
            }
        }

        println!("{}", table);

        Ok(())
    }
}
