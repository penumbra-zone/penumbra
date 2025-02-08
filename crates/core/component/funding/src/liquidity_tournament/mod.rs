mod action;
mod plan;
mod view;

pub mod proof;
pub use action::{ActionLiquidityTournamentVote, LiquidityTournamentVoteBody};
pub use plan::ActionLiquidityTournamentVotePlan;
pub use view::ActionLiquidityTournamentVoteView;

/// The maximum number of allowable bytes in the denom string.
pub const LIQUIDITY_TOURNAMENT_VOTE_DENOM_MAX_BYTES: usize = 256;
