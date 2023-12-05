#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use effect_hash::{EffectHash, EffectingData};
pub use epoch::Epoch;
pub use note_source::{NoteSource, SpendInfo};
pub use transaction::TransactionContext;

mod epoch;
mod note_source;

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub mod component;

pub mod genesis;
pub mod params;
pub mod state_key;

// Located here at the bottom of the dep tree for convenience
mod effect_hash;
mod transaction;
