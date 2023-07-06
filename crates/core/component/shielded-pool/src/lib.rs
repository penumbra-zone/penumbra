#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub mod component;

pub mod event;
pub mod state_key;

pub mod note;
mod note_payload;
pub mod rseed;
pub mod symmetric;

pub use note::{Note, NoteCiphertext, NoteView};
pub use note_payload::NotePayload;
pub use rseed::Rseed;
pub use symmetric::PayloadKey;

pub mod nullifier_derivation;
pub mod output;
pub mod spend;

pub use nullifier_derivation::{NullifierDerivationCircuit, NullifierDerivationProof};
pub use output::{Output, OutputCircuit, OutputPlan, OutputProof, OutputView};
pub use spend::{Spend, SpendCircuit, SpendPlan, SpendProof, SpendView};
