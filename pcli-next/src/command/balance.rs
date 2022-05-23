use anyhow::Result;
use comfy_table::{presets, Table};
use penumbra_crypto::{keys::DiversifierIndex, FullViewingKey, Value};
use penumbra_view::ViewClient;
use structopt::StructOpt;

// TODO

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

impl BalanceCmd {
    pub fn needs_sync(&self) -> bool {
        !self.offline
    }

    pub async fn exec<V: ViewClient + Clone>(
        &self,
        fvk: &FullViewingKey,
        mut view: V,
    ) -> Result<()> {
        let asset_cache = view.assets().await?;

        // Initialize the table
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);

        if self.by_address {
            let notes = view
                .unspent_notes_by_address_and_denom(fvk.hash(), &asset_cache)
                .await?;

            let rows: Vec<(DiversifierIndex, Value)> = if self.by_note {
                notes
                    .iter()
                    .flat_map(|(index, notes_by_denom)| {
                        // Include each note individually:
                        notes_by_denom.iter().flat_map(|(denom, notes)| {
                            notes
                                .iter()
                                .map(|record| (*index, denom.value(record.note.amount())))
                        })
                    })
                    .collect()
            } else {
                notes
                    .iter()
                    .flat_map(|(index, notes_by_denom)| {
                        // Sum the notes for each denom:
                        notes_by_denom.iter().map(|(denom, notes)| {
                            let sum = notes.iter().map(|record| record.note.amount()).sum();
                            (*index, denom.value(sum))
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
            let notes = view
                .unspent_notes_by_denom_and_address(fvk.hash(), &asset_cache)
                .await?;

            let rows: Vec<Value> = if self.by_note {
                notes
                    .iter()
                    .flat_map(|(denom, notes)| {
                        // Include each note individually:
                        notes.iter().flat_map(|(_index, notes)| {
                            notes.iter().map(|record| denom.value(record.note.amount()))
                        })
                    })
                    .collect()
            } else {
                notes
                    .iter()
                    .map(|(denom, notes)| {
                        // Sum the notes for each index:
                        let sum = notes
                            .values()
                            .flat_map(|records| records.iter().map(|record| record.note.amount()))
                            .sum();
                        denom.value(sum)
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
