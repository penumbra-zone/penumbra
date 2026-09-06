//! State key patterns for the token factory component.

use crate::TokenFactoryId;

/// State key for the master enable switch (governance-controlled).
pub fn param_enabled() -> &'static str {
    "token_factory/parameters/token_factory_enabled"
}

/// State key for the mintable-tokens enable switch (governance-controlled).
pub fn param_mintable_enabled() -> &'static str {
    "token_factory/parameters/mintable_enabled"
}

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

/// State key for storing token metadata.
pub fn token_metadata(id: &TokenFactoryId) -> String {
    format!("token_factory/metadata/{}", hex::encode(id.as_bytes()))
}

/// Prefix for all token metadata keys.
pub fn all_metadata() -> &'static str {
    "token_factory/metadata/"
}
