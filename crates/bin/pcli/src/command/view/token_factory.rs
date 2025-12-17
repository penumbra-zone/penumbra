//! View commands for token factory assets.

use anyhow::Result;
use comfy_table::{presets, Table};
use penumbra_sdk_view::ViewClient;

/// View your token factory assets (created tokens and mint capabilities).
#[derive(Debug, clap::Parser)]
pub struct TokenFactoryCmd {
    /// Show only mint capabilities (not regular factory tokens).
    #[clap(long)]
    mint_caps_only: bool,
}

impl TokenFactoryCmd {
    pub fn offline(&self) -> bool {
        false
    }

    pub async fn exec(&self, view: &mut impl ViewClient) -> Result<()> {
        let asset_cache = view.assets().await?;
        let notes = view.unspent_notes_by_account_and_asset().await?;

        let mut factory_tokens: Vec<(String, String, String)> = Vec::new();
        let mut mint_caps: Vec<(String, String, u64)> = Vec::new();

        for (_account, notes_by_asset) in notes {
            for (asset_id, notes) in notes_by_asset {
                let total: u128 = notes.iter().map(|n| n.note.amount().value()).sum();
                if total == 0 {
                    continue;
                }

                // look up denom
                let denom = asset_cache
                    .get(&asset_id)
                    .map(|m| m.base_denom().denom.clone())
                    .unwrap_or_else(|| format!("{:?}", asset_id));

                // check if it's a factory token or mint cap
                if denom.starts_with("factory/") {
                    if !self.mint_caps_only {
                        factory_tokens.push((
                            denom.clone(),
                            total.to_string(),
                            format!("{:?}", asset_id),
                        ));
                    }
                } else if denom.starts_with("factory_mint_") {
                    // parse seq number from denom: factory_mint_{seq}_{token_id}
                    let parts: Vec<&str> = denom.split('_').collect();
                    let seq = parts.get(2).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                    let token_id = parts.get(3).map(|s| s.to_string()).unwrap_or_default();
                    mint_caps.push((token_id, denom, seq));
                }
            }
        }

        if !self.mint_caps_only && !factory_tokens.is_empty() {
            println!("\nFactory Tokens:");
            let mut table = Table::new();
            table.load_preset(presets::NOTHING);
            table.set_header(vec!["Denom", "Balance", "Asset ID"]);
            for (denom, balance, asset_id) in &factory_tokens {
                table.add_row(vec![denom, balance, asset_id]);
            }
            println!("{table}");
        }

        if !mint_caps.is_empty() {
            println!("\nMint Capabilities:");
            let mut table = Table::new();
            table.load_preset(presets::NOTHING);
            table.set_header(vec!["Token ID", "Sequence", "Denom"]);
            for (token_id, denom, seq) in &mint_caps {
                table.add_row(vec![token_id, &seq.to_string(), denom]);
            }
            println!("{table}");
            println!("\nUse `pcli tx token-factory mint --token-id <ID> --seq <SEQ> --amount <AMT>` to mint more tokens.");
        }

        if factory_tokens.is_empty() && mint_caps.is_empty() {
            println!("No token factory assets found.");
            println!("\nCreate a token with: pcli tx token-factory create --name MYTOKEN --supply 1000000");
            println!("Or launch with bonding curve: pcli tx token-factory launch --name MYTOKEN --supply 1000000 --quote-asset penumbra --start-price 0.001 --end-price 0.1");
        }

        Ok(())
    }
}
