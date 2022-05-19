#![allow(clippy::clone_on_copy)]
pub use ark_ff::{One, Zero};
pub use decaf377::{FieldExt, Fq, Fr};
pub use decaf377_fmd as fmd;
pub use decaf377_ka as ka;
pub use decaf377_rdsa as rdsa;

mod address;
pub mod asset;
mod delegation_token;
mod identity_key;
pub mod keys;
pub mod memo;
pub mod note;
mod note_payload;
mod nullifier;
mod prf;
pub mod proofs;
pub mod value;

pub use address::Address;
pub use asset::Asset;
pub use delegation_token::DelegationToken;
pub use identity_key::IdentityKey;
pub use keys::FullViewingKey;
pub use note::Note;
pub use note_payload::NotePayload;
pub use nullifier::Nullifier;
pub use value::Value;

// Temporary for v0 to v1 testnet address migration.
pub use address::parse_v0_testnet_address;

fn fmt_hex<T: AsRef<[u8]>>(data: T, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", hex::encode(data))
}

use once_cell::sync::Lazy;

pub static STAKING_TOKEN_DENOM: Lazy<asset::Denom> =
    Lazy::new(|| asset::REGISTRY.parse_denom("upenumbra").unwrap());
pub static STAKING_TOKEN_ASSET_ID: Lazy<asset::Id> = Lazy::new(|| STAKING_TOKEN_DENOM.id());
