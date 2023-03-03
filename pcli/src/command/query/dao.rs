use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use futures::{StreamExt, TryStreamExt};
use penumbra_component::dao;
use penumbra_crypto::{
    asset::{self, Denom},
    Amount, Value,
};
use penumbra_view::ViewClient;

use crate::App;

#[derive(Debug, clap::Subcommand)]
pub enum DaoCmd {
    Balance { asset: Option<String> },
}

impl DaoCmd {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            DaoCmd::Balance { asset } => self.print_balance(app, asset).await,
        }
    }

    pub async fn print_balance(&self, app: &mut App, asset: &Option<String>) -> Result<()> {
        let denom_by_asset = app.view().assets().await?;
        let asset_by_denom = denom_by_asset
            .iter()
            .map(|(asset, denom)| (denom, asset))
            .collect::<BTreeMap<_, _>>();

        let asset_id = if let Some(asset) = asset {
            // Try to parse as a denomination, then as an asset ID, and fail if neither works
            if let Some(asset_id) = Denom::try_from(asset.as_str())
                .ok()
                .and_then(|denom| asset_by_denom.get(&denom))
            {
                Some(**asset_id)
            } else if let Ok(asset_id) = asset.parse() {
                Some(asset_id)
            } else {
                anyhow::bail!("unknown asset: {}", asset);
            }
        } else {
            None
        };

        let mut client = app.specific_client().await?;
        if let Some(asset_id) = asset_id {
            let key = dao::state_key::balance_for_asset(asset_id);
            let amount: Amount = client.key_domain(&key).await?;
            let value = Value { asset_id, amount };
            let string = value.format(&denom_by_asset);
            println!("{string}");
        } else {
            anyhow::bail!("printing the entire DAO balance is not yet supported; try specifying an asset ID or base denomination");
            // let prefix = dao::state_key::all_assets_balance();
            // let results: Vec<_> = client.prefix_domain(prefix).await?.try_collect().await?;
            // println!("DAO balance ({} unique assets):", results.len());
            // for (key, amount) in results {
            //     // Parse every key/value pair into a Value
            //     let asset_id: asset::Id = key
            //         .rsplit('/')
            //         .next()
            //         .expect("valid key")
            //         .parse()
            //         .expect("valid asset ID");
            //     let value = Value { asset_id, amount };
            //     let string = value.format(&denom_by_asset);
            //     println!("{string}");
            // }
        };

        Ok(())
    }
}
