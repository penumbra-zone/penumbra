use std::path::Path;

use crate::command::tx::FeeTier;
use crate::App;
use anyhow::{anyhow, bail, Context};
use clap::Subcommand;
use comfy_table::presets;
use debug::DebugGda;
use dialoguer::Confirm;
use penumbra_asset::{asset::Cache, Value};
use penumbra_auction::auction::dutch::actions::ActionDutchAuctionWithdrawPlan;
use penumbra_auction::auction::{dutch::DutchAuction, dutch::DutchAuctionDescription, AuctionId};
use penumbra_dex::lp::position::Position;
use penumbra_keys::keys::AddressIndex;
use penumbra_num::Amount;
use penumbra_proto::{view::v1::GasPricesRequest, DomainType, Name};
use penumbra_view::SpendableNoteRecord;
use penumbra_view::ViewClient;
use penumbra_wallet::plan::Planner;
use rand::RngCore;
use rand_core::OsRng;
use serde_json;

mod debug;
pub mod gda;

/// Commands related to Dutch auctions
#[derive(Debug, Subcommand)]
pub enum DutchCmd {
    /// Schedule a gradual dutch auction, a prototype for penumbra developers.
    #[clap(display_order = 1000, name = "gradual")]
    DutchAuctionGradualSchedule {
        /// Source account initiating the auction.
        #[clap(long, display_order = 100, default_value = "0")]
        source: u32,
        /// The value the seller wishes to auction.
        #[clap(long, display_order = 200)]
        input: String,
        /// The maximum output the seller can receive.
        ///
        /// This implicitly defines the starting price for the auction.
        #[clap(long, display_order = 400)]
        max_output: String,
        /// The minimum output the seller is willing to receive.
        ///
        /// This implicitly defines the ending price for the auction.
        #[clap(long, display_order = 500)]
        min_output: String,
        /// The duration for the auction
        #[clap(arg_enum, long, display_order = 600, name = "duration")]
        recipe: gda::GdaRecipe,
        /// Skip asking for confirmation, pay any fees, and execute the transaction.
        #[clap(long, display_order = 700)]
        yes: bool,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t, display_order = 1000)]
        fee_tier: FeeTier,
        #[clap(long, hide = true)]
        // Use to produce a debug file for offline analysis.
        debug: bool,
    },
    /// Schedule a Dutch auction, a tool to help accomplish price discovery.
    #[clap(display_order = 100, name = "schedule")]
    DutchAuctionSchedule {
        /// Source account initiating the auction.
        #[clap(long, display_order = 100, default_value = "0")]
        source: u32,
        /// The value the seller wishes to auction.
        #[clap(long, display_order = 200)]
        input: String,
        /// The maximum output the seller can receive.
        ///
        /// This implicitly defines the starting price for the auction.
        #[clap(long, display_order = 400)]
        max_output: String,
        /// The minimum output the seller is willing to receive.
        ///
        /// This implicitly defines the ending price for the auction.
        #[clap(long, display_order = 500)]
        min_output: String,
        /// The block height at which the auction begins.
        ///
        /// This allows the seller to schedule an auction at a future time.
        #[clap(long, display_order = 600)]
        start_height: u64,
        /// The block height at which the auction ends.
        ///
        /// Together with `start_height`, `max_output`, and `min_output`,
        /// this implicitly defines the speed of the auction.
        #[clap(long, display_order = 700)]
        end_height: u64,
        /// The number of discrete price steps to use for the auction.
        ///
        /// `end_height - start_height` must be a multiple of `step_count`.
        #[clap(long, display_order = 800)]
        step_count: u64,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t, display_order = 1000)]
        fee_tier: FeeTier,
    },
    /// Terminate a Dutch auction.
    #[clap(display_order = 300, name = "end")]
    DutchAuctionEnd {
        /// Source account terminating the auction.
        #[clap(long, display_order = 100, default_value = "0")]
        source: u32,
        /// Identifier of the auction.
        #[clap(long, display_order = 200)]
        auction_id: String,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t, display_order = 300)]
        fee_tier: FeeTier,
    },
    /// Withdraw a Dutch auction, and claim its reserves.
    #[clap(display_order = 200, name = "withdraw")]
    DutchAuctionWithdraw {
        /// Source account withdrawing from the auction.
        #[clap(long, display_order = 100)]
        source: u32,
        /// The auction to withdraw funds from.
        #[clap(long, display_order = 200)]
        auction_id: String,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t, display_order = 600)]
        fee_tier: FeeTier,
    },
}

