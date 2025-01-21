use anyhow::{Context, Result};
use comfy_table::{presets, Table};
use futures::{Stream, StreamExt, TryStreamExt};
use std::pin::Pin;

use penumbra_sdk_asset::{asset, asset::Metadata, Value};
use penumbra_sdk_dex::{
    lp::position::{self, Position},
    BatchSwapOutputData, DirectedTradingPair, SwapExecution, TradingPair,
};
use penumbra_sdk_proto::core::component::{
    dex::v1::{
        query_service_client::QueryServiceClient as DexQueryServiceClient,
        simulation_service_client::SimulationServiceClient, ArbExecutionRequest,
        BatchSwapOutputDataRequest, LiquidityPositionByIdRequest, LiquidityPositionsByPriceRequest,
        LiquidityPositionsRequest, SimulateTradeRequest, SwapExecutionRequest,
    },
    shielded_pool::v1::{
        query_service_client::QueryServiceClient as ShieldedPoolQueryServiceClient,
        AssetMetadataByIdRequest,
    },
};
use penumbra_sdk_view::ViewClient;
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
        /// Pairs must be specified with a colon separating them, e.g. "penumbra:test_usd".
        #[clap(value_name = "asset_1:asset_2")]
        trading_pair: TradingPair,
    },
    /// Display information about a specific trading pair & height's swap execution.
    SwapExecution {
        /// The height to query for the swap execution.
        #[clap(long)]
        height: u64,
        /// The trading pair to query for the swap execution.
        /// Pairs must be specified with a colon separating them, e.g. "penumbra:test_usd".
        #[clap(value_name = "asset_1:asset_2")]
        trading_pair: DirectedTradingPair,
    },
    /// Display information about an arb execution at a specific height.
    #[clap(visible_alias = "arb")]
    ArbExecution {
        /// The height to query for the swap execution.
        #[clap(long)]
        height: u64,
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
        /// Pairs must be specified with a colon separating them, e.g. "penumbra:test_usd".
        #[clap(value_name = "asset_1:asset_2")]
        trading_pair: DirectedTradingPair,
        /// A limit on the number of positions to display.
        #[clap(long)]
        limit: Option<u64>,
    },
    /// Simulates execution of a trade against the current DEX state.
    Simulate {
        /// The input amount to swap, written as a typed value 1.87penumbra, 12cubes, etc.
        input: String,
        /// The denomination to swap the input into, e.g. `gm`
        #[clap(long, display_order = 100)]
        into: String,
    },
}

