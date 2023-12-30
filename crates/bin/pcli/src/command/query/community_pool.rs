use crate::App;
use anyhow::{Context, Result};
use futures::TryStreamExt;
use penumbra_asset::Value;
use penumbra_proto::{
    core::component::community_pool::v1alpha1::CommunityPoolAssetBalancesRequest,
    penumbra::core::component::community_pool::v1alpha1::query_service_client::QueryServiceClient as CommunityPoolQueryServiceClient,
};
use penumbra_view::ViewClient;
use std::io::{stdout, Write};

#[derive(Debug, clap::Subcommand)]
pub enum CommunityPoolCmd {
    /// Get the balance in the Community Pool, or the balance of a specific asset.
    Balance {
        /// Get only the balance of the specified asset.
        asset: Option<String>,
    },
}

impl CommunityPoolCmd {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            CommunityPoolCmd::Balance { asset } => self.print_balance(app, asset).await,
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

        let mut client = CommunityPoolQueryServiceClient::new(app.pd_channel().await?);
        let chain_id = app.view().app_params().await?.chain_params.chain_id;
        let balances = client
            .community_pool_asset_balances(CommunityPoolAssetBalancesRequest {
                chain_id,
                asset_ids: asset_id.map_or_else(std::vec::Vec::new, |id| vec![id.into()]),
            })
            .await?
            .into_inner()
            .try_collect::<Vec<_>>()
            .await
            .context("cannot process Community Pool balance data")?;

        let asset_cache = app.view().assets().await?;
        let mut writer = stdout();
        for balance_response in balances {
            let balance: Value = balance_response
                .balance
                .expect("balance should always be set")
                .try_into()
                .context("cannot parse balance")?;
            let value_str = balance.format(&asset_cache);

            writeln!(writer, "{value_str}")?;
        }

        Ok(())
    }
}
