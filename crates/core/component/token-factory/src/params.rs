//! Governance-controlled parameters for the token factory.
//!
//! Both switches default to `false`, so the component ships **dormant**: the
//! consensus code can be activated (app upgrade) without enabling any token
//! creation. Each capability is then turned on independently by governance —
//! fair-launch (fixed supply, no rug) first, and the mintable / unlimited-supply
//! path as a separate, later decision.

use penumbra_sdk_proto::{core::component::token_factory::v1 as pb, DomainType};
use serde::{Deserialize, Serialize};

/// Parameters controlling which token-factory capabilities are enabled.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::TokenFactoryParameters", into = "pb::TokenFactoryParameters")]
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

impl DomainType for TokenFactoryParameters {
    type Proto = pb::TokenFactoryParameters;
}

impl TryFrom<pb::TokenFactoryParameters> for TokenFactoryParameters {
    type Error = anyhow::Error;

    fn try_from(proto: pb::TokenFactoryParameters) -> anyhow::Result<Self> {
        Ok(Self {
            token_factory_enabled: proto.token_factory_enabled,
            mintable_enabled: proto.mintable_enabled,
        })
    }
}

impl From<TokenFactoryParameters> for pb::TokenFactoryParameters {
    fn from(params: TokenFactoryParameters) -> Self {
        Self {
            token_factory_enabled: params.token_factory_enabled,
            mintable_enabled: params.mintable_enabled,
        }
    }
}
