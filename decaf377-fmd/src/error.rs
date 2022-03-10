use std::cell::{BorrowError, BorrowMutError};

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
    /// Wrapper for a runtime internal error.
    #[error("Internal Error.")]
    InternalError(String),
}

impl From<BorrowError> for Error {
    fn from(err: BorrowError) -> Error {
        Error::InternalError(err.to_string())
    }
}

impl From<BorrowMutError> for Error {
    fn from(err: BorrowMutError) -> Error {
        Error::InternalError(err.to_string())
    }
}
