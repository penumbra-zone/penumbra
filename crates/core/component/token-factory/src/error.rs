//! Error types for the token factory component.

use thiserror::Error;

/// Errors that can occur during token factory operations.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TokenFactoryError {
    /// The token factory ID has invalid length.
    #[error("invalid token factory ID length: expected 32 bytes, got {0}")]
    InvalidIdLength(usize),

    /// The nonce for this token factory ID is already in use.
    #[error("token factory nonce already used")]
    NonceAlreadyUsed,

    /// The initial supply exceeds the maximum allowed.
    #[error("initial supply {0} exceeds maximum {1}")]
    InitialSupplyTooLarge(u128, u128),

    /// The token factory denom string is malformed.
    #[error("malformed token factory denom: {0}")]
    MalformedDenom(String),

    /// The metadata base denom does not match the token factory ID.
    #[error("metadata base denom mismatch: expected {expected}, got {actual}")]
    MetadataBaseMismatch {
        /// The expected base denom derived from the nonce.
        expected: String,
        /// The actual base denom in the provided metadata.
        actual: String,
    },

    /// The metadata contains fields that exceed length limits.
    #[error("metadata field '{field}' exceeds maximum length of {max_len} bytes")]
    MetadataFieldTooLong {
        /// The name of the field that exceeded the limit.
        field: &'static str,
        /// The maximum allowed length.
        max_len: usize,
    },

    /// The mint capability sequence number would overflow.
    #[error("mint capability sequence number overflow")]
    SequenceOverflow,

    /// The mint amount exceeds the maximum allowed.
    #[error("mint amount {0} exceeds maximum {1}")]
    MintAmountTooLarge(u128, u128),
}
