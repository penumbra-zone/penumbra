#![allow(clippy::clone_on_copy)]
pub use ark_ff::{One, Zero};
pub use decaf377::{FieldExt, Fq, Fr};
pub use decaf377_fmd as fmd;
pub use decaf377_ka as ka;
pub use decaf377_rdsa as rdsa;

mod address;
mod effect_hash;
pub mod keys;
pub mod note;
mod note_payload;
mod nullifier;
mod prf;
pub mod proofs;
pub mod rseed;
pub mod symmetric;
mod transaction;

pub use address::{Address, AddressVar, AddressView};
pub use effect_hash::{EffectHash, EffectingData};
pub use keys::FullViewingKey;
pub use note::{Note, NoteCiphertext, NoteView};
pub use note_payload::NotePayload;
pub use nullifier::{Nullifier, NullifierVar};
pub use rseed::Rseed;
pub use symmetric::PayloadKey;
pub use transaction::TransactionContext;

fn fmt_hex<T: AsRef<[u8]>>(data: T, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", hex::encode(data))
}
