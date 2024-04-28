use super::FeeTier;
use crate::App;
use anyhow::{anyhow, bail, Context};
use clap::Subcommand;
use penumbra_asset::Value;
use penumbra_auction::auction::{dutch::DutchAuction, AuctionId};
use penumbra_dex::lp::position::Position;
use penumbra_keys::keys::AddressIndex;
use penumbra_proto::{view::v1::GasPricesRequest, DomainType, Name};
use penumbra_view::SpendableNoteRecord;
use penumbra_wallet::plan::Planner;
use rand::RngCore;
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
        //    ///  The sequence number of the withdrawal.
        //    #[clap(long, display_order = 300)]
        //    seq: u64,
        //    /// The amount of the input asset directly owned by the auction.
        //    ///
        //    /// The auction may also own the input asset indirectly,
        //    /// via the reserves of `current_position` if it exists.
        //    #[clap(long, display_order = 400)]
        //    reserves_input: String,
        //    /// The amount of the output asset directly owned by the auction.
        //    ///
        //    /// The auction may also own the output asset indirectly,
        //    /// via the reserves of `current_position` if it exists.
        //    #[clap(long, display_order = 500)]
        //    reserves_output: String,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t, display_order = 600)]
        fee_tier: FeeTier,
    },
}

impl DutchCmd {
    /// Process the command by performing the appropriate action.
    pub async fn exec(&self, app: &mut App) -> anyhow::Result<()> {
        // let gas_prices = app
        //     .view
        //     .as_mut()
        //     .context("view service must be initialized")?
        //     .gas_prices(GasPricesRequest {})
        //     .await?
        //     .into_inner()
        //     .gas_prices
        //     .expect("gas prices must be available")
        //     .try_into()?;

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
<<<<<<< HEAD:crates/bin/pcli/src/command/auction.rs
                // let input = input.parse::<Value>()?;
                // let output = output.parse::<asset::Id>()?;
                // let max_output = Amount::from(*max_output);
                // let min_output = Amount::from(*min_output);

                // let mut planner = Planner::new(OsRng);
                // planner
                //     .set_gas_prices(gas_prices)
                //     .set_fee_tier((*fee_tier).into());

                // planner.dutch_auction_schedule(
                //     input,
                //     output,
                //     max_output,
                //     min_output,
                //     *start_height,
                //     *end_height,
                //     *step_count,
                //     [0; 32],
                // );

                // let plan = planner
                //     .plan(
                //         app.view
                //             .as_mut()
                //             .context("view service must be initialized")?,
                //         AddressIndex::new(*source),
                //     )
                //     .await
                //     .context("can't build send transaction")?;
                // app.build_and_submit_transaction(plan).await?;
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
                // let auction_id = auction_id.parse::<AuctionId>()?;
                // let reserves_input = reserves_input.parse::<Value>()?;
                // let reserves_output = reserves_output.parse::<Value>()?;

                // let mut planner = Planner::new(OsRng);
                // planner
                //     .set_gas_prices(gas_prices)
                //     .set_fee_tier((*fee_tier).into());

                // planner.dutch_auction_withdraw(auction_id, *seq, reserves_input, reserves_output);

                // let plan = planner
                //     .plan(
                //         app.view
                //             .as_mut()
                //             .context("view service must be initialized")?,
                //         AddressIndex::new(*source),
                //     )
                //     .await
                //     .context("can't build send transaction")?;
                // app.build_and_submit_transaction(plan).await?;
=======
                let mut nonce = [0u8; 32];
                OsRng.fill_bytes(&mut nonce);

                let input = input.parse::<Value>()?;
                let max_output = max_output.parse::<Value>()?;
                let min_output = min_output.parse::<Value>()?;
                let output_id = max_output.asset_id;

                let plan = Planner::new(OsRng)
                    .set_gas_prices(gas_prices)
                    .set_fee_tier((*fee_tier).into())
                    .dutch_auction_schedule(
                        input,
                        output_id,
                        max_output.amount,
                        min_output.amount,
                        *start_height,
                        *end_height,
                        *step_count,
                        nonce,
                    )
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await
                    .context("can't build send transaction")?;
                app.build_and_submit_transaction(plan).await?;
>>>>>>> main:crates/bin/pcli/src/command/tx/auction.rs
                Ok(())
            }
            DutchCmd::DutchAuctionEnd {
                auction_id,
                source,
                fee_tier,
            } => {
                // let auction_id = auction_id.parse::<AuctionId>()?;

<<<<<<< HEAD:crates/bin/pcli/src/command/auction.rs
                // let mut planner = Planner::new(OsRng);
                // planner
                //     .set_gas_prices(gas_prices)
                //     .set_fee_tier((*fee_tier).into());

                // planner.dutch_auction_end(auction_id);

                // let plan = planner
                //     .plan(
                //         app.view
                //             .as_mut()
                //             .context("view service must be initialized")?,
                //         AddressIndex::new(*source),
                //     )
                //     .await
                //     .context("can't build send transaction")?;
                // app.build_and_submit_transaction(plan).await?;
=======
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
                // seq,
                // reserves_input,
                // reserves_output,
                fee_tier,
            } => {
                let auction_id = auction_id.parse::<AuctionId>()?;

                use pbjson_types::Any;
                use penumbra_view::ViewClient;
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

                use penumbra_proto::core::component::auction::v1alpha1 as pb_auction;
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
                    .dutch_auction_withdraw(auction_id, seq, reserves_input, reserves_output)
                    .plan(
                        app.view
                            .as_mut()
                            .context("view service must be initialized")?,
                        AddressIndex::new(*source),
                    )
                    .await
                    .context("can't build send transaction")?;
                app.build_and_submit_transaction(plan).await?;
>>>>>>> main:crates/bin/pcli/src/command/tx/auction.rs
                Ok(())
            }
        }
    }
}
