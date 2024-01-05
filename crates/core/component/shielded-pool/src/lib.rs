#![deny(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg_attr(docsrs, doc(cfg(feature = "component")))]
#[cfg(feature = "component")]
pub mod component;

pub mod ics20_withdrawal;
pub use ics20_withdrawal::Ics20Withdrawal;

pub mod event;
pub mod genesis;
pub mod state_key;

pub mod note;
mod note_payload;
pub mod rseed;

pub use note::{Note, NoteCiphertext, NoteView};
pub use note_payload::NotePayload;
pub use rseed::Rseed;

pub mod convert;
pub mod nullifier_derivation;
pub mod output;
pub mod spend;

pub use convert::{ConvertCircuit, ConvertProof, ConvertProofPrivate, ConvertProofPublic};
pub use nullifier_derivation::{
    NullifierDerivationCircuit, NullifierDerivationProof, NullifierDerivationProofPrivate,
    NullifierDerivationProofPublic,
};
pub use output::{Output, OutputCircuit, OutputPlan, OutputProof, OutputView};
pub use spend::{
    Spend, SpendCircuit, SpendPlan, SpendProof, SpendProofPrivate, SpendProofPublic, SpendView,
};
