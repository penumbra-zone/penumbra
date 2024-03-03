use decaf377::Fq;
use once_cell::sync::Lazy;

mod action;
mod ciphertext;
mod payload;
mod plaintext;
mod plan;
mod view;

pub mod proof;

pub use action::{Body, Swap};
pub use ciphertext::SwapCiphertext;
pub use payload::SwapPayload;
pub use plaintext::{SwapPlaintext, SwapPlaintextVar};
pub use plan::SwapPlan;
pub use view::SwapView;

// Swap ciphertext byte length.
pub const SWAP_CIPHERTEXT_BYTES: usize = 272;
// Swap plaintext byte length.
pub const SWAP_LEN_BYTES: usize = 256;

pub static DOMAIN_SEPARATOR: Lazy<Fq> =
    Lazy::new(|| Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.swap").as_bytes()));
