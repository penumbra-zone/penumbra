//! State key patterns for the token factory component.

use crate::TokenFactoryId;

/// State key for tracking whether a token factory nonce has been used.
///
/// The presence of this key indicates the nonce has been consumed.
pub fn nonce_used(id: &TokenFactoryId) -> String {
    format!("token_factory/nonce/{}", hex::encode(id.as_bytes()))
}

/// Prefix for all token factory nonce keys.
pub fn all_nonces() -> &'static str {
    "token_factory/nonce/"
}
