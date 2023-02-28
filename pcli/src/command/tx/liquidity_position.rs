use anyhow::{anyhow, Error, Result};

use penumbra_crypto::{asset, Amount, Value};
use penumbra_transaction::action::{ProposalKind, Vote};

#[derive(Debug, clap::Subcommand)]
pub enum PositionCmd {
    /// Open a new liquidity position based on order details and credits an open position NFT.
    #[clap(display_order = 100, subcommand)]
    Order(OrderCmd),
    /// Debits an opened position NFT and credits a closed position NFT.
    Close {},
    /// Debits a closed position NFT and credits a withdrawn position NFT and the final reserves.
    Withdraw {},
    /// Debits a withdrawn position NFT and credits a claimed position NFT and any liquidity incentives.
    RewardClaim {},
}

impl PositionCmd {
    pub fn offline(&self) -> bool {
        match self {
            PositionCmd::Order(_) => false,
            PositionCmd::Close { .. } => false,
            PositionCmd::Withdraw { .. } => false,
            PositionCmd::RewardClaim { .. } => false,
        }
    }
}

type PurchaseVar = (Value, Value);
/// Turns a string like `100penumbra@1.2gm` into a [`PurchaseVar`] tuple consisting of
/// `(100 penumbra, 1.2 gm)` represented as [`Value`] types.
fn parse_purchase_var(pvar: &str) -> Result<PurchaseVar> {
    if let Some((lhs, rhs)) = pvar.split_once('@') {
        Ok((lhs.parse()?, rhs.parse()?))
    } else {
        Err(anyhow!("invalid argument"))
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum OrderCmd {
    Buy {
        /// The desired purchase, formatted as a string, e.g. `100penumbra@1.2gm` would attempt
        /// to purchase 100 penumbra at a price of 1.2 gm each.
        #[clap(value_parser = clap::builder::ValueParser::new(parse_purchase_var))]
        desired: PurchaseVar,
        /// The fee associated with transactions against the liquidity position.
        #[clap(long, default_value = "0")]
        spread: u128,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
    },
    Sell {
        /// The desired sale, formatted as a string, e.g. `100penumbra@1.2gm` would attempt
        /// to sell 100 penumbra at a price of 1.2 gm each.
        #[clap(value_parser = clap::builder::ValueParser::new(parse_purchase_var))]
        desired: PurchaseVar,
        /// The fee associated with transactions against the liquidity position.
        #[clap(long, default_value = "0")]
        spread: u64,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
    },
}
