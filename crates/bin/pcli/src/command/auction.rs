use super::tx::FeeTier;
use crate::App;
use anyhow::Context;
use clap::Subcommand;
use penumbra_asset::{asset, Value};
use penumbra_auction::auction::AuctionId;
use penumbra_keys::keys::AddressIndex;
use penumbra_num::Amount;
use penumbra_proto::view::v1::GasPricesRequest;
use penumbra_wallet::plan::Planner;
use rand_core::OsRng;

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
        #[clap(long, default_value = "0", display_order = 100)]
        source: u32,
        /// The value the seller wishes to auction.
        #[clap(long, display_order = 200)]
        input: String,
        /// The asset ID of the target asset the seller wishes to acquire.
        #[clap(long, display_order = 300)]
        output: String,
        /// The maximum output the seller can receive.
        ///
        /// This implicitly defines the starting price for the auction.
        #[clap(long, default_value = "0", display_order = 400)]
        max_output: u64,
        /// The minimum output the seller is willing to receive.
        ///
        /// This implicitly defines the ending price for the auction.
        #[clap(long, default_value = "0", display_order = 500)]
        min_output: u64,
        /// The block height at which the auction begins.
        ///
        /// This allows the seller to schedule an auction at a future time.
        #[clap(long, default_value = "0", display_order = 600)]
        start_height: u64,
        /// The block height at which the auction ends.
        ///
        /// Together with `start_height`, `max_output`, and `min_output`,
        /// this implicitly defines the speed of the auction.
        #[clap(long, default_value = "0", display_order = 700)]
        end_height: u64,
        /// The number of discrete price steps to use for the auction.
        ///
        /// `end_height - start_height` must be a multiple of `step_count`.
        #[clap(long, default_value = "0", display_order = 800)]
        step_count: u64,
        /// A random nonce used to allow identical auctions to have
        /// distinct auction IDs.
        #[clap(long, default_value = "0", display_order = 900)]
        nonce: u64,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t, display_order = 1000)]
        fee_tier: FeeTier,
    },
    /// Withdraws the reserves of the Dutch auction.
    #[clap(display_order = 200, name = "withdraw")]
    DutchAuctionWithdraw {
        /// Source address withdrawing from the auction.
        #[clap(long, default_value = "0", display_order = 100)]
        source: u32,
        /// The auction to withdraw funds from.
        #[clap(long, display_order = 200)]
        auction_id: String,
        ///  The sequence number of the withdrawal.
        #[clap(long, default_value = "0", display_order = 300)]
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
        #[clap(long, default_value = "0", display_order = 100)]
        source: u32,
        /// Identifier of the auction.
        #[clap(long, default_value = "0", display_order = 200)]
        auction_id: String,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t, display_order = 300)]
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
                output,
                max_output,
                min_output,
                start_height,
                end_height,
                step_count,
                nonce: _,
                fee_tier,
            } => {
                let input = input.parse::<Value>()?;
                let output = output.parse::<asset::Id>()?;
                let max_output = Amount::from(*max_output);
                let min_output = Amount::from(*min_output);

                let mut planner = Planner::new(OsRng);
                planner
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into());

                planner.dutch_auction_schedule(
                    input,
                    output,
                    max_output,
                    min_output,
                    *start_height,
                    *end_height,
                    *step_count,
                    [0; 32],
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
        }
    }
}
