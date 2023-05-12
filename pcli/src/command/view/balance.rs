use anyhow::Result;
use comfy_table::{presets, Table};
use penumbra_crypto::{asset::Cache, keys::AddressIndex, FullViewingKey, Value};
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

        let rows: Vec<(Option<AddressIndex>, Value)> = if self.by_note {
            let notes = view
                .unspent_notes_by_address_and_asset(fvk.account_group_id())
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
                .unspent_notes_by_address_and_asset(fvk.account_group_id())
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

        let (indexed_rows, ephemeral_rows) = combine_ephemeral(rows, self.by_note);

        table.set_header(vec!["Account", "Amount"]);

        for row in indexed_rows.iter().chain(ephemeral_rows.iter()) {
            table.add_row(format_row(row, &asset_cache));
        }

        println!("{table}");

        Ok(())
    }
}

fn format_row(row: &(Option<AddressIndex>, Value), asset_cache: &Cache) -> Vec<String> {
    let (index, value) = row;

    let mut string_row = Vec::with_capacity(2);

    if let Some(index) = index {
        string_row.push(format!("{}", index.account));
    }
    string_row.push(value.format(asset_cache));

    string_row
}

/// Split the rows into (indexed, ephemeral) pair with all of the ephemeral notes
/// combined by asset. The AddressIndex is left in to signal the ephemerality to
/// the table parsing. This should be changed when well typed, JSON output is supported
#[allow(clippy::type_complexity)]
fn combine_ephemeral(
    rows: Vec<(Option<AddressIndex>, Value)>,
    by_note: bool,
) -> (
    Vec<(Option<AddressIndex>, Value)>,
    Vec<(Option<AddressIndex>, Value)>,
) {
    if by_note {
        return (rows, Vec::new());
    }

    // get all ephemeral rows
    let (mut ephemeral_notes, indexed_rows): (Vec<_>, Vec<_>) =
        rows.into_iter().partition(|(index, _)| {
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
        // Make sure to get the currently-in-progress row
        new_ephemeral_notes.push(cur_row);
        new_ephemeral_notes
    };
    (indexed_rows, ephemeral_rows)
}
