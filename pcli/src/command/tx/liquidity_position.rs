use penumbra_transaction::action::{ProposalKind, Vote};

#[derive(Debug, clap::Subcommand)]
pub enum LiquidityPositionCmd {
    /// Open a new liquidity position and credits an open position NFT.
    Open {},
    /// Debits an opened position NFT and credits a closed position NFT.
    Close {},
    /// Debits a closed position NFT and credits a withdrawn position NFT and the final reserves.
    Withdraw {},
    /// Debits a withdrawn position NFT and credits a claimed position NFT and any liquidity incentives.
    RewardClaim {},
}

impl LiquidityPositionCmd {
    pub fn offline(&self) -> bool {
        match self {
            LiquidityPositionCmd::Open { .. } => false,
            LiquidityPositionCmd::Close { .. } => false,
            LiquidityPositionCmd::Withdraw { .. } => false,
            LiquidityPositionCmd::RewardClaim { .. } => false,
        }
    }
}
