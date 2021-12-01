use thiserror::Error;

/// An error in message detection.
#[derive(Clone, Error, Debug)]
pub enum Error {
    /// Clue creation for larger than maximum precision was requested.
    #[error("Precision {0} bigger than MAX_PRECISION")]
    PrecisionTooLarge(usize),
    /// An address encoding was invalid.
    #[error("Invalid address")]
    InvalidAddress,
    /// A detection key encoding was invalid.
    #[error("Invalid detection key")]
    InvalidDetectionKey,
    /// Invalid P encoding
    #[error("Invalid P encoding")]
    InvalidPEncoding,
    /// Invalid Y encoding
    #[error("Invalid Y encoding")]
    InvalidYEncoding,
    /// P cannot be identity and Y cannot be 0
    #[error("P cannot be identity and Y cannot be 0")]
    InvalidPorY,
    /// Invalid if any message bit is 0
    #[error("All message bits should be 1")]
    ZeroMessageBit,
}
