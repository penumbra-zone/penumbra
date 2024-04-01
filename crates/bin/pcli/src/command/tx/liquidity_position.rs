use anyhow::Result;

use penumbra_asset::asset;
use penumbra_dex::{
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
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
        /// Only close positions for the given trading pair.
        #[clap(long)]
        trading_pair: Option<TradingPair>,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Debits an opened position NFT and credits a closed position NFT.
    Close {
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
        /// The [`position::Id`] of the position to close.
        position_id: position::Id,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t)]
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
        #[clap(short, long, value_enum, default_value_t)]
        fee_tier: FeeTier,
    },
    /// Debits a closed position NFT and credits a withdrawn position NFT and the final reserves.
    Withdraw {
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
        /// The [`position::Id`] of the position to withdraw.
        position_id: position::Id,
        /// The selected fee tier to multiply the fee amount by.
        #[clap(short, long, value_enum, default_value_t)]
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
        #[clap(short, long, value_enum, default_value_t)]
        fee_tier: FeeTier,
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
        #[clap(short, long, value_enum, default_value_t)]
        fee_tier: FeeTier,
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

    pub fn as_position<R: CryptoRngCore>(
        &self,
        // Preserved since we'll need it after denom metadata refactor
        _asset_cache: &asset::Cache,
        rng: R,
    ) -> Result<Position> {
        let mut position = match self {
            OrderCmd::Buy { buy_order, .. } => {
                tracing::info!(?buy_order, "parsing buy order");
                let order = BuyOrder::parse_str(buy_order)?;
                order.into_position(rng)
            }
            OrderCmd::Sell { sell_order, .. } => {
                tracing::info!(?sell_order, "parsing sell order");
                let order = SellOrder::parse_str(sell_order)?;
                order.into_position(rng)
            }
        };
        tracing::info!(?position);

        if self.is_auto_closing() {
            position.close_on_fill = true;
        }

        Ok(position)
    }
}
