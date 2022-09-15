#![allow(clippy::clone_on_copy)]
pub use ark_ff::{One, Zero};
pub use decaf377::{FieldExt, Fq, Fr};
pub use decaf377_fmd as fmd;
pub use decaf377_ka as ka;
pub use decaf377_rdsa as rdsa;

mod address;
pub mod asset;
pub mod balance;
mod delegation_token;
pub mod dex;
pub mod eddy;
mod flow;
mod governance_key;
mod identity_key;
pub mod keys;
pub mod memo;
pub mod note;
mod note_payload;
mod nullifier;
mod prf;
pub mod proofs;
pub mod symmetric;
pub mod transaction;
pub mod value;

pub use address::Address;
pub use asset::Asset;
pub use delegation_token::DelegationToken;
pub use flow::{MockFlowCiphertext, SwapFlow};
pub use governance_key::GovernanceKey;
pub use identity_key::IdentityKey;
pub use keys::FullViewingKey;
pub use note::Note;
pub use note_payload::NotePayload;
pub use nullifier::Nullifier;
pub use symmetric::PayloadKey;
pub use value::Value;

fn fmt_hex<T: AsRef<[u8]>>(data: T, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", hex::encode(data))
}

use once_cell::sync::Lazy;

pub static STAKING_TOKEN_DENOM: Lazy<asset::Denom> =
    Lazy::new(|| asset::REGISTRY.parse_denom("upenumbra").unwrap());
pub static STAKING_TOKEN_ASSET_ID: Lazy<asset::Id> = Lazy::new(|| STAKING_TOKEN_DENOM.id());
