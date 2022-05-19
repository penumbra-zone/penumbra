use std::{fs::File, io::BufReader, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Context as _, Result};
use comfy_table::Table;
use directories::ProjectDirs;
use penumbra_crypto::keys::{SeedPhrase, SpendSeed};
use penumbra_wallet::{ClientState, Wallet};
use rand_core::OsRng;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use structopt::StructOpt;

use crate::ClientStateFile;

#[derive(Debug, StructOpt)]
pub enum ChainCmd {
    /// Display chain parameters.
    Params,
    /// Display information about the current chain state.
    Info {
        /// If true, will also display chain parameters.
        #[structopt(short, long)]
        verbose: bool,
    },
}

impl ChainCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn needs_sync(&self) -> bool {
        // Always true, though strictly not necessary until chain parameters are
        // determined by consensus.
        true
    }

    pub async fn exec(&self, state: &ClientStateFile) -> Result<()> {
        match self {
            ChainCmd::Params => {
                let params = match state.chain_params() {
                    Some(params) => params,
                    None => {
                        // This indicates that the sync failed.
                        return Err(anyhow!("chain parameters not available"));
                    }
                };
                println!("Chain Parameters: ");
                let mut table = Table::new();
                table
                    .set_header(vec!["Parameter", "Value"])
                    .add_row(vec!["Chain ID", &params.chain_id])
                    .add_row(vec![
                        "Epoch Duration",
                        &format!("{}", params.epoch_duration),
                    ])
                    .add_row(vec![
                        "Unbonding Epochs",
                        &format!("{}", params.unbonding_epochs),
                    ])
                    .add_row(vec![
                        "Active Validator Limit",
                        &format!("{}", params.active_validator_limit),
                    ])
                    .add_row(vec![
                        "Base Reward Rate (bps of bps)",
                        &format!("{}", params.base_reward_rate),
                    ])
                    .add_row(vec![
                        "Slashing Penalty (Misbehavior) (bps)",
                        &format!("{}", params.slashing_penalty_misbehavior_bps),
                    ])
                    .add_row(vec![
                        "Slashing Penalty (Downtime) (bps)",
                        &format!("{}", params.slashing_penalty_downtime_bps),
                    ])
                    .add_row(vec![
                        "Signed Blocks Window (blocks)",
                        &format!("{}", params.signed_blocks_window_len),
                    ])
                    .add_row(vec![
                        "Missed Blocks Max",
                        &format!("{}", params.missed_blocks_maximum),
                    ])
                    .add_row(vec!["IBC Enabled", &format!("{}", params.ibc_enabled)])
                    .add_row(vec![
                        "Inbound ICS-20 Enabled",
                        &format!("{}", params.inbound_ics20_transfers_enabled),
                    ])
                    .add_row(vec![
                        "Outbound ICS-20 Enabled",
                        &format!("{}", params.outbound_ics20_transfers_enabled),
                    ]);

                println!("{}", table);
                // println!("  Chain ID: {}", params.chain_id);
                // println!("  Epoch Duration: {}", params.epoch_duration);
                // println!("  Unbonding Epochs: {}", params.unbonding_epochs);
                // println!("  Active Validator Limit: {}", params.unbonding_epochs);
                // tracing::debug!(?params, "exec");
            }
            ChainCmd::Info { verbose } => {}
        };

        Ok(())
    }
}
