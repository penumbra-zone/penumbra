use jmt::WriteOverlay;
use std::sync::Arc;
use tokio::sync::RwLock;

mod metrics;
mod state;
mod storage;

pub use crate::metrics::register_metrics;
pub use storage::Storage;

pub type State = Arc<RwLock<WriteOverlay<Storage>>>;
