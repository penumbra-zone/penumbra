use thiserror::Error;

/// An error in message detection.
#[derive(Clone, Error, Debug)]
pub enum Error {
    /// Clue creation for larger than maximum precision was requested.
    #[error("Precision {0} is larger than `MAX_PRECISION` or current key expansion.")]
    PrecisionTooLarge(usize),
    /// An address encoding was invalid.
    #[error("Invalid address.")]
    InvalidAddress,
    /// A detection key encoding was invalid.
    #[error("Invalid detection key.")]
    InvalidDetectionKey,
    /// A clue key encoding was invalid.
    #[error("Invalid clue key.")]
    InvalidClueKey,
}
