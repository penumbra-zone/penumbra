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
}
