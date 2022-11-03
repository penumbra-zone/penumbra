use jmt::WriteOverlay;
use std::sync::Arc;
use tokio::sync::RwLock;

mod app_hash;
mod metrics;
mod overlay_ext;
mod storage;

pub use crate::metrics::register_metrics;
pub use app_hash::{get_with_proof, AppHash, PENUMBRA_COMMITMENT_PREFIX, PENUMBRA_PROOF_SPECS};
pub use overlay_ext::StateExt;
pub use storage::Storage;

pub type State = Arc<RwLock<WriteOverlay<Storage>>>;
