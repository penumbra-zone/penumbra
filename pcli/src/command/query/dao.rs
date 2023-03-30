use std::collections::BTreeMap;

use anyhow::Result;
use penumbra_app::dao;
use penumbra_chain::KnownAssets;
use penumbra_crypto::{
    asset::{Cache, Denom},
    Amount, Asset, Value,
};
use penumbra_proto::client::v1alpha1::{AssetListRequest, ChainParametersRequest};

use crate::App;

#[derive(Debug, clap::Subcommand)]
pub enum DaoCmd {
    Balance { asset: Option<String> },
}

impl DaoCmd {
    // TODO: this is duplicated between various pcli q subcommands, is there a single place it could live?
    async fn get_asset_cache(&self, app: &mut App) -> Result<(String, Cache)> {
        let mut oblivious_client = app.oblivious_client().await?;

        let chain_params = oblivious_client
            .chain_parameters(tonic::Request::new(ChainParametersRequest {
                chain_id: "".to_string(),
            }))
            .await?
            .into_inner()
            .chain_parameters
            .ok_or_else(|| anyhow::anyhow!("empty ChainParametersResponse message"))?;

        let chain_id = chain_params.chain_id;
        let assets = oblivious_client
            .asset_list(tonic::Request::new(AssetListRequest {
                chain_id: chain_id.clone(),
            }))
            .await?
            .into_inner()
            .asset_list
            .ok_or_else(|| anyhow::anyhow!("empty AssetListResponse message"))?
            .assets;

        let mut known_assets = KnownAssets(vec![]);
        for new_asset in assets {
            let new_asset = Asset::try_from(new_asset)?;
            known_assets.0.push(new_asset);
        }

        Ok((chain_id, known_assets.into()))
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            DaoCmd::Balance { asset } => self.print_balance(app, asset).await,
        }
    }

    pub async fn print_balance(&self, app: &mut App, asset: &Option<String>) -> Result<()> {
        let (_chain_id, denom_by_asset) = self.get_asset_cache(app).await?;
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
