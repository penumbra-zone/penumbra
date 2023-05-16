use anyhow::Result;

use penumbra_crypto::{
    asset,
    dex::lp::{
        position::{self, Position},
        BuyOrder, SellOrder,
    },
};
use rand_core::CryptoRngCore;

use super::replicate::ReplicateCmd;

#[derive(Debug, clap::Subcommand)]
pub enum PositionCmd {
    /// Open a new liquidity position based on order details and credits an open position NFT.
    #[clap(display_order = 100, subcommand)]
    Order(OrderCmd),
    /// Debits an all opened position NFTs associated with a specific source and credits closed position NFTs.
    CloseAll {
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
    },
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
    /// Debits all closed position NFTs associated with a specific account and credits withdrawn position NFTs and the final reserves.
    WithdrawAll {
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
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
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
    },
    Sell {
        /// The desired sale, formatted as a string, e.g. `100penumbra@1.2gm` would attempt
        /// to sell 100 penumbra at a price of 1.2 gm per 1penumbra.
        ///
        /// An optional suffix of the form `/10bps` may be added to specify a fee spread for the
        /// resulting position, though this is less useful for buy/sell orders than passive LPs.
        sell_order: String,
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
        // Preserved since we'll need it after denom metadata refactor
        _asset_cache: &asset::Cache,
        rng: R,
    ) -> Result<Position> {
        let position = match self {
            OrderCmd::Buy { buy_order, .. } => {
                tracing::info!(?buy_order, "parsing buy order");
                let order = BuyOrder::parse_str(&buy_order)?;
                order.into_position(rng)
            }
            OrderCmd::Sell { sell_order, .. } => {
                tracing::info!(?sell_order, "parsing sell order");
                let order = SellOrder::parse_str(&sell_order)?;
                order.into_position(rng)
            }
        };
        tracing::info!(?position);

        Ok(position)
    }
}
