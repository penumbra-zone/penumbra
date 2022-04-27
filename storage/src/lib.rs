use std::sync::Arc;

use jmt::WriteOverlay;
use tokio::sync::Mutex;

mod overlay_ext;
mod storage;

pub use overlay_ext::OverlayExt;
pub use storage::Storage;

pub type Overlay = Arc<Mutex<WriteOverlay<Storage>>>;
