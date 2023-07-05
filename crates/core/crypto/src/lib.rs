#![allow(clippy::clone_on_copy)]
pub use ark_ff::{One, Zero};
pub use decaf377::{FieldExt, Fq, Fr};
pub use decaf377_fmd as fmd;
pub use decaf377_ka as ka;
pub use decaf377_rdsa as rdsa;

mod effect_hash;
pub mod note;
mod note_payload;
mod nullifier;
pub mod rseed;
mod transaction;

pub use effect_hash::{EffectHash, EffectingData};
pub use note::{Note, NoteCiphertext, NoteView};
pub use note_payload::NotePayload;
pub use nullifier::{Nullifier, NullifierVar};
pub use rseed::Rseed;
pub use transaction::TransactionContext;

pub mod symmetric;
pub use symmetric::PayloadKey;
