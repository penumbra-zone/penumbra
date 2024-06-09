use crate::command::tx::auction::dutch::DutchCmd;
use clap::Subcommand;

pub mod dutch;

#[derive(Debug, Subcommand)]
pub enum AuctionCmd {
    /// Commands related to Dutch auctions
    #[clap(display_order = 100, subcommand)]
    Dutch(DutchCmd),
}
