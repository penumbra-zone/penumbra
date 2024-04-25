use super::tx::FeeTier;
use crate::App;
use anyhow::{anyhow, Context, Result};
use clap::Subcommand;
use penumbra_asset::{
    asset::{self, Unit, REGISTRY},
    Value,
};
use penumbra_auction::auction::AuctionId;
use penumbra_keys::keys::AddressIndex;
use penumbra_num::Amount;
use penumbra_proto::view::v1::GasPricesRequest;
use penumbra_wallet::plan::Planner;
use rand::{Rng, RngCore};
use rand_core::OsRng;
use regex::Regex;

#[derive(Debug, Subcommand)]
pub enum AuctionCmd {
    /// Commands related to Dutch auctions
    #[clap(display_order = 100, subcommand)]
    Dutch(DutchCmd),
}

/// Commands related to Dutch auctions
#[derive(Debug, Subcommand)]
pub enum DutchCmd {
    /// Schedule a Dutch auction, a tool to help accomplish price discovery.
    #[clap(display_order = 100, name = "schedule")]
    DutchAuctionSchedule {
        /// Source address initiating the auction.
        #[clap(long, display_order = 100)]
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
    /// Withdraws the reserves of the Dutch auction.
    #[clap(display_order = 200, name = "withdraw")]
    DutchAuctionWithdraw {
        /// Source address withdrawing from the auction.
        #[clap(long, display_order = 100)]
        source: u32,
        /// The auction to withdraw funds from.
        #[clap(long, display_order = 200)]
        auction_id: String,
        ///  The sequence number of the withdrawal.
        #[clap(long, display_order = 300)]
        seq: u64,
        /// The amount of the input asset directly owned by the auction.
        ///
        /// The auction may also own the input asset indirectly,
        /// via the reserves of `current_position` if it exists.
        #[clap(long, display_order = 400)]
        reserves_input: String,
        /// The amount of the output asset directly owned by the auction.
        ///
        /// The auction may also own the output asset indirectly,
        /// via the reserves of `current_position` if it exists.
        #[clap(long, display_order = 500)]
        reserves_output: String,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t, display_order = 600)]
        fee_tier: FeeTier,
    },
    /// Ends a Dutch auction.
    #[clap(display_order = 300, name = "end")]
    DutchAuctionEnd {
        /// Source address withdrawing from auction.
        #[clap(long, display_order = 100)]
        source: u32,
        /// Identifier of the auction.
        #[clap(long, display_order = 200)]
        auction_id: String,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t, display_order = 300)]
        fee_tier: FeeTier,
    },
}

fn extract_unit(input: &str) -> Result<Unit> {
    let unit_re = Regex::new(r"[0-9.]+([^0-9.].*+)$")?;
    if let Some(captures) = unit_re.captures(input) {
        let unit = captures.get(1).expect("matched regex").as_str();
        Ok(asset::REGISTRY.parse_unit(unit))
    } else {
        Err(anyhow!("could not extract unit from {}", input))
    }
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

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                planner.dutch_auction_schedule(
                    input,
                    output_id,
                    max_output.amount,
                    min_output.amount,
                    *start_height,
                    *end_height,
                    *step_count,
                    nonce,
                );

                let plan = planner
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

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                planner.dutch_auction_end(auction_id);

                let plan = planner
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
                seq,
                reserves_input,
                reserves_output,
                fee_tier,
            } => {
                let auction_id = auction_id.parse::<AuctionId>()?;
                let reserves_input = reserves_input.parse::<Value>()?;
                let reserves_output = reserves_output.parse::<Value>()?;

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                planner.dutch_auction_withdraw(auction_id, *seq, reserves_input, reserves_output);

                let plan = planner
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
        }
    }
}
