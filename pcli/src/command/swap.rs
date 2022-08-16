use std::collections::BTreeMap;

use anyhow::{anyhow, Context, Result};
use comfy_table::{presets, Table};
use futures::stream::TryStreamExt;
use penumbra_component::stake::{rate::RateData, validator};
use penumbra_crypto::{DelegationToken, IdentityKey, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_proto::client::oblivious::ValidatorInfoRequest;
use penumbra_view::ViewClient;
use penumbra_wallet::plan;
use rand_core::OsRng;

use crate::App;

#[derive(Debug, clap::Subcommand)]
pub enum SwapCmd {
    /// Submit a new Swap to the chain which will burn input assets and allow a future SwapClaim for the given Swap NFT.
    /// Only the first asset has an input amount specified, as in typical usage, the second asset is always
    /// the asset that the submitter wants to swap the first for.
    Swap {
        /// Asset ID of the first input asset.
        asset_1_id: String,
        /// The amount of asset 1 to burn as part of the swap.
        asset_1_input_amount: String,
        /// Asset ID of the second input asset.
        asset_2_id: String,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Optional. Only spend funds originally received by the given address index.
        #[clap(long)]
        source: Option<u64>,
    },
    /// Submit a SwapClaim to the chain which will obtain the output amounts for a given Swap NFT.
    SwapClaim {
        /// The asset ID of the swap NFT to be claimed.
        swap_nft_asset_id: String,
    },
    /// Display this wallet's swaps and swap claims.
    Show,
}

impl SwapCmd {
    pub fn needs_sync(&self) -> bool {
        true
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            SwapCmd::Swap {
                asset_1_id,
                asset_1_input_amount,
                asset_2_id,
                fee,
                source,
            } => {
                println!("Sorry, this command is not yet implemented");
            }
            SwapCmd::SwapClaim { swap_nft_asset_id } => {
                println!("Sorry, this command is not yet implemented");
            }
            SwapCmd::Show => {
                println!("Sorry, this command is not yet implemented");
            }
        }

        Ok(())
    }
}
