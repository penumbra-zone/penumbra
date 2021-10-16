mod address;
mod detection;
mod error;
mod hash;
mod hkd;

pub use address::{Address, ExpandedAddress};
pub use detection::DetectionKey;
pub use error::Error;

/// A clue that allows probabilistic message detection.
pub struct Clue(pub [u8; 68]);

/// The maximum detection precision, chosen so that the message bits fit in 3 bytes.
pub const MAX_PRECISION: usize = 24;
