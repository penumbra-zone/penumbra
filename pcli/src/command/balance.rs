use anyhow::Result;
use comfy_table::{presets, Table};
use penumbra_crypto::{keys::DiversifierIndex, FullViewingKey, Value};
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

            let rows: Vec<(DiversifierIndex, Value)> = if self.by_note {
                notes
                    .iter()
                    .flat_map(|(index, notes_by_asset)| {
                        // Include each note individually:
                        notes_by_asset.iter().flat_map(|(asset, notes)| {
                            notes
                                .iter()
                                .map(|record| (*index, asset.value(record.note.amount())))
                        })
                    })
                    .collect()
            } else {
                notes
                    .iter()
                    .flat_map(|(index, notes_by_asset)| {
                        // Sum the notes for each asset:
                        notes_by_asset.iter().map(|(asset, notes)| {
                            let sum = notes.iter().map(|record| record.note.amount()).sum();
                            (*index, asset.value(sum))
                        })
                    })
                    .collect()
            };

            table.set_header(vec!["Addr Index", "Amount"]);
            for (index, value) in rows {
                table.add_row(vec![
                    format!("{}", u128::from(index)),
                    value.try_format(&asset_cache).unwrap(),
                ]);
            }
        } else {
            let notes = view.unspent_notes_by_asset_and_address(fvk.hash()).await?;

            let rows: Vec<Value> = if self.by_note {
                notes
                    .iter()
                    .flat_map(|(asset, notes)| {
                        // Include each note individually:
                        notes.iter().flat_map(|(_index, notes)| {
                            notes.iter().map(|record| asset.value(record.note.amount()))
                        })
                    })
                    .collect()
            } else {
                notes
                    .iter()
                    .map(|(asset, notes)| {
                        // Sum the notes for each index:
                        let sum = notes
                            .values()
                            .flat_map(|records| records.iter().map(|record| record.note.amount()))
                            .sum();
                        asset.value(sum)
                    })
                    .collect()
            };
            table.set_header(vec!["Amount"]);
            for value in rows {
                table.add_row(vec![value.try_format(&asset_cache).unwrap()]);
            }
        }

        println!("{}", table);

        Ok(())
    }
}
