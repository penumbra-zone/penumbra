use crate::App;
use anyhow::Context;
use clap::Subcommand;
use penumbra_asset::{asset, Value};
use penumbra_auction::auction::AuctionId;
use penumbra_keys::keys::AddressIndex;
use penumbra_num::Amount;
use penumbra_proto::view::v1::GasPricesRequest;
use penumbra_view::ViewClient;
use penumbra_wallet::plan::Planner;
use rand_core::OsRng;

#[derive(Debug, Subcommand)]
pub enum AuctionCmd {
    /// Commands related to Dutch auctions
    #[clap(display_order = 100, subcommand)]
    Dutch(DutchCmd),
}

/// Commands related to querying Dutch auctions.
#[derive(Debug, Subcommand)]
pub enum DutchCmd {
    /// Withdraws the reserves of the Dutch auction.
    #[clap(display_order = 100, name = "id")]
    DutchAuctionQueryId {
        /// The auction to withdraw funds from.
        #[clap(long, display_order = 100)]
        auction_id: String,
    },
}

impl DutchCmd {
    /// Process the command by performing the appropriate action.
    pub async fn exec(&self, app: &mut App) -> anyhow::Result<()> {
        match self {
            DutchCmd::DutchAuctionQueryId { auction_id } => {
                let auction_id = auction_id.parse::<AuctionId>()?;

                // Query stateful auction information from view server
                let client = app.view().auctions_by_id(auction_id).await?;

                let dutch_auction = &client[0];

                println!(
                    "Dutch auction description and state: {:?} and {:?}",
                    dutch_auction.description, dutch_auction.state
                );
            }
        }
        Ok(())
    }
}
