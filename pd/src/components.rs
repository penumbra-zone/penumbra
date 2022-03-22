use std::sync::{Arc, Mutex};

use jmt::WriteOverlay;

use crate::Storage;

mod app;
mod component;
mod ibc;
mod shielded_pool;

// TODO: demote this from `pub` at some point when that's
// not likely to generate conflicts
pub mod validator_set;

pub use app::App;
pub use component::Component;
pub use ibc::IBCComponent;
pub use shielded_pool::ShieldedPool;

type Overlay = Arc<Mutex<WriteOverlay<Storage>>>;
