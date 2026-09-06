//! Governance-controlled parameters for the token factory.
//!
//! Both switches default to `false`, so the component ships **dormant**: the
//! consensus code can be activated (app upgrade) without enabling any token
//! creation. Each capability is then turned on independently by governance —
//! fair-launch (fixed supply, no rug) first, and the mintable / unlimited-supply
//! path as a separate, later decision.

use serde::{Deserialize, Serialize};

/// Parameters controlling which token-factory capabilities are enabled.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenFactoryParameters {
    /// Master switch: whether any token can be created at all. Off at genesis.
    pub token_factory_enabled: bool,
    /// Whether mintable tokens are allowed — i.e. a create with `enable_mint`
    /// (a `MintCapability` is issued, giving unlimited, non-fairlaunch supply).
    /// This also gates the mint action itself, so disabling it via governance
    /// freezes all minting, even for already-issued capabilities. Off at genesis.
    pub mintable_enabled: bool,
}

impl Default for TokenFactoryParameters {
    fn default() -> Self {
        // Dormant at genesis: everything off until governance turns it on.
        Self {
            token_factory_enabled: false,
            mintable_enabled: false,
        }
    }
}
