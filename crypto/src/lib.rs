#![allow(clippy::clone_on_copy)]
pub use ark_ff::{One, Zero};
pub use decaf377::{FieldExt, Fq, Fr};
pub use decaf377_fmd as fmd;
pub use decaf377_ka as ka;
pub use decaf377_rdsa as rdsa;

mod address;
pub mod asset;
pub mod keys;
pub mod memo;
pub mod merkle;
pub mod note;
mod nullifier;
mod prf;
pub mod proofs;
pub mod value;

pub use address::Address;
pub use note::Note;
pub use nullifier::Nullifier;
pub use value::Value;

fn fmt_hex<T: AsRef<[u8]>>(data: T, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", hex::encode(data))
}

fn fmt_fq(data: &Fq, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    fmt_hex(&data.to_bytes(), f)
}
