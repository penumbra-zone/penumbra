use anyhow::Result;

use penumbra_sdk_asset::asset;
use penumbra_sdk_dex::{
    lp::{
        position::{self, Position},
        BuyOrder, SellOrder,
    },
    TradingPair,
};
use rand_core::CryptoRngCore;

use super::{replicate::ReplicateCmd, FeeTier};

#[derive(Debug, clap::Subcommand)]
pub enum PositionCmd {
    /// Open a new liquidity position based on order details and credits an open position NFT.
    #[clap(display_order = 100, subcommand)]
    Order(OrderCmd),
    /// Debits an all opened position NFTs associated with a specific source and credits closed position NFTs.
    CloseAll {
        /// Only spend fugdnds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
        /// Only close positions for the given trading pair.
        #[clap(long)]
        trading_pair: Option<TradingPair>,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Debits opened position NFTs and credits closed position NFTs.
    Close {
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
        /// The list of [`position::Id`] of the positions to close.
        position_ids: Vec<position::Id>,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Debits all closed position NFTs associated with a specific account and credits withdrawn position NFTs and the final reserves.
    WithdrawAll {
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
        /// Only withdraw positions for the given trading pair.
        #[clap(long)]
        trading_pair: Option<TradingPair>,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Debits closed position NFTs and credits withdrawn position NFTs and the final reserves.
    Withdraw {
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
        /// The list of [`position::Id`] of the positions to withdraw.
        position_ids: Vec<position::Id>,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
    },

    /// Debits a withdrawn position NFT and credits a claimed position NFT and any liquidity incentives.
    #[clap(hide(true))] // remove when reward claims exist
    RewardClaim {},
    /// Replicate a trading function
    #[clap(subcommand)]
    Replicate(ReplicateCmd),
}

impl PositionCmd {
    pub fn offline(&self) -> bool {
        match self {
            PositionCmd::Order(_) => false,
            PositionCmd::Close { .. } => false,
            PositionCmd::CloseAll { .. } => false,
            PositionCmd::Withdraw { .. } => false,
            PositionCmd::WithdrawAll { .. } => false,
            PositionCmd::RewardClaim { .. } => false,
            PositionCmd::Replicate(replicate) => replicate.offline(),
        }
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum OrderCmd {
    Buy {
        /// The desired purchase, formatted as a string, e.g. `100penumbra@1.2gm` would attempt
        /// to purchase 100 penumbra at a price of 1.2 gm per 1penumbra.
        ///
        /// An optional suffix of the form `/10bps` may be added to specify a fee spread for the
        /// resulting position, though this is less useful for buy/sell orders than passive LPs.
        buy_order: String,
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
        /// When set, tags the position as an auto-closing buy.
        #[clap(long)]
        auto_close: bool,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
        /// Duplicate the order for the given number of times.
        #[clap(short, long, default_value = "1")]
        num_copies: u32,
    },
    Sell {
        /// The desired sale, formatted as a string, e.g. `100penumbra@1.2gm` would attempt
        /// to sell 100 penumbra at a price of 1.2 gm per 1penumbra.
        ///
        /// An optional suffix of the form `/10bps` may be added to specify a fee spread for the
        /// resulting position, though this is less useful for buy/sell orders than passive LPs.
        sell_order: String,
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
        /// When set, tags the position as an auto-closing sell.
        #[clap(long)]
        auto_close: bool,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, default_value_t)]
        fee_tier: FeeTier,
        /// Duplicate the order for the given number of times.
        #[clap(short, long, default_value = "1")]
        num_copies: u32,
    },
}

impl OrderCmd {
    pub fn source(&self) -> u32 {
        match self {
            OrderCmd::Buy { source, .. } => *source,
            OrderCmd::Sell { source, .. } => *source,
        }
    }

    pub fn fee_tier(&self) -> FeeTier {
        match self {
            OrderCmd::Buy { fee_tier, .. } => *fee_tier,
            OrderCmd::Sell { fee_tier, .. } => *fee_tier,
        }
    }

    pub fn is_auto_closing(&self) -> bool {
        match self {
            OrderCmd::Buy { auto_close, .. } => *auto_close,
            OrderCmd::Sell { auto_close, .. } => *auto_close,
        }
    }

    pub fn num_copies(&self) -> u32 {
        match self {
            OrderCmd::Buy { num_copies, .. } => *num_copies,
            OrderCmd::Sell { num_copies, .. } => *num_copies,
        }
    }

    pub fn as_position(
        &self,
        // Preserved since we'll need it after denom metadata refactor
        _asset_cache: &asset::Cache,
        mut rng: impl CryptoRngCore,
    ) -> Result<Vec<Position>> {
        let positions = match self {
            OrderCmd::Buy { buy_order, .. } => {
                tracing::info!(?buy_order, "parsing buy order");
                let order = BuyOrder::parse_str(buy_order)?;
                let mut positions = Vec::new();
                for _ in 0..self.num_copies() {
                    let mut position = order.into_position(&mut rng);
                    if self.is_auto_closing() {
                        position.close_on_fill = true;
                    }
                    positions.push(position);
                }
                positions
            }
            OrderCmd::Sell { sell_order, .. } => {
                tracing::info!(?sell_order, "parsing sell order");
                let order = SellOrder::parse_str(sell_order)?;
                let mut positions = Vec::new();

                for _ in 0..self.num_copies() {
                    let mut position = order.into_position(&mut rng);
                    if self.is_auto_closing() {
                        position.close_on_fill = true;
                    }
                    positions.push(position);
                }
                positions
            }
        };

        Ok(positions)
    }
}
