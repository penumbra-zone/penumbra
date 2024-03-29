//! An implementation of [Fuzzy Message Detection][fmd].
//!
//! [fmd]: https://protocol.penumbra.zone/main/crypto/fmd.html
#![deny(clippy::unwrap_used)]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod clue;
mod clue_key;
mod detection;
mod error;
mod hash;
mod hkd;
mod precision;

pub use clue::Clue;
pub use clue_key::{ClueKey, ExpandedClueKey};
pub use detection::DetectionKey;
pub use error::Error;
pub use precision::Precision;

pub(crate) use precision::MAX_PRECISION;
