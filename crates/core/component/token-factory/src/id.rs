//! Token factory identifier.
//!
//! The [`TokenFactoryId`] uniquely identifies a token created through the
//! token factory component. It is a 32-byte value that MUST be generated
//! using a cryptographically secure random number generator.
//!
//! # Security Considerations
//!
//! ## Nonce Generation
//!
//! The 32-byte nonce provides 256 bits of entropy, making collisions
//! computationally infeasible when generated correctly. However, if the
//! nonce is predictable (e.g., generated from a weak RNG or derived from
//! public information), an adversary could:
//!
//! - Front-run the token creation by submitting a transaction with the
//!   same nonce before the legitimate creator
//! - Cause token creation to fail if the nonce is already used
//!
//! ## Denom Format
//!
//! The token denom is `factory/{hex_nonce}`, which:
//! - Is guaranteed to be unique per nonce
//! - Does not reveal any information about the creator
//! - Can be parsed back to the TokenFactoryId

use crate::error::TokenFactoryError;
use penumbra_sdk_proto::DomainType;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// Length of a token factory ID in bytes.
pub const TOKEN_FACTORY_ID_LEN: usize = 32;

/// Unique identifier for a token factory token.
///
/// This is a 32-byte nonce that defines the token's denom as `factory/{hex_nonce}`.
///
/// # Generation
///
/// Token factory IDs MUST be generated using [`rand_core::OsRng`] or an
/// equivalent cryptographically secure source. Example:
///
/// ```ignore
/// use rand_core::RngCore;
/// let mut nonce = [0u8; 32];
/// rand_core::OsRng.fill_bytes(&mut nonce);
/// let id = TokenFactoryId::new(nonce);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenFactoryId([u8; TOKEN_FACTORY_ID_LEN]);

impl TokenFactoryId {
    /// Create a new token factory ID from a 32-byte nonce.
    ///
    /// # Security
    ///
    /// The caller is responsible for ensuring the nonce is generated
    /// using a cryptographically secure random number generator.
    /// Predictable nonces can lead to front-running attacks.
    pub fn new(nonce: [u8; TOKEN_FACTORY_ID_LEN]) -> Self {
        Self(nonce)
    }

    /// Get the raw bytes of the ID.
    pub fn as_bytes(&self) -> &[u8; TOKEN_FACTORY_ID_LEN] {
        &self.0
    }

    /// Get the token denom for this factory token.
    ///
    /// Returns a string in the format `factory/{hex_nonce}`.
    pub fn denom(&self) -> String {
        format!("factory/{}", hex::encode(self.0))
    }

    /// Parse a token factory ID from a byte slice.
    ///
    /// # Errors
    ///
    /// Returns [`TokenFactoryError::InvalidIdLength`] if the slice is not
    /// exactly 32 bytes.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, TokenFactoryError> {
        if bytes.len() != TOKEN_FACTORY_ID_LEN {
            return Err(TokenFactoryError::InvalidIdLength(bytes.len()));
        }
        let mut arr = [0u8; TOKEN_FACTORY_ID_LEN];
        arr.copy_from_slice(bytes);
        Ok(Self(arr))
    }
}

impl Display for TokenFactoryId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "factory/{}", hex::encode(self.0))
    }
}

impl FromStr for TokenFactoryId {
    type Err = TokenFactoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Strip the "factory/" prefix if present
        let hex_str = s.strip_prefix("factory/").unwrap_or(s);

        let bytes = hex::decode(hex_str)
            .map_err(|_| TokenFactoryError::MalformedDenom(s.to_string()))?;

        Self::from_slice(&bytes)
    }
}

impl AsRef<[u8]> for TokenFactoryId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

// Protobuf conversion
impl DomainType for TokenFactoryId {
    type Proto = penumbra_sdk_proto::core::component::token_factory::v1::TokenFactoryId;
}

impl From<TokenFactoryId> for penumbra_sdk_proto::core::component::token_factory::v1::TokenFactoryId {
    fn from(id: TokenFactoryId) -> Self {
        Self {
            inner: id.0.to_vec(),
        }
    }
}

impl TryFrom<penumbra_sdk_proto::core::component::token_factory::v1::TokenFactoryId> for TokenFactoryId {
    type Error = TokenFactoryError;

    fn try_from(proto: penumbra_sdk_proto::core::component::token_factory::v1::TokenFactoryId) -> Result<Self, Self::Error> {
        Self::from_slice(&proto.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_roundtrip() {
        let nonce = [0x42u8; 32];
        let id = TokenFactoryId::new(nonce);

        let denom = id.denom();
        let parsed: TokenFactoryId = denom.parse().unwrap();

        assert_eq!(id, parsed);
    }

    #[test]
    fn test_invalid_length() {
        let short = vec![0u8; 16];
        let result = TokenFactoryId::from_slice(&short);
        assert!(matches!(result, Err(TokenFactoryError::InvalidIdLength(16))));
    }

    #[test]
    fn test_denom_format() {
        let id = TokenFactoryId::new([0xab; 32]);
        let denom = id.denom();
        assert!(denom.starts_with("factory/"));
        assert_eq!(denom.len(), 8 + 64); // "factory/" + 64 hex chars
    }
}
