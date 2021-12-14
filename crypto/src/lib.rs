pub use decaf377::{FieldExt, Fq, Fr};
pub use decaf377_fmd as fmd;
pub use decaf377_ka as ka;
pub use decaf377_rdsa as rdsa;

pub mod action;
mod address;
pub mod asset;
pub mod keys;
pub mod memo;
pub mod merkle;
pub mod note;
mod nullifier;
mod prf;
pub mod proofs;
pub mod transaction;
pub mod value;

pub use action::{output::Output, spend::Spend, Action};
pub use address::{Address, CURRENT_CHAIN_ID};
pub use note::Note;
pub use nullifier::Nullifier;
pub use transaction::Transaction;
pub use value::Value;

fn fmt_hex<T: AsRef<[u8]>>(data: T, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", hex::encode(data))
}

fn fmt_fq(data: &Fq, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    fmt_hex(&data.to_bytes(), f)
}
