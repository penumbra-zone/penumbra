use anyhow::Result;
use comfy_table::{presets, Table};
use penumbra_asset::{asset::Cache, Value};
use penumbra_keys::FullViewingKey;
use penumbra_view::ViewClient;
#[derive(Debug, clap::Args)]
pub struct BalanceCmd {
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

        let rows: Vec<(Option<u32>, Value)> = if self.by_note {
            let notes = view
                .unspent_notes_by_account_and_asset(fvk.wallet_id())
                .await?;

            notes
                .iter()
                .flat_map(|(index, notes_by_asset)| {
                    // Include each note individually:
                    notes_by_asset.iter().flat_map(|(asset, notes)| {
                        notes
                            .iter()
                            .map(|record| (Some(*index), asset.value(record.note.amount())))
                    })
                })
                .collect()
        } else {
            let notes = view
                .unspent_notes_by_account_and_asset(fvk.wallet_id())
                .await?;

            notes
                .iter()
                .flat_map(|(index, notes_by_asset)| {
                    // Sum the notes for each asset:
                    notes_by_asset.iter().map(|(asset, notes)| {
                        let sum: u128 = notes
                            .iter()
                            .map(|record| u128::from(record.note.amount()))
                            .sum();
                        (Some(*index), asset.value(sum.into()))
                    })
                })
                // Exclude withdrawn LPNFTs.
                .filter(|(_, value)| match asset_cache.get(&value.asset_id) {
                    None => true,
                    Some(denom) => !denom.is_withdrawn_position_nft(),
                })
                .collect()
        };

        table.set_header(vec!["Account", "Amount"]);

        for row in rows.clone() {
            table.add_row(format_row(&row, &asset_cache));
        }

        println!("{table}");

        Ok(())
    }
}

fn format_row(row: &(Option<u32>, Value), asset_cache: &Cache) -> Vec<String> {
    let (index, value) = row;

    let mut string_row = Vec::with_capacity(2);

    if let Some(index) = index {
        string_row.push(format!("{}", index));

        string_row.push(value.format(asset_cache));
    }

    string_row
}