impl DexCmd {
    pub async fn get_batch_outputs(
        &self,
        app: &mut App,
        height: &u64,
        trading_pair: &TradingPair,
    ) -> Result<BatchSwapOutputData> {
        let mut client = DexQueryServiceClient::new(app.pd_channel().await?);
        client
            .batch_swap_output_data(BatchSwapOutputDataRequest {
                height: *height,
                trading_pair: Some((*trading_pair).into()),
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
        trading_pair: &DirectedTradingPair,
    ) -> Result<SwapExecution> {
        let mut client = DexQueryServiceClient::new(app.pd_channel().await?);

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

    pub async fn get_arb_execution(&self, app: &mut App, height: &u64) -> Result<SwapExecution> {
        let mut client = DexQueryServiceClient::new(app.pd_channel().await?);
        client
            .arb_execution(ArbExecutionRequest {
                height: *height,
                ..Default::default()
            })
            .await?
            .into_inner()
            .swap_execution
            .ok_or_else(|| anyhow::anyhow!("proto response missing arb execution"))?
            .try_into()
            .context("cannot parse batch swap output data")
    }

    pub async fn get_simulated_execution(
        &self,
        app: &mut App,
        input: Value,
        output: asset::Id,
    ) -> Result<SwapExecution> {
        use penumbra_sdk_proto::core::component::dex::v1::simulate_trade_request::{
            routing::Setting, Routing,
        };
        let mut client = SimulationServiceClient::new(app.pd_channel().await?);
        client
            .simulate_trade(SimulateTradeRequest {
                input: Some(input.into()),
                output: Some(output.into()),
                routing: Some(Routing {
                    setting: Some(Setting::Default(Default::default())),
                }),
            })
            .await?
            .into_inner()
            .output
            .ok_or_else(|| anyhow::anyhow!("proto response missing swap execution"))?
            .try_into()
            .context("cannot parse simulation response")
    }

    pub async fn get_all_liquidity_positions(
        &self,
        mut client: DexQueryServiceClient<Channel>,
        include_closed: bool,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Position>> + Send + 'static>>> {
        let stream = client.liquidity_positions(LiquidityPositionsRequest { include_closed });
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
        mut client: DexQueryServiceClient<Channel>,
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
        swap_execution: &SwapExecution,
    ) -> Result<()> {
        let cache = app.view().assets().await?;

        println!(
            "{} => {} via:",
            swap_execution.input.format(&cache),
            swap_execution.output.format(&cache),
        );

        // Try to make a nice table of execution traces. To do this, first find
        // the max length of any subtrace:
        let max_trace_len = swap_execution
            .traces
            .iter()
            .map(|trace| trace.len())
            .max()
            .unwrap_or(0);

        // Spacer | trace hops | trace price
        let column_count = 1 + max_trace_len + 1;

        let mut table = Table::new();
        table.load_preset(presets::NOTHING);
        let mut headers = vec![""; column_count];
        headers[1] = "Trace";
        headers[column_count - 1] = "Subprice";
        table.set_header(headers);

        let price_string = |input: Value, output: Value| -> String {
            use penumbra_sdk_dex::lp::SellOrder;
            format!(
                "{}/{}",
                SellOrder {
                    offered: output,
                    desired: input,
                    fee: 0,
                }
                .price_str(&cache)
                .expect("assets are known"),
                // kind of hacky, this is assuming coincidency between price_str calcs
                // and this code
                Value {
                    asset_id: output.asset_id,
                    amount: cache
                        .get(&output.asset_id)
                        .expect("asset ID should exist in the cache")
                        .default_unit()
                        .unit_amount(),
                }
                .format(&cache)
            )
        };

        for trace in &swap_execution.traces {
            let mut row = vec![String::new(); column_count];
            // Put all but the last element of the trace in the columns, left-to-right
            for i in 0..(trace.len() - 1) {
                row[1 + i] = format!("{} =>", trace[i].format(&cache));
            }
            // Right-align the last element of the trace, in case subtraces have different lengths
            row[column_count - 2] = trace
                .last()
                .context("trace should have elements")?
                .format(&cache)
                .to_string();
            // Print the price in the last column.
            row[column_count - 1] = price_string(
                *trace.first().context("trace should have elements")?,
                *trace.last().context("trace should have elements")?,
            );

            table.add_row(row);
        }

        println!("{}", table);

        Ok(())
    }

    pub async fn print_batch_outputs(
        &self,
        app: &mut App,
        height: &u64,
        trading_pair: &TradingPair,
    ) -> Result<()> {
        let mut client = ShieldedPoolQueryServiceClient::new(app.pd_channel().await?);

        let outputs = self.get_batch_outputs(app, height, trading_pair).await?;

        let asset_1: Metadata = client
            .asset_metadata_by_id(AssetMetadataByIdRequest {
                asset_id: Some(trading_pair.asset_1().into()),
            })
            .await?
            .into_inner()
            .denom_metadata
            .context("denom metadata for asset 1 not found")?
            .try_into()?;
        let asset_2: Metadata = client
            .asset_metadata_by_id(AssetMetadataByIdRequest {
                asset_id: Some(trading_pair.asset_2().into()),
            })
            .await?
            .into_inner()
            .denom_metadata
            .context("denom metadata for asset 2 not found")?
            .try_into()?;

        let unit_1 = asset_1.default_unit();
        let unit_2 = asset_2.default_unit();

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
                let swap_execution = self.get_swap_execution(app, height, trading_pair).await?;

                self.print_swap_execution(app, &swap_execution).await?;
            }
            DexCmd::ArbExecution { height } => {
                let swap_execution = self.get_arb_execution(app, height).await?;

                self.print_swap_execution(app, &swap_execution).await?;
            }
            DexCmd::Simulate { input, into } => {
                let input = input.parse::<Value>()?;
                let into = asset::REGISTRY.parse_unit(into.as_str()).base();

                let swap_execution = self.get_simulated_execution(app, input, into.id()).await?;
                self.print_swap_execution(app, &swap_execution).await?;
            }
            DexCmd::AllPositions { include_closed } => {
                let client = DexQueryServiceClient::new(app.pd_channel().await?);

                let positions_stream = self
                    .get_all_liquidity_positions(client.clone(), *include_closed)
                    .await?;

                let asset_cache = app.view().assets().await?;

                let positions = positions_stream.try_collect::<Vec<_>>().await?;

                println!("{}", utils::render_positions(&asset_cache, &positions));
            }
            DexCmd::Positions {
                trading_pair,
                limit,
            } => {
                let client = DexQueryServiceClient::new(app.pd_channel().await?);
                let positions = self
                    .get_liquidity_positions_by_price(client, *trading_pair, *limit)
                    .await?
                    .try_collect::<Vec<_>>()
                    .await?;
                let asset_cache = app.view().assets().await?;
                println!("{}", render_positions(&asset_cache, &positions));
            }
            DexCmd::Position { id, raw } => {
                let mut client = DexQueryServiceClient::new(app.pd_channel().await?);
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
                    table.add_row(vec!["Nonce".to_string(), hex::encode(position.nonce)]);
                    println!("{}", table);
                }
            }
        };

        Ok(())
    }
}
