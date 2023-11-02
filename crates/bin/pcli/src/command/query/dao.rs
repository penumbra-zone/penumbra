use crate::App;
use anyhow::{Context, Result};
use futures::TryStreamExt;
use penumbra_asset::Value;
use penumbra_proto::{
    core::component::dao::v1alpha1::DaoAssetBalancesRequest,
    penumbra::core::component::dao::v1alpha1::query_service_client::QueryServiceClient as DaoQueryServiceClient,
};
use penumbra_view::ViewClient;
use std::{
    collections::BTreeMap,
    io::{stdout, Write},
};

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

    pub async fn print_balance(&self, app: &mut App, asset: &Option<String>) -> Result<()> {
        let asset_id = asset.as_ref().map(|asset| {
            // Try to parse as an asset ID, then if it's not an asset ID, assume it's a unit name
            if let Ok(asset_id) = asset.parse() {
                asset_id
            } else {
                penumbra_asset::asset::REGISTRY
                    .parse_unit(asset.as_str())
                    .id()
            }
        });

        let mut client = DaoQueryServiceClient::new(app.pd_channel().await?);
        let chain_id = app.view().app_params().await?.chain_params.chain_id;
        let balances = client
            .dao_asset_balances(DaoAssetBalancesRequest {
                chain_id,
                asset_ids: asset_id.map_or_else(|| vec![], |id| vec![id.into()]),
            })
            .await?
            .into_inner()
            .try_collect::<Vec<_>>()
            .await
            .context("cannot process dao balance data")?;

        let asset_cache = app.view().assets().await?;
        let mut writer = stdout();
        for balance_response in balances {
            let balance: Value = balance_response
                .balance
                .expect("balance should always be set")
                .try_into()
                .context("cannot parse balance")?;
            let value_str = balance.format(&asset_cache);

            writeln!(writer, "{value_str}");
        }
        // if let Some(asset_id) = asset_id {
        //     let key = state_key::balance_for_asset(asset_id);
        //     let amount: Amount = client
        //         .key_domain(&key)
        //         .await?
        //         .context(format!("No balance found for asset {asset_id}"))?;

        //     let value = Value { asset_id, amount };
        //     let value_str = value.format(&asset_cache);

        //     println!("{value_str}");
        // } else {
        //     let prefix = dao::state_key::all_assets_balance();
        //     let results: Vec<_> = client.prefix_domain(prefix).await?.try_collect().await?;

        //     println!("DAO balance ({} unique assets):", results.len());

        //     for (key, amount) in results {
        //         // Parse every key/value pair into a Value
        //         let asset_id: asset::Id = key
        //             .rsplit('/')
        //             .next()
        //             .expect("valid key")
        //             .parse()
        //             .expect("valid asset ID");
        //         let value = Value { asset_id, amount };
        //         let value_str = value.format(&asset_cache);
        //         println!("{value_str}");
        //     }
        // };

        Ok(())
    }
}
