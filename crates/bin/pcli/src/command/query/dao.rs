use anyhow::Result;
//use anyhow::{Context, Result};
/*
use futures::TryStreamExt;
//use penumbra_app::dao;
use penumbra_dao::component::state_key;

use penumbra_asset::{asset, Value};
use penumbra_num::Amount;
use penumbra_view::ViewClient;

use crate::{command::query::dao, App};
 */
use crate::App;

#[derive(Debug, clap::Subcommand)]
pub enum DaoCmd {
    /// Get the balance in the DAO, or the balance of a specific asset.
    Balance {
        /// Get only the balance of the specified asset.
        asset: Option<String>,
    },
}

impl DaoCmd {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            DaoCmd::Balance { asset } => self.print_balance(app, asset).await,
        }
    }

    pub async fn print_balance(&self, _app: &mut App, _asset: &Option<String>) -> Result<()> {
        unimplemented!("dao component needs an RPC defined");
        // below code is not usable outside of our own crates because
        // it does raw state key access
        /*
        let asset_id = asset.as_ref().map(|asset| {
            // Try to parse as an asset ID, then if it's not an asset ID, assume it's a unit name
            if let Ok(asset_id) = asset.parse() {
                asset_id
            } else {
                asset::REGISTRY.parse_unit(asset.as_str()).id()
            }
        });

        let mut client = app.specific_client().await?;
        let asset_cache = app.view().assets().await?;
        if let Some(asset_id) = asset_id {
            let key = state_key::balance_for_asset(asset_id);
            let amount: Amount = client
                .key_domain(&key)
                .await?
                .context(format!("No balance found for asset {asset_id}"))?;

            let value = Value { asset_id, amount };
            let value_str = value.format(&asset_cache);

            println!("{value_str}");
        } else {
            let prefix = dao::state_key::all_assets_balance();
            let results: Vec<_> = client.prefix_domain(prefix).await?.try_collect().await?;

            println!("DAO balance ({} unique assets):", results.len());

            for (key, amount) in results {
                // Parse every key/value pair into a Value
                let asset_id: asset::Id = key
                    .rsplit('/')
                    .next()
                    .expect("valid key")
                    .parse()
                    .expect("valid asset ID");
                let value = Value { asset_id, amount };
                let value_str = value.format(&asset_cache);
                println!("{value_str}");
            }
        };

        Ok(())
         */
    }
}
