use penumbra_crypto::{asset, Value};
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
            PositionCmd::Open { .. } => false,
            PositionCmd::Close { .. } => false,
            PositionCmd::Withdraw { .. } => false,
            PositionCmd::RewardClaim { .. } => false,
        }
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum OrderCmd {
    Buy {
        /// The target amount of reserves of the desired asset to attempt to purchase.
        desired_purchase: Value,
        /// The maximum price to pay for the desired asset.
        desired_price: Amount,
        /// The fee associated with transactions against the liquidity position.
        #[clap(long, default_value = "0")]
        lp_fee: u64,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
    },
    Sell {
        /// The reserves to attempt to sell.
        reserves: Value,
        /// The minimum price to sell per unit of reserves.
        desired_price: Amount,
        /// The fee associated with transactions against the liquidity position.
        #[clap(long, default_value = "0")]
        lp_fee: u64,
        /// The transaction fee (paid in upenumbra).
        #[clap(long, default_value = "0")]
        fee: u64,
        /// Only spend funds originally received by the given address index.
        #[clap(long, default_value = "0")]
        source: u32,
    },
}
