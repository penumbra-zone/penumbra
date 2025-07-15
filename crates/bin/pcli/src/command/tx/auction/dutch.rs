use std::path::Path;

use crate::command::tx::FeeTier;
use crate::App;
use anyhow::Result;
use anyhow::{anyhow, bail, Context};
use clap::Subcommand;
use comfy_table::presets;
use dialoguer::Confirm;
use penumbra_sdk_asset::{asset::Cache, Value};
use penumbra_sdk_auction::auction::{
    dutch::DutchAuction, dutch::DutchAuctionDescription, AuctionId,
};
use penumbra_sdk_keys::keys::AddressIndex;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{view::v1::GasPricesRequest, DomainType};
use penumbra_sdk_view::ViewClient;
use penumbra_sdk_wallet::plan::Planner;
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
        #[clap(short, long, default_value_t, display_order = 1000)]
        fee_tier: FeeTier,
        #[clap(long, hide = true)]
        // Use to produce a debug file for numerical analysis.
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
        #[clap(short, long, default_value_t, display_order = 1000)]
        fee_tier: FeeTier,
    },
    /// Terminate a Dutch auction.
    #[clap(display_order = 300, name = "end")]
    DutchAuctionEnd {
        /// Source account terminating the auction.
        #[clap(long, display_order = 100, default_value = "0")]
        source: u32,
        /// If set, ends all auctions owned by the specified account.
        #[clap(long, display_order = 150)]
        all: bool,
        /// Identifiers of the auctions to end, if `--all` is not set.
        #[clap(display_order = 200)]
        auction_ids: Vec<AuctionId>,
        /// Maximum number of auctions to process in a single transaction.
        #[clap(long, default_value = "20", display_order = 250)]
        batch: u8,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t, display_order = 300)]
        fee_tier: FeeTier,
    },
    /// Withdraw a Dutch auction, and claim its reserves.
    #[clap(display_order = 200, name = "withdraw")]
    DutchAuctionWithdraw {
        /// Source account withdrawing from the auction.
        #[clap(long, display_order = 100, default_value = "0")]
        source: u32,
        /// If set, withdraws all auctions owned by the specified account.
        #[clap(long, display_order = 150)]
        all: bool,
        /// Identifiers of the auctions to withdraw, if `--all` is not set.
        #[clap(display_order = 200)]
        auction_ids: Vec<AuctionId>,
        /// Maximum number of auctions to process in a single transaction.
        #[clap(long, default_value = "20", display_order = 250)]
        batch: u8,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t, display_order = 600)]
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
                    .context("can't build auction schedule transaction")?;
                app.build_and_submit_transaction(plan).await?;
                Ok(())
            }
            DutchCmd::DutchAuctionEnd {
                all,
                auction_ids,
                source,
                batch,
                fee_tier,
            } => {
                let auction_ids = match (all, auction_ids.is_empty()) {
                    (true, _) => auctions_to_end(app.view(), *source).await?,
                    (false, false) => auction_ids.to_owned(),
                    (false, true) => {
                        bail!("auction_ids are required when --all is not set")
                    }
                };

                if auction_ids.is_empty() {
                    println!("no active auctions to end");
                    return Ok(());
                }

                // Process auctions in batches
                let batches = auction_ids.chunks(*batch as usize);
                let num_batches = &batches.len();
                for (batch_num, auction_batch) in batches.enumerate() {
                    println!(
                        "processing batch {} of {}, starting with {}",
                        batch_num + 1,
                        num_batches,
                        batch_num * *batch as usize
                    );

                    if auction_batch.is_empty() {
                        continue;
                    }

                    let mut planner = Planner::new(OsRng);
                    planner
                        .set_gas_prices(gas_prices)
                        .set_fee_tier((*fee_tier).into());

                    for auction_id in auction_batch {
                        planner.dutch_auction_end(*auction_id);
                    }

                    let plan = planner
                        .plan(
                            app.view
                                .as_mut()
                                .context("view service must be initialized")?,
                            AddressIndex::new(*source),
                        )
                        .await
                        .context("can't build auction end transaction")?;
                    app.build_and_submit_transaction(plan).await?;
                }
                Ok(())
            }
            DutchCmd::DutchAuctionWithdraw {
                all,
                source,
                auction_ids,
                batch,
                fee_tier,
            } => {
                let auctions = match (all, auction_ids.is_empty()) {
                    (true, _) => auctions_to_withdraw(app.view(), *source).await?,
                    (false, false) => {
                        let all = auctions_to_withdraw(app.view(), *source).await?;
                        let mut selected_auctions = Vec::new();

                        for auction_id in auction_ids {
                            let auction = all
                                .iter()
                                .find(|a| a.description.id() == *auction_id)
                                .ok_or_else(|| {
                                    anyhow!(
                                        "auction id {} is unknown from the view service!",
                                        auction_id
                                    )
                                })?
                                .to_owned();
                            selected_auctions.push(auction);
                        }

                        selected_auctions
                    }
                    (false, true) => {
                        bail!("auction_ids are required when --all is not set")
                    }
                };

                if auctions.is_empty() {
                    println!("no ended auctions to withdraw");
                    return Ok(());
                }

                let batches = auctions.chunks(*batch as usize);
                let num_batches = &batches.len();
                // Process auctions in batches
                for (batch_num, auction_batch) in batches.enumerate() {
                    println!(
                        "processing batch {} of {}, starting with {}",
                        batch_num + 1,
                        num_batches,
                        batch_num * *batch as usize
                    );
                    if auction_batch.is_empty() {
                        continue;
                    }

                    let mut planner = Planner::new(OsRng);
                    planner
                        .set_gas_prices(gas_prices)
                        .set_fee_tier((*fee_tier).into());

                    for auction in auction_batch {
                        planner.dutch_auction_withdraw(auction);
                    }

                    let plan = planner
                        .plan(
                            app.view
                                .as_mut()
                                .context("view service must be initialized")?,
                            AddressIndex::new(*source),
                        )
                        .await
                        .context("can't build auction withdrawal transaction")?;
                    app.build_and_submit_transaction(plan).await?;
                }
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

                    let gda_debug_data = serde_json::to_string(&gda)?;
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

async fn all_dutch_auction_states(
    view_client: &mut impl ViewClient,
    source: impl Into<AddressIndex>,
) -> Result<Vec<(AuctionId, DutchAuction, u64)>> {
    fetch_dutch_auction_states(view_client, source, true).await
}

async fn active_dutch_auction_states(
    view_client: &mut impl ViewClient,
    source: impl Into<AddressIndex>,
) -> Result<Vec<(AuctionId, DutchAuction, u64)>> {
    fetch_dutch_auction_states(view_client, source, false).await
}

async fn fetch_dutch_auction_states(
    view_client: &mut impl ViewClient,
    source: impl Into<AddressIndex>,
    include_inactive: bool,
) -> Result<Vec<(AuctionId, DutchAuction, u64)>> {
    let auctions = view_client
        .auctions(Some(source.into()), include_inactive, true)
        .await?
        .into_iter()
        .filter_map(|(id, _, local_seq, state, _)| {
            if let Some(state) = state {
                if let Ok(da) = DutchAuction::decode(state.value) {
                    Some((id, da, local_seq))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    Ok(auctions)
}
/// Return all the auctions that need to be ended, based on our local view of the chain state.
async fn auctions_to_end(view_client: &mut impl ViewClient, source: u32) -> Result<Vec<AuctionId>> {
    let auctions = active_dutch_auction_states(view_client, source).await?;

    let auction_ids = auctions
        .into_iter()
        .filter_map(|(id, _auction, local_seq)| {
            // We want to end auctions that we track as "opened" (local_seq == 0)
            // so that we can close them, or catch-up with the chain state if they are already closed.
            if local_seq == 0 {
                Some(id)
            } else {
                None
            }
        })
        .collect();

    Ok(auction_ids)
}

async fn auctions_to_withdraw(
    view_client: &mut impl ViewClient,
    source: u32,
) -> Result<Vec<DutchAuction>> {
    let auctions = all_dutch_auction_states(view_client, source).await?;

    let auction_ids = auctions
        .into_iter()
        .filter_map(|(_, auction, local_seq)| {
            // We want to end auctions that we track as "closed" (local_seq == 1)
            // so that we can close them, or catch-up with the chain state if they are already closed.
            if local_seq == 1 {
                Some(auction)
            } else {
                None
            }
        })
        .collect();

    Ok(auction_ids)
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
