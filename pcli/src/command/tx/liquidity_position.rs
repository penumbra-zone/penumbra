use std::str::FromStr;

use anyhow::{anyhow, Result};

use penumbra_crypto::{
    asset,
    dex::{
        lp::{
            position::{self, Position},
            Reserves,
        },
        DirectedTradingPair,
    },
    fixpoint::U128x128,
    Value,
};
use rand_core::CryptoRngCore;

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
    Withdraw {
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
        /// The [`position::Id`] of the position to withdraw.
        position_id: position::Id,
    },
    /// Debits a withdrawn position NFT and credits a claimed position NFT and any liquidity incentives.
    #[clap(hide(true))] // remove when reward claims exist
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

impl OrderCmd {
    pub fn fee(&self) -> u64 {
        match self {
            OrderCmd::Buy { fee, .. } => *fee,
            OrderCmd::Sell { fee, .. } => *fee,
        }
    }

    pub fn source(&self) -> u32 {
        match self {
            OrderCmd::Buy { source, .. } => *source,
            OrderCmd::Sell { source, .. } => *source,
        }
    }

    pub fn into_position<R: CryptoRngCore>(
        &self,
        asset_cache: &asset::Cache,
        rng: R,
    ) -> Result<Position> {
        let (pair, p, q, reserves) = match self {
            OrderCmd::Buy { buy_order, .. } => {
                let pair =
                    DirectedTradingPair::new(buy_order.price.asset_id, buy_order.desired.asset_id);

                let desired_unit = asset_cache
                    .get(&buy_order.desired.asset_id)
                    .ok_or_else(|| anyhow!("unknown asset {}", buy_order.desired.asset_id))?
                    .default_unit();

                // We're buying 1 unit of the desired asset...
                let p = desired_unit.unit_amount();
                // ... for the given amount of the price asset.
                let q = buy_order.price.amount;

                // we want to end up with (r1, r2) = (0, desired)
                // amm is p * r1 + q * r2 = k
                // => k = q * desired (set r1 = 0)
                // => when r2 = 0, p * r1 = q * desired
                // =>              r1 = q * desired / p
                let q_over_p = (U128x128::from(q) / U128x128::from(p))
                    .ok_or_else(|| anyhow::anyhow!("supplied zero buy price"))?;
                let r1 = (U128x128::from(buy_order.desired.amount) * q_over_p)
                    .ok_or_else(|| anyhow::anyhow!("overflow computing r1"))?
                    .round_up()
                    .try_into()
                    .expect("rounded to integer");

                (
                    pair,
                    p,
                    q,
                    Reserves {
                        r1,
                        r2: 0u64.into(),
                    },
                )
            }
            OrderCmd::Sell { sell_order, .. } => {
                let pair = DirectedTradingPair::new(
                    sell_order.selling.asset_id,
                    sell_order.price.asset_id,
                );

                let selling_unit = asset_cache
                    .get(&sell_order.selling.asset_id)
                    .ok_or_else(|| anyhow!("unknown asset {}", sell_order.selling.asset_id))?
                    .default_unit();

                // We're selling 1 unit of the selling asset...
                let p = selling_unit.unit_amount();
                // ... for the given amount of the price asset.
                let q = sell_order.price.amount;

                (
                    pair,
                    p,
                    q,
                    Reserves {
                        r1: sell_order.selling.amount,
                        r2: 0u64.into(),
                    },
                )
            }
        };

        let spread = match self {
            OrderCmd::Buy { spread, .. } | OrderCmd::Sell { spread, .. } => *spread,
        };
        // `spread` is another name for `fee`, which is at most 5000 bps.
        if spread > 5000 {
            anyhow::bail!("spread parameter must be at most 5000bps (i.e. 50%)");
        }

        Ok(Position::new(rng, pair, spread, p, q, reserves))
    }
}
