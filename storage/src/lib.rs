use std::sync::Arc;

use jmt::WriteOverlay;
use tokio::sync::Mutex;

mod overlay_ext;
mod storage;

pub use overlay_ext::StateExt;
pub use storage::Storage;

pub type State = Arc<Mutex<WriteOverlay<Storage>>>;
