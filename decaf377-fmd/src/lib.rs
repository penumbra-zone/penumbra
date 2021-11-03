//! An implementation of [Fuzzy Message Detection][fmd].
//!
//! [fmd]: https://protocol.penumbra.zone/main/crypto/fmd.html

mod clue_key;
mod detection;
mod error;
mod hash;
mod hkd;

pub use clue_key::{ClueKey, ExpandedClueKey};
pub use detection::DetectionKey;
pub use error::Error;

/// A clue that allows probabilistic message detection.
pub struct Clue(pub [u8; 68]);

/// The maximum detection precision, chosen so that the message bits fit in 3 bytes.
pub const MAX_PRECISION: usize = 24;
