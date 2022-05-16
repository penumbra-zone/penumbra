// Required to ensure that Rust can infer a Send bound inside the TCT
#![recursion_limit = "256"]

use std::sync::Arc;

use jmt::WriteOverlay;
use tokio::sync::RwLock;

mod metrics;
mod overlay_ext;
mod storage;

pub use crate::metrics::register_metrics;
pub use overlay_ext::StateExt;
pub use storage::Storage;

pub type State = Arc<RwLock<WriteOverlay<Storage>>>;