impl DutchCmd {
    /// Process the command by performing the appropriate action.
    pub async fn exec(&self, app: &mut App) -> anyhow::Result<()> {
        let gas_prices = app
            .view
            .as_mut()
            .context("view service must be initialized")?
            .gas_prices(GasPricesRequest {})
            .await?
            .into_inner()
            .gas_prices
            .expect("gas prices must be available")
            .try_into()?;

        match self {
            DutchCmd::DutchAuctionSchedule {
                source,
                input,
                max_output,
                min_output,
                start_height,
                end_height,
                step_count,
                fee_tier,
            } => {
                let mut nonce = [0u8; 32];
                OsRng.fill_bytes(&mut nonce);

                let input = input.parse::<Value>()?;
                let max_output = max_output.parse::<Value>()?;
                let min_output = min_output.parse::<Value>()?;
                let output_id = max_output.asset_id;

                let plan = Planner::new(OsRng)
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into())
                    .dutch_auction_schedule(DutchAuctionDescription {
                        input,
                        output_id,
                        max_output: max_output.amount,
                        min_output: min_output.amount,
                        start_height: *start_height,
                        end_height: *end_height,
                        step_count: *step_count,
                        nonce,
                    })
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await
                    .context("can't build send transaction")?;
                app.build_and_submit_transaction(plan).await?;
                Ok(())
            }
            DutchCmd::DutchAuctionEnd {
                auction_id,
                source,
                fee_tier,
            } => {
                let auction_id = auction_id.parse::<AuctionId>()?;

                let plan = Planner::new(OsRng)
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into())
                    .dutch_auction_end(auction_id)
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await
                    .context("can't build send transaction")?;
                app.build_and_submit_transaction(plan).await?;
                Ok(())
            }
            DutchCmd::DutchAuctionWithdraw {
                source,
                auction_id,
                fee_tier,
            } => {
                let auction_id = auction_id.parse::<AuctionId>()?;

                use pbjson_types::Any;
                let view_client = app.view();
                let (auction_id, _, auction_raw, _): (
                    AuctionId,
                    SpendableNoteRecord,
                    Option<Any>,
                    Vec<Position>,
                ) = view_client
                    .auctions(None, true, true)
                    .await?
                    .into_iter()
                    .find(|(id, _, _, _)| &auction_id == id)
                    .ok_or_else(|| anyhow!("the auction id is unknown from the view service!"))?;

                let Some(raw_da_state) = auction_raw else {
                    bail!("auction state is missing from view server response")
                };

                use penumbra_proto::core::component::auction::v1 as pb_auction;
                // We're processing a Dutch auction:
                assert_eq!(raw_da_state.type_url, pb_auction::DutchAuction::type_url());

                let dutch_auction = DutchAuction::decode(raw_da_state.value)?;

                let reserves_input = Value {
                    amount: dutch_auction.state.input_reserves,
                    asset_id: dutch_auction.description.input.asset_id,
                };
                let reserves_output = Value {
                    amount: dutch_auction.state.output_reserves,
                    asset_id: dutch_auction.description.output_id,
                };
                let seq = dutch_auction.state.sequence + 1;

                let mut planner = Planner::new(OsRng);

                let plan = planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into())
                    .dutch_auction_withdraw(ActionDutchAuctionWithdrawPlan {
                        auction_id,
                        seq,
                        reserves_input,
                        reserves_output,
                    })
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await
                    .context("can't build send transaction")?;
                app.build_and_submit_transaction(plan).await?;
                Ok(())
            }
            DutchCmd::DutchAuctionGradualSchedule {
                source,
                input: input_str,
                max_output: max_output_str,
                min_output: min_output_str,
                recipe: duration,
                yes,
                fee_tier,
                debug,
            } => {
                println!("Gradual dutch auction prototype");

                let input = input_str.parse::<Value>()?;
                let max_output = max_output_str.parse::<Value>()?;
                let min_output = min_output_str.parse::<Value>()?;

                let asset_cache = app.view().assets().await?;
                let current_height = app.view().status().await?.full_sync_height;

                let gda = gda::GradualAuction::new(
                    input,
                    max_output,
                    min_output,
                    duration.clone(),
                    current_height,
                );

                let auction_descriptions = gda.generate_auctions();

                let input_fmt = input.format(&asset_cache);
                let max_output_fmt = max_output.format(&asset_cache);
                let min_output_fmt = min_output.format(&asset_cache);

                println!("total to auction: {input_fmt}");
                println!("start price: {max_output_fmt}");
                println!("end price: {min_output_fmt}");
                display_auction_description(&asset_cache, auction_descriptions.clone());

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                for description in &auction_descriptions {
                    planner.dutch_auction_schedule(description.clone());
                }

                if *debug {
                    let debug_data_path = Path::new("gda-debug-definition-data.json");
                    let auction_data_path = Path::new("gda-debug-auction-data.json");

                    let gda_debug: DebugGda = gda.into();
                    let gda_debug_data = serde_json::to_string(&gda_debug)?;
                    std::fs::write(debug_data_path, gda_debug_data)?;

                    let gda_auction_data = serde_json::to_string(
                        &auction_descriptions
                            .clone()
                            .into_iter()
                            .map(Into::<debug::DebugDescription>::into)
                            .collect::<Vec<_>>(),
                    )?;
                    std::fs::write(auction_data_path, gda_auction_data)?;
                    tracing::debug!(?debug_data_path, ?auction_data_path, "wrote debug data");
                    return Ok(());
                }

                let plan = planner
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await
                    .context("can't build send transaction")?;

                let tx = app.build_transaction(plan.clone()).await?;
                let fee_fmt = tx
                    .transaction_body
                    .transaction_parameters
                    .fee
                    .0
                    .format(&asset_cache);

                println!("Total fee: {fee_fmt}");

                if !yes {
                    Confirm::new()
                        .with_prompt("Do you wish to proceed")
                        .interact()?;
                }
                app.build_and_submit_transaction(plan).await?;

                Ok(())
            }
        }
    }
}

