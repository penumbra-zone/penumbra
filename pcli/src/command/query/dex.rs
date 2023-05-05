use std::pin::Pin;

use anyhow::{Context, Result};
use comfy_table::{presets, Table};
use futures::{Future, FutureExt, Stream, StreamExt, TryStreamExt};

use penumbra_crypto::{
    asset,
    dex::{
        execution::SwapExecution,
        lp::{position::Position, Reserves},
        BatchSwapOutputData, TradingPair,
    },
    Asset,
};
use penumbra_proto::client::v1alpha1::{
    specific_query_service_client::SpecificQueryServiceClient, AssetInfoRequest,
    BatchSwapOutputDataRequest, LiquidityPositionsRequest, StubCpmmReservesRequest,
    SwapExecutionRequest,
};
use penumbra_view::ViewClient;
use tonic::transport::Channel;

use crate::App;

#[derive(Debug, clap::Subcommand)]
pub enum DexCmd {
    /// Display information about constant-pair market maker reserves.
    CPMMReserves {
        /// The trading pair to query for CPMM Reserves.
        trading_pair: TradingPair,
    },
    /// Display information about a specific trading pair & height's batch swap.
    BatchOutputs {
        /// The height to query for batch outputs.
        #[clap(long)]
        height: u64,
        /// The trading pair to query for batch outputs.
        trading_pair: TradingPair,
    },
    /// Display information about a specific trading pair & height's swap execution.
    SwapExecution {
        /// The height to query for the swap execution.
        #[clap(long)]
        height: u64,
        /// The trading pair to query for the swap execution.
        trading_pair: TradingPair,
    },
    /// Display information about liquidity positions known to the chain.
    LiquidityPositions {
        /// Display closed and withdrawn liquidity positions.
        #[clap(long, default_value_t = false, action=clap::ArgAction::SetTrue)]
        only_open: bool,
    },
}

impl DexCmd {
    pub async fn print_cpmm_reserves(
        &self,
        app: &mut App,
        trading_pair: &TradingPair,
    ) -> Result<()> {
        let mut client = app.specific_client().await?;

        let chain_id = app.view().chain_params().await?.chain_id;
        let reserves_data: Reserves = client
            .stub_cpmm_reserves(StubCpmmReservesRequest {
                trading_pair: Some((*trading_pair).into()),
                chain_id: chain_id.clone(),
            })
            .await?
            .into_inner()
            .try_into()
            .context("cannot parse stub CPMM reserves data")?;
        println!("Constant-Product Market Maker Reserves:");
        let mut table = Table::new();

        let asset_1: Asset = client
            .asset_info(AssetInfoRequest {
                asset_id: Some(trading_pair.asset_1().into()),
                chain_id: chain_id.clone(),
            })
            .await?
            .into_inner()
            .asset
            .unwrap()
            .try_into()?;

        let display_denom_1 = asset_1.denom.best_unit_for(reserves_data.r1);

        let (denom_1, reserve_amount_1) = (
            format!("{display_denom_1}"),
            display_denom_1.format_value(reserves_data.r1),
        );

        let asset_2: Asset = client
            .asset_info(AssetInfoRequest {
                asset_id: Some(trading_pair.asset_2().into()),
                chain_id: chain_id.clone(),
            })
            .await?
            .into_inner()
            .asset
            .unwrap()
            .try_into()?;
        let display_denom_2 = asset_2.denom.best_unit_for(reserves_data.r2);
        let (denom_2, reserve_amount_2) = (
            format!("{display_denom_2}"),
            display_denom_2.format_value(reserves_data.r2),
        );

        table.load_preset(presets::NOTHING);
        table
            .set_header(vec!["Denomination", "Reserve Amount"])
            .add_row(vec![denom_1, reserve_amount_1])
            .add_row(vec![denom_2, reserve_amount_2]);

        println!("{table}");

        Ok(())
    }

    pub async fn get_batch_outputs(
        &self,
        app: &mut App,
        chain_id: String,
        height: &u64,
        trading_pair: &TradingPair,
    ) -> Result<BatchSwapOutputData> {
        let mut client = app.specific_client().await?;
        client
            .batch_swap_output_data(BatchSwapOutputDataRequest {
                height: *height,
                trading_pair: Some((*trading_pair).into()),
                chain_id,
            })
            .await?
            .into_inner()
            .try_into()
            .context("cannot parse batch swap output data")
    }

    pub async fn get_swap_execution(
        &self,
        app: &mut App,
        height: &u64,
        trading_pair: &TradingPair,
    ) -> Result<SwapExecution> {
        let mut client = app.specific_client().await?;
        client
            .swap_execution(SwapExecutionRequest {
                height: *height,
                trading_pair: Some((*trading_pair).into()),
                ..Default::default()
            })
            .await?
            .into_inner()
            .swap_execution
            .ok_or_else(|| anyhow::anyhow!("proto response missing swap execution"))?
            .try_into()
            .context("cannot parse batch swap output data")
    }

