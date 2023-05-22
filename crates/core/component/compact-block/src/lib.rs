#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub mod component;

pub mod event;
pub mod state_key;

mod compact_block;
mod state_payload;

pub use compact_block::CompactBlock;
pub use state_payload::{StatePayload, StatePayloadDebugKind};
