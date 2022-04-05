use std::sync::Arc;

use jmt::WriteOverlay;
use tokio::sync::Mutex;

use crate::Storage;

mod app;
mod component;
mod ibc;
mod shielded_pool;
mod staking;

// TODO: demote this from `pub` at some point when that's
// not likely to generate conflicts
pub mod validator_set;

pub use self::ibc::IBCComponent;
pub use app::App;
pub use component::Component;
pub use shielded_pool::ShieldedPool;
pub use staking::Staking;

type Overlay = Arc<Mutex<WriteOverlay<Storage>>>;
