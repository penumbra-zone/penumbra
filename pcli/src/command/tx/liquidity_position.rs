use std::str::FromStr;

use anyhow::{anyhow, Result};

use penumbra_crypto::{dex::lp::position, Value};

#[derive(Debug, clap::Subcommand)]
pub enum PositionCmd {
    /// Open a new liquidity position based on order details and credits an open position NFT.
    #[clap(display_order = 100, subcommand)]
    Order(OrderCmd),
    /// Debits an opened position NFT and credits a closed position NFT.
    Close {
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
        /// The [`position::Id`] of the position to close.
        position_id: position::Id,
    },
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

/// Expresses the desire to buy `desired` units at price `price`.
#[derive(Clone, Debug)]
pub struct BuyOrder {
    pub desired: Value,
    pub price: Value,
}

/// Expresses the desire to sell `selling` units at price `price`.
#[derive(Clone, Debug)]
pub struct SellOrder {
    pub selling: Value,
    pub price: Value,
}

/// Turns a string like `100penumbra@1.2gm` into a [`PurchaseVar`] tuple consisting of
/// `(100 penumbra, 1.2 gm)` represented as [`Value`] types.
impl FromStr for BuyOrder {
    type Err = anyhow::Error;

    fn from_str(pvar: &str) -> Result<Self> {
        if let Some((desired, price)) = pvar.split_once('@').map(|(d, p)| (d.parse(), p.parse())) {
            Ok(BuyOrder {
                desired: desired?,
                price: price?,
            })
        } else {
            Err(anyhow!("invalid argument"))
        }
    }
}

/// Turns a string like `100penumbra@1.2gm` into a [`PurchaseVar`] tuple consisting of
/// `(100 penumbra, 1.2 gm)` represented as [`Value`] types.
impl FromStr for SellOrder {
    type Err = anyhow::Error;

    fn from_str(pvar: &str) -> Result<Self> {
        if let Some((selling, price)) = pvar.split_once('@').map(|(d, p)| (d.parse(), p.parse())) {
            Ok(SellOrder {
                selling: selling?,
                price: price?,
            })
        } else {
            Err(anyhow!("invalid argument"))
        }
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum OrderCmd {
    Buy {
        /// The desired purchase, formatted as a string, e.g. `100penumbra@1.2gm` would attempt
        /// to purchase 100 penumbra at a price of 1.2 gm each.
        buy_order: BuyOrder,
        /// The fee associated with transactions against the liquidity position.
        #[clap(long, default_value = "0")]
        spread: u32,
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
        sell_order: SellOrder,
        /// The fee associated with transactions against the liquidity position.
        #[clap(long, default_value = "0")]
        spread: u32,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
    },
}
