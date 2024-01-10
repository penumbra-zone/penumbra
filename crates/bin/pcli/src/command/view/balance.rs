use anyhow::Result;
use comfy_table::{presets, Table};

use penumbra_keys::FullViewingKey;
use penumbra_sct::CommitmentSource;
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

    pub async fn exec<V: ViewClient>(&self, _fvk: &FullViewingKey, view: &mut V) -> Result<()> {
        let asset_cache = view.assets().await?;

        // Initialize the table
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);

        let notes = view.unspent_notes_by_account_and_asset().await?;

        if self.by_note {
            table.set_header(vec!["Account", "Value", "Source"]);

            let rows = notes
                .iter()
                .flat_map(|(index, notes_by_asset)| {
                    // Include each note individually:
                    notes_by_asset.iter().flat_map(|(asset, notes)| {
                        notes.iter().map(|record| {
                            (
                                *index,
                                asset.value(record.note.amount()),
                                record.source.clone(),
                            )
                        })
                    })
                })
                // Exclude withdrawn LPNFTs.
                .filter(|(_, value, _)| match asset_cache.get(&value.asset_id) {
                    None => true,
                    Some(denom) => !denom.is_withdrawn_position_nft(),
                });

            for (index, value, source) in rows {
                table.add_row(vec![
                    format!("# {}", index),
                    value.format(&asset_cache),
                    format_source(&source),
                ]);
            }

            println!("{table}");

            return Ok(());
        } else {
            table.set_header(vec!["Account", "Amount"]);

            let rows = notes
                .iter()
                .flat_map(|(index, notes_by_asset)| {
                    // Sum the notes for each asset:
                    notes_by_asset.iter().map(|(asset, notes)| {
                        let sum: u128 = notes
                            .iter()
                            .map(|record| u128::from(record.note.amount()))
                            .sum();
                        (*index, asset.value(sum.into()))
                    })
                })
                // Exclude withdrawn LPNFTs.
                .filter(|(_, value)| match asset_cache.get(&value.asset_id) {
                    None => true,
                    Some(denom) => !denom.is_withdrawn_position_nft(),
                });

            for (index, value) in rows {
                table.add_row(vec![format!("# {}", index), value.format(&asset_cache)]);
            }

            println!("{table}");

            return Ok(());
        }
    }
}

fn format_source(source: &CommitmentSource) -> String {
    match source {
        CommitmentSource::Genesis => "Genesis".to_owned(),
        CommitmentSource::Transaction { id: None } => "Tx (Unknown)".to_owned(),
        CommitmentSource::Transaction { id: Some(id) } => format!("Tx {}", hex::encode(&id[..])),
        CommitmentSource::FundingStreamReward { epoch_index } => {
            format!("Funding Stream (Epoch {})", epoch_index)
        }
        CommitmentSource::CommunityPoolOutput => format!("CommunityPoolOutput"),
        CommitmentSource::Ics20Transfer {
            packet_seq,
            channel_id,
            sender,
        } => format!(
            "ICS20 packet {} via {} from {}",
            packet_seq, channel_id, sender
        ),
    }
}
