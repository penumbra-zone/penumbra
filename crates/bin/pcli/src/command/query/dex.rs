use anyhow::{Context, Result};
use comfy_table::{presets, Table};
use futures::{Stream, StreamExt, TryStreamExt};
use std::pin::Pin;
use serde::Serialize;
use colored_json::ToColoredJson;

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

// JSON Output Types
#[derive(Serialize, Debug)]
pub struct PositionJson {
    pub id: String,
    pub state: String,
    pub trading_pair: TradingPairJson,
    pub reserves: ReservesJson,
    pub fee_bps: u32,
    pub prices: Option<PricesJson>,
    pub phi: PhiJson,
    pub nonce: String,
    pub close_on_fill: bool,
}

#[derive(Serialize, Debug)]
pub struct TradingPairJson {
    pub asset_1: String,
    pub asset_2: String,
    pub asset_1_symbol: Option<String>,
    pub asset_2_symbol: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ReservesJson {
    pub r1: String,
    pub r2: String,
    pub r1_formatted: String, // "1.5penumbra"
    pub r2_formatted: String, // "3.2gm"
    pub r1_raw: String,       // Raw amount for calculations
    pub r2_raw: String,
}

#[derive(Serialize, Debug)]
pub struct PricesJson {
    pub price_str: Option<String>, // Human readable price like "2.5 gm/penumbra"
    pub effective_price: Option<f64>,
    pub price_direction: String,
}

#[derive(Serialize, Debug)]
pub struct PhiJson {
    pub p: String,
    pub q: String,
    pub fee: u32,
}

#[derive(Serialize, Debug)]
pub struct PositionsResponse {
    pub positions: Vec<PositionJson>,
    pub count: usize,
    pub timestamp: String,
    pub trading_pair_filter: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct BatchOutputsJson {
    pub height: u64,
    pub trading_pair: TradingPairJson,
    pub delta_1: String,
    pub delta_2: String,
    pub lambda_1: String,
    pub lambda_2: String,
    pub unfilled_1: String,
    pub unfilled_2: String,
    pub consumed_1: String,
    pub consumed_2: String,
}

#[derive(Serialize, Debug)]
pub struct SwapExecutionJson {
    pub input: String,
    pub output: String,
    pub traces: Vec<Vec<String>>,
    pub trace_count: usize,
}

#[derive(Serialize, Debug)]
pub struct SimulationJson {
    pub input: String,
    pub output_asset: String,
    pub execution: SwapExecutionJson,
}

// Helper functions for JSON conversion
fn position_to_json(position: &Position, asset_cache: &asset::Cache) -> PositionJson {
    let trading_pair = position.phi.pair;
    let asset_1_meta = asset_cache.get(&trading_pair.asset_1());
    let asset_2_meta = asset_cache.get(&trading_pair.asset_2());

    let trading_pair_json = TradingPairJson {
        asset_1: trading_pair.asset_1().to_string(),
        asset_2: trading_pair.asset_2().to_string(),
        asset_1_symbol: asset_1_meta.map(|m| m.symbol().to_string()),
        asset_2_symbol: asset_2_meta.map(|m| m.symbol().to_string()),
    };

    let reserves_json = ReservesJson {
        r1: position.reserves.r1.to_string(),
        r2: position.reserves.r2.to_string(),
        r1_formatted: Value {
            amount: position.reserves.r1,
            asset_id: trading_pair.asset_1(),
        }.format(asset_cache),
        r2_formatted: Value {
            amount: position.reserves.r2,
            asset_id: trading_pair.asset_2(),
        }.format(asset_cache),
        r1_raw: position.reserves.r1.value().to_string(),
        r2_raw: position.reserves.r2.value().to_string(),
    };

    let prices_json = extract_position_prices(position, asset_cache);

    PositionJson {
        id: position.id().to_string(),
        state: position.state.to_string(),
        trading_pair: trading_pair_json,
        reserves: reserves_json,
        fee_bps: position.phi.component.fee,
        prices: prices_json,
        phi: PhiJson {
            p: position.phi.component.p.value().to_string(),
            q: position.phi.component.q.value().to_string(),
            fee: position.phi.component.fee,
        },
        nonce: hex::encode(position.nonce),
        close_on_fill: position.close_on_fill,
    }
}

fn extract_position_prices(position: &Position, asset_cache: &asset::Cache) -> Option<PricesJson> {
    if let Some(sell_order) = position.interpret_as_sell() {
        Some(PricesJson {
            price_str: sell_order.price_str(asset_cache).ok(),
            effective_price: calculate_effective_price(position),
            price_direction: "sell_asset_1".to_string(),
        })
    } else if let Some(buy_order) = position.interpret_as_buy() {
        Some(PricesJson {
            price_str: buy_order.price_str(asset_cache).ok(),
            effective_price: calculate_effective_price(position),
            price_direction: "buy_asset_1".to_string(),
        })
    } else if let Some((sell_1, _sell_2)) = position.interpret_as_mixed() {
        // For mixed positions, show the sell_1 price
        Some(PricesJson {
            price_str: sell_1.price_str(asset_cache).ok(),
            effective_price: calculate_effective_price(position),
            price_direction: "mixed".to_string(),
        })
    } else {
        None
    }
}

fn calculate_effective_price(position: &Position) -> Option<f64> {
    if position.reserves.r1.value() > 0 && position.reserves.r2.value() > 0 {
        Some(position.reserves.r2.value() as f64 / position.reserves.r1.value() as f64)
    } else {
        None
    }
}

fn swap_execution_to_json(execution: &SwapExecution, asset_cache: &asset::Cache) -> SwapExecutionJson {
    let traces: Vec<Vec<String>> = execution
        .traces
        .iter()
        .map(|trace| {
            trace
                .iter()
                .map(|value| value.format(asset_cache))
                .collect()
        })
        .collect();

    SwapExecutionJson {
        input: execution.input.format(asset_cache),
        output: execution.output.format(asset_cache),
        trace_count: execution.traces.len(),
        traces,
    }
}

fn get_current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}", duration.as_secs())
}

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
        /// If set, output raw JSON instead of a table.
        #[clap(long)]
        json: bool,
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
        /// If set, output raw JSON instead of a table.
        #[clap(long)]
        json: bool,
    },
    /// Display information about an arb execution at a specific height.
    #[clap(visible_alias = "arb")]
    ArbExecution {
        /// The height to query for the swap execution.
        #[clap(long)]
        height: u64,
        /// If set, output raw JSON instead of a table.
        #[clap(long)]
        json: bool,
    },
    /// Display information about all liquidity positions known to the chain.
    #[clap(display_order(900))]
    AllPositions {
        /// Display closed and withdrawn liquidity positions as well as open ones.
        #[clap(long)]
        include_closed: bool,
        /// If set, output raw JSON instead of a table.
        #[clap(long)]
        json: bool,
    },
    /// Display a single position's state given a position ID.
    Position {
        /// The ID of the position to query.
        id: position::Id,
        /// If set, output raw JSON instead of a table.
        #[clap(long)]
        json: bool,
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
        /// If set, output raw JSON instead of a table.
        #[clap(long)]
        json: bool,
    },
    /// Simulates execution of a trade against the current DEX state.
    Simulate {
        /// The input amount to swap, written as a typed value 1.87penumbra, 12cubes, etc.
        input: String,
        /// The denomination to swap the input into, e.g. `gm`
        #[clap(long, display_order = 100)]
        into: String,
        /// If set, output raw JSON instead of a table.
        #[clap(long)]
        json: bool,
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
                json,
            } => {
                if *json {
                    let outputs = self.get_batch_outputs(app, height, trading_pair).await?;
                    let asset_cache = app.view().assets().await?;
                    let asset_1_meta = asset_cache.get(&trading_pair.asset_1());
                    let asset_2_meta = asset_cache.get(&trading_pair.asset_2());
                    
                    let consumed_1 = outputs.delta_1 - outputs.unfilled_1;
                    let consumed_2 = outputs.delta_2 - outputs.unfilled_2;

                    let json_output = BatchOutputsJson {
                        height: outputs.height,
                        trading_pair: TradingPairJson {
                            asset_1: trading_pair.asset_1().to_string(),
                            asset_2: trading_pair.asset_2().to_string(),
                            asset_1_symbol: asset_1_meta.map(|m| m.symbol().to_string()),
                            asset_2_symbol: asset_2_meta.map(|m| m.symbol().to_string()),
                        },
                        delta_1: outputs.delta_1.to_string(),
                        delta_2: outputs.delta_2.to_string(),
                        lambda_1: outputs.lambda_1.to_string(),
                        lambda_2: outputs.lambda_2.to_string(),
                        unfilled_1: outputs.unfilled_1.to_string(),
                        unfilled_2: outputs.unfilled_2.to_string(),
                        consumed_1: consumed_1.to_string(),
                        consumed_2: consumed_2.to_string(),
                    };

                    let json_str = serde_json::to_string_pretty(&json_output)?;
                    println!("{}", json_str.to_colored_json_auto()?);
                } else {
                    self.print_batch_outputs(app, height, trading_pair).await?;
                }
            }
            DexCmd::SwapExecution {
                height,
                trading_pair,
                json,
            } => {
                let swap_execution = self.get_swap_execution(app, height, trading_pair).await?;

                if *json {
                    let asset_cache = app.view().assets().await?;
                    let json_output = swap_execution_to_json(&swap_execution, &asset_cache);
                    let json_str = serde_json::to_string_pretty(&json_output)?;
                    println!("{}", json_str.to_colored_json_auto()?);
                } else {
                    self.print_swap_execution(app, &swap_execution).await?;
                }
            }
            DexCmd::ArbExecution { height, json } => {
                let swap_execution = self.get_arb_execution(app, height).await?;

                if *json {
                    let asset_cache = app.view().assets().await?;
                    let json_output = swap_execution_to_json(&swap_execution, &asset_cache);
                    let json_str = serde_json::to_string_pretty(&json_output)?;
                    println!("{}", json_str.to_colored_json_auto()?);
                } else {
                    self.print_swap_execution(app, &swap_execution).await?;
                }
            }
            DexCmd::Simulate { input, into, json } => {
                let input_value = input.parse::<Value>()?;
                let into_asset = asset::REGISTRY.parse_unit(into.as_str()).base();

                let swap_execution = self.get_simulated_execution(app, input_value.clone(), into_asset.id()).await?;
                
                if *json {
                    let asset_cache = app.view().assets().await?;
                    let json_output = SimulationJson {
                        input: input_value.format(&asset_cache),
                        output_asset: into_asset.to_string(),
                        execution: swap_execution_to_json(&swap_execution, &asset_cache),
                    };
                    let json_str = serde_json::to_string_pretty(&json_output)?;
                    println!("{}", json_str.to_colored_json_auto()?);
                } else {
                    self.print_swap_execution(app, &swap_execution).await?;
                }
            }
            DexCmd::AllPositions { include_closed, json } => {
                let client = DexQueryServiceClient::new(app.pd_channel().await?);
                let positions_stream = self
                    .get_all_liquidity_positions(client.clone(), *include_closed)
                    .await?;
                let asset_cache = app.view().assets().await?;
                let positions = positions_stream.try_collect::<Vec<_>>().await?;

                if *json {
                    let json_positions: Vec<PositionJson> = positions
                        .iter()
                        .map(|pos| position_to_json(pos, &asset_cache))
                        .collect();
                    
                    let response = PositionsResponse {
                        positions: json_positions,
                        count: positions.len(),
                        timestamp: get_current_timestamp(),
                        trading_pair_filter: None,
                    };
                    
                    let json_str = serde_json::to_string_pretty(&response)?;
                    println!("{}", json_str.to_colored_json_auto()?);
                } else {
                    println!("{}", utils::render_positions(&asset_cache, &positions));
                }
            }
            DexCmd::Positions {
                trading_pair,
                limit,
                json,
            } => {
                let client = DexQueryServiceClient::new(app.pd_channel().await?);
                let positions = self
                    .get_liquidity_positions_by_price(client, *trading_pair, *limit)
                    .await?
                    .try_collect::<Vec<_>>()
                    .await?;
                let asset_cache = app.view().assets().await?;

                if *json {
                    let json_positions: Vec<PositionJson> = positions
                        .iter()
                        .map(|pos| position_to_json(pos, &asset_cache))
                        .collect();
                    
                    let response = PositionsResponse {
                        positions: json_positions,
                        count: positions.len(),
                        timestamp: get_current_timestamp(),
                        trading_pair_filter: Some(format!("{}:{}", 
                            //trading_pair.start.base().to_string(),
                            //trading_pair.end.base().to_string()
                            trading_pair.start.to_string(),
                            trading_pair.end.to_string()
                        )),
                    };
                    
                    let json_str = serde_json::to_string_pretty(&response)?;
                    println!("{}", json_str.to_colored_json_auto()?);
                } else {
                    println!("{}", render_positions(&asset_cache, &positions));
                }
            }
            DexCmd::Position { id, json } => {
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

                if *json {
                    let asset_cache = app.view().assets().await?;
                    let json_position = position_to_json(&position, &asset_cache);
                    let json_str = serde_json::to_string_pretty(&json_position)?;
                    println!("{}", json_str.to_colored_json_auto()?);
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
