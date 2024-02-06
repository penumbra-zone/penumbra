pub mod action_handler;
pub mod metrics;
pub mod rpc;
pub mod stake;
pub mod validator_handler;

mod epoch_handler;

use once_cell::sync::Lazy;
pub use stake::Staking;
// Max validator power is 1152921504606846975 (i64::MAX / 8)
// https://github.com/tendermint/tendermint/blob/master/types/validator_set.go#L25
pub const MAX_VOTING_POWER: u128 = 1152921504606846975;
pub const FP_SCALING_FACTOR: Lazy<penumbra_num::fixpoint::U128x128> =
    Lazy::new(|| 1_0000_0000u128.into());

pub use self::metrics::register_metrics;
pub use stake::{ConsensusIndexRead, SlashingData};
pub use stake::{StateReadExt, StateWriteExt};
