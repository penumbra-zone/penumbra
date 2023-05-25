#![allow(clippy::clone_on_copy)]
pub use ark_ff::{One, Zero};
use asset::Denom;
pub use decaf377::{FieldExt, Fq, Fr};
pub use decaf377_fmd as fmd;
pub use decaf377_ka as ka;
pub use decaf377_rdsa as rdsa;

mod address;
pub mod asset;
pub mod balance;
pub mod eddy;
mod effect_hash;
mod fee;
pub mod fixpoint;
mod flow;
mod governance;
pub mod keys;
pub mod memo;
pub mod note;
mod note_payload;
mod nullifier;
mod prf;
pub mod proofs;
pub mod rseed;
pub mod stake;
pub mod symmetric;
mod transaction;
pub mod value;

pub use address::{Address, AddressVar, AddressView};
pub use asset::Amount;
pub use balance::Balance;
pub use effect_hash::{EffectHash, EffectingData};
pub use fee::Fee;
pub use flow::{MockFlowCiphertext, SwapFlow};
pub use governance::{
    key::GovernanceKey,
    proposal_nft::{self, ProposalNft},
    voting_receipt_token::{self, VotingReceiptToken},
};
pub use keys::FullViewingKey;
pub use note::{Note, NoteCiphertext, NoteView};
pub use note_payload::NotePayload;
pub use nullifier::{Nullifier, NullifierVar};
pub use rseed::Rseed;
pub use symmetric::PayloadKey;
pub use transaction::TransactionContext;
pub use value::{Value, ValueVar, ValueView};

fn fmt_hex<T: AsRef<[u8]>>(data: T, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", hex::encode(data))
}

use once_cell::sync::Lazy;

pub static STAKING_TOKEN_DENOM: Lazy<asset::DenomMetadata> = Lazy::new(|| {
    asset::DenomMetadata::default_for(&Denom {
        denom: "upenumbra".to_string(),
    })
    .unwrap()
});

pub static STAKING_TOKEN_ASSET_ID: Lazy<asset::Id> = Lazy::new(|| STAKING_TOKEN_DENOM.id());