fn display_auction_description(asset_cache: &Cache, auctions: Vec<DutchAuctionDescription>) {
    let mut tally_max_output = Amount::zero();
    let mut tally_min_output = Amount::zero();
    let mut tally_input = Amount::zero();
    let input_id = auctions[0].input.asset_id;
    let output_id = auctions[0].output_id;

    let mut table = comfy_table::Table::new();
    table.load_preset(presets::NOTHING);

    table.set_header(vec![
        "start",
        "",
        "end",
        "lot",
        "start price for the lot",
        "reserve price for the lot",
    ]);

    for auction in auctions {
        let start_height = auction.start_height;
        let end_height = auction.end_height;
        let input_chunk = Value {
            asset_id: auction.input.asset_id,
            amount: Amount::from(auction.input.amount.value()),
        };

        let max_price = Value {
            asset_id: auction.output_id,
            amount: Amount::from(auction.max_output.value()),
        };

        let min_price = Value {
            asset_id: auction.output_id,
            amount: Amount::from(auction.min_output.value()),
        };

        let max_price_fmt = max_price.format(&asset_cache);
        let min_price_fmt = min_price.format(&asset_cache);

        let input_chunk_fmt = input_chunk.format(&asset_cache);

        tally_input += input_chunk.amount;
        tally_max_output += max_price.amount;
        tally_min_output += min_price.amount;

        table.add_row(vec![
            format!("{start_height}"),
            "--------->".to_string(),
            format!("{end_height}"),
            input_chunk_fmt,
            max_price_fmt,
            min_price_fmt,
        ]);
    }

    println!("{}", table);

    let tally_input_fmt = Value {
        asset_id: input_id,
        amount: tally_input,
    }
    .format(&asset_cache);

    let tally_output_max_fmt = Value {
        asset_id: output_id,
        amount: tally_max_output,
    }
    .format(&asset_cache);

    let tally_output_min_fmt = Value {
        asset_id: output_id,
        amount: tally_min_output,
    }
    .format(&asset_cache);

    println!("Total auctioned: {tally_input_fmt}");
    println!("Total max output: {tally_output_max_fmt}");
    println!("Total min output: {tally_output_min_fmt}");
}