    pub async fn get_liquidity_positions(
        &self,
        mut client: SpecificQueryServiceClient<Channel>,
        only_open: bool,
        chain_id: String,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<Pin<Box<dyn Stream<Item = Result<Position>> + Send + 'static>>>,
                > + Send
                + 'static,
        >,
    > {
        async move {
            let stream = client.liquidity_positions(LiquidityPositionsRequest {
                only_open,
                chain_id,
            });
            let stream = stream.await?.into_inner();

            Ok(stream
                .map_err(|e| anyhow::anyhow!("error fetching liquidity positions: {}", e))
                .and_then(|msg| async move {
                    msg.data
                        .ok_or_else(|| {
                            anyhow::anyhow!("missing liquidity position in response data")
                        })
                        .map(Position::try_from)?
                })
                .boxed())
        }
        .boxed()
    }

    pub async fn print_swap_execution(
        &self,
        app: &mut App,
        height: &u64,
        trading_pair: &TradingPair,
    ) -> Result<()> {
        let swap_execution = self.get_swap_execution(app, height, trading_pair).await?;

        let cache = app.view().assets().await?;

        for trace in swap_execution.traces {
            println!(
                "{}",
                trace
                    .iter()
                    .map(|v| v.format(&cache))
                    .collect::<Vec<_>>()
                    .join(" => ")
            );
        }

        Ok(())
    }

    pub async fn print_batch_outputs(
        &self,
        app: &mut App,
        height: &u64,
        trading_pair: &TradingPair,
    ) -> Result<()> {
        let mut client = app.specific_client().await?;

        let chain_id = app.view().chain_params().await?.chain_id;

        let outputs = self
            .get_batch_outputs(app, chain_id.clone(), height, trading_pair)
            .await?;

        let asset_1: Asset = client
            .asset_info(AssetInfoRequest {
                asset_id: Some(trading_pair.asset_1().into()),
                chain_id: chain_id.clone(),
            })
            .await?
            .into_inner()
            .asset
            .unwrap()
            .try_into()?;

        let display_denom_1 = asset_1.denom.best_unit_for(std::cmp::max(
            outputs.delta_1,
            outputs.lambda_1_1 + outputs.lambda_1_2,
        ));

        let (denom_1, input_amount_1, output_amount_1) = (
            format!("{display_denom_1}"),
            display_denom_1.format_value(outputs.delta_1),
            display_denom_1.format_value(outputs.lambda_1_1 + outputs.lambda_1_2),
        );
        let asset_2: Asset = client
            .asset_info(AssetInfoRequest {
                asset_id: Some(trading_pair.asset_2().into()),
                chain_id: chain_id.clone(),
            })
            .await?
            .into_inner()
            .asset
            .unwrap()
            .try_into()?;
        let display_denom_2 = asset_2.denom.best_unit_for(std::cmp::max(
            outputs.delta_2,
            outputs.lambda_2_1 + outputs.lambda_2_2,
        ));

        let (denom_2, input_amount_2, output_amount_2) = (
            format!("{display_denom_2}"),
            display_denom_2.format_value(outputs.delta_2),
            display_denom_2.format_value(outputs.lambda_2_1 + outputs.lambda_2_2),
        );

        println!("Batch Swap Outputs for height {}:", outputs.height);
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);
        table
            .set_header(vec!["Denomination", "Input Amount", "Output Amount"])
            .add_row(vec![denom_1, input_amount_1, output_amount_1])
            .add_row(vec![denom_2, input_amount_2, output_amount_2]);

        println!("{table}");

        Ok(())
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            DexCmd::CPMMReserves { trading_pair } => {
                self.print_cpmm_reserves(app, trading_pair).await?;
            }
            DexCmd::BatchOutputs {
                height,
                trading_pair,
            } => {
                self.print_batch_outputs(app, height, trading_pair).await?;
            }
            DexCmd::SwapExecution {
                height,
                trading_pair,
            } => {
                self.print_swap_execution(app, height, trading_pair).await?;
            }
            DexCmd::LiquidityPositions { only_open } => {
                let client = app.specific_client().await?;
                let chain_id = app.view().chain_params().await?.chain_id;

                let positions_stream = self
                    .get_liquidity_positions(client.clone(), *only_open, chain_id.clone())
                    .await
                    .await?;

                let asset_cache = app.view().assets().await?;

                let positions = positions_stream.try_collect::<Vec<_>>().await?;

                println!("{}", render_positions(&asset_cache, &positions));
            }
        };

        Ok(())
    }
}

fn render_positions(asset_cache: &asset::Cache, positions: &[Position]) -> String {
    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table.set_header(vec![
        "ID",
        "Trading Pair",
        "State",
        "Reserves",
        "Trading Function",
    ]);

    for position in positions {
        let trading_pair = position.phi.pair;
        let denom_1 = asset_cache
            .get(&trading_pair.asset_1())
            .expect("asset should be known to view service");
        let denom_2 = asset_cache
            .get(&trading_pair.asset_2())
            .expect("asset should be known to view service");

        let display_denom_1 = denom_1.default_unit();
        let display_denom_2 = denom_2.default_unit();

        table.add_row(vec![
            format!("{}", position.id()),
            format!("({}, {})", display_denom_1, display_denom_2),
            position.state.to_string(),
            format!(
                "({}, {})",
                display_denom_1.format_value(position.reserves.r1),
                display_denom_2.format_value(position.reserves.r2)
            ),
            format!(
                "fee: {}bps, p/q: {:.6}, q/p: {:.6}",
                position.phi.component.fee,
                f64::from(position.phi.component.p) / f64::from(position.phi.component.q),
                f64::from(position.phi.component.q) / f64::from(position.phi.component.p),
            ),
        ]);
    }

    format!("{table}")
}
