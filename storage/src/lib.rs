use std::sync::Arc;

use tokio::sync::Mutex;
use jmt::WriteOverlay;

mod storage;
mod overlay_ext;

pub use storage::Storage;
pub use overlay_ext::OverlayExt;

pub type Overlay = Arc<Mutex<WriteOverlay<Storage>>>;
