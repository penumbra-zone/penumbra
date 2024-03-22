pub mod action_handler;
mod epoch_handler;
pub mod metrics;
pub mod rpc;
pub mod stake;
pub mod validator_handler;

pub use self::{
    metrics::register_metrics,
    stake::{ConsensusIndexRead, SlashingData, Staking, StateReadExt, StateWriteExt},
};

// Max validator power is 1152921504606846975 (i64::MAX / 8)
// https://github.com/tendermint/tendermint/blob/master/types/validator_set.go#L25
pub const MAX_VOTING_POWER: u128 = 1152921504606846975;
