mod ciphertext;
mod plaintext;
pub use ciphertext::SwapCiphertext;
pub use plaintext::SwapPlaintext;

use once_cell::sync::Lazy;

// Swap ciphertext byte length
pub const SWAP_CIPHERTEXT_BYTES: usize = 169;
// Swap plaintext byte length
pub const SWAP_LEN_BYTES: usize = 153;
pub const OVK_WRAPPED_LEN_BYTES: usize = 80;

/// The nonce used for swap encryption.
///
/// The nonce will always be `[0u8; 12]` which is okay since we use a new
/// ephemeral key each time.
pub static SWAP_ENCRYPTION_NONCE: Lazy<[u8; 12]> = Lazy::new(|| [0u8; 12]);

// Can add to this/make this an enum when we add additional types of swaps.
// TODO: is this actually something we would do? suppose it doesn't hurt to build this
// in early.
pub const SWAP_TYPE: u8 = 0;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Swap type unsupported")]
    SwapTypeUnsupported,
    #[error("Swap deserialization error")]
    SwapDeserializationError,
    #[error("Decryption error")]
    DecryptionError,
}
