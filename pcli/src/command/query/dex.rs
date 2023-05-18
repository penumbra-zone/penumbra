use anyhow::{Context, Result};
use comfy_table::{presets, Table};
use futures::{Stream, StreamExt, TryStreamExt};
use std::pin::Pin;

use penumbra_crypto::{
    dex::{
        execution::SwapExecution,
        lp::position::{self, Position},
        BatchSwapOutputData, DirectedTradingPair, TradingPair,
    },
    Asset, Value,
};
use penumbra_proto::client::v1alpha1::{
    specific_query_service_client::SpecificQueryServiceClient, AssetInfoRequest,
    BatchSwapOutputDataRequest, LiquidityPositionByIdRequest, LiquidityPositionsByPriceRequest,
    LiquidityPositionsRequest, SwapExecutionRequest,
};
use penumbra_view::ViewClient;
use tonic::transport::Channel;

use crate::{
    command::utils::{self, render_positions},
    App,
};

#[derive(Debug, clap::Subcommand)]
pub enum DexCmd {
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
    /// Display information about all liquidity positions known to the chain.
    #[clap(display_order(900))]
    AllPositions {
        /// Display closed and withdrawn liquidity positions as well as open ones.
        #[clap(long)]
        include_closed: bool,
    },
    /// Display a single position's state given a position ID.
    Position {
        /// The ID of the position to query.
        id: position::Id,
        /// If set, output raw JSON instead of a table.
        #[clap(long)]
        raw: bool,
    },
    /// Display open liquidity for a given pair, sorted by effective price.
    Positions {
        /// The trading pair to query, with ordering determining direction of trade (1=>2).
        trading_pair: DirectedTradingPair,
        /// A limit on the number of positions to display.
        #[clap(long)]
        limit: Option<u64>,
    },
}

impl DexCmd {
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

    pub async fn get_all_liquidity_positions(
        &self,
        mut client: SpecificQueryServiceClient<Channel>,
        include_closed: bool,
        chain_id: Option<String>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Position>> + Send + 'static>>> {
        let stream = client.liquidity_positions(LiquidityPositionsRequest {
            include_closed,
            chain_id: chain_id.unwrap_or_default(),
        });
        let stream = stream.await?.into_inner();

        Ok(stream
            .map_err(|e| anyhow::anyhow!("error fetching liquidity positions: {}", e))
            .and_then(|msg| async move {
                msg.data
                    .ok_or_else(|| anyhow::anyhow!("missing liquidity position in response data"))
                    .map(Position::try_from)?
            })
            .boxed())
    }

    pub async fn get_liquidity_positions_by_price(
        &self,
        mut client: SpecificQueryServiceClient<Channel>,
        pair: DirectedTradingPair,
        limit: Option<u64>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Position>> + Send + 'static>>> {
        let stream = client.liquidity_positions_by_price(LiquidityPositionsByPriceRequest {
            trading_pair: Some(pair.into()),
            limit: limit.unwrap_or_default(),
            ..Default::default()
        });
        let stream = stream.await?.into_inner();

        Ok(stream
            .map_err(|e| anyhow::anyhow!("error fetching liquidity positions: {}", e))
            .and_then(|msg| async move {
                msg.data
                    .ok_or_else(|| anyhow::anyhow!("missing liquidity position in response data"))
                    .map(Position::try_from)?
            })
            .boxed())
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

        let unit_1 = asset_1.denom.default_unit();
        let unit_2 = asset_2.denom.default_unit();

        let consumed_1 = outputs.delta_1 - outputs.unfilled_1;
        let consumed_2 = outputs.delta_2 - outputs.unfilled_2;

        println!("Batch Swap Outputs for height {}:", outputs.height);
        println!(
            "Trade {} => {}",
            unit_1.format_value(outputs.delta_1),
            unit_2
        );
        println!(
            "\tOutput:         {} for {}",
            unit_2.format_value(outputs.lambda_2),
            unit_1.format_value(consumed_1)
        );
        println!(
            "\tUnfilled Input: {}",
            unit_1.format_value(outputs.unfilled_1)
        );
        println!(
            "Trade {} => {}",
            unit_2.format_value(outputs.delta_2),
            unit_1
        );
        println!(
            "\tOutput:         {} for {}",
            unit_1.format_value(outputs.lambda_1),
            unit_2.format_value(consumed_2)
        );
        println!(
            "\tUnfilled Input: {}",
            unit_2.format_value(outputs.unfilled_2)
        );

        Ok(())
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
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
            DexCmd::AllPositions { include_closed } => {
                let client = app.specific_client().await?;
                let chain_id = app.view().chain_params().await?.chain_id;

                let positions_stream = self
                    .get_all_liquidity_positions(client.clone(), *include_closed, Some(chain_id))
                    .await?;

                let asset_cache = app.view().assets().await?;

                let positions = positions_stream.try_collect::<Vec<_>>().await?;

                println!("{}", utils::render_positions(&asset_cache, &positions));
            }
            DexCmd::Positions {
                trading_pair,
                limit,
            } => {
                let client = app.specific_client().await?;
                let positions = self
                    .get_liquidity_positions_by_price(client, *trading_pair, *limit)
                    .await?
                    .try_collect::<Vec<_>>()
                    .await?;
                let asset_cache = app.view().assets().await?;
                println!("{}", render_positions(&asset_cache, &positions));
            }
            DexCmd::Position { id, raw } => {
                let mut client = app.specific_client().await?;
                let position: Position = client
                    .liquidity_position_by_id(LiquidityPositionByIdRequest {
                        position_id: Some((*id).into()),
                        ..Default::default()
                    })
                    .await?
                    .into_inner()
                    .data
                    .ok_or_else(|| anyhow::anyhow!("position not found"))?
                    .try_into()?;

                if *raw {
                    println!("{}", serde_json::to_string_pretty(&position)?);
                } else {
                    let asset_cache = app.view().assets().await?;
                    let mut table = Table::new();
                    table.load_preset(presets::NOTHING);
                    table.add_row(vec!["ID".to_string(), id.to_string()]);
                    table.add_row(vec!["State".to_string(), position.state.to_string()]);
                    table.add_row(vec![
                        "Reserves 1".to_string(),
                        Value {
                            asset_id: position.phi.pair.asset_1(),
                            amount: position.reserves.r1,
                        }
                        .format(&asset_cache),
                    ]);
                    table.add_row(vec![
                        "Reserves 2".to_string(),
                        Value {
                            asset_id: position.phi.pair.asset_2(),
                            amount: position.reserves.r2,
                        }
                        .format(&asset_cache),
                    ]);
                    table.add_row(vec![
                        "Fee".to_string(),
                        format!("{}bps", position.phi.component.fee),
                    ]);
                    table.add_row(vec![
                        "p".to_string(),
                        position.phi.component.p.value().to_string(),
                    ]);
                    table.add_row(vec![
                        "q".to_string(),
                        position.phi.component.q.value().to_string(),
                    ]);
                    table.add_row(vec!["Nonce".to_string(), hex::encode(&position.nonce)]);
                    println!("{}", table);
                }
            }
        };

        Ok(())
    }
}
