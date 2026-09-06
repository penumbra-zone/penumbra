//! Mint capability - represents the right to mint additional tokens.
//!
//! The mint capability is a unique asset (quantity 1) that grants its holder
//! the ability to mint tokens for a specific token factory. Each mint operation
//! consumes the capability at sequence N and produces a new one at sequence N+1,
//! creating an auditable chain of mints.

use anyhow::{anyhow, Context};
use penumbra_sdk_asset::Value;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{
    penumbra::core::component::token_factory::v1 as pb, DomainType,
};
use serde::{Deserialize, Serialize};

use crate::{asset_id_from_denom, TokenFactoryError, TokenFactoryId};

/// Mint capability for a token factory.
///
/// Holding this asset grants the right to mint tokens. The sequence number
/// increments with each mint, preventing replay and tracking mint history.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MintCapability {
    /// The token factory this capability controls.
    id: TokenFactoryId,
    /// Sequence number, incremented on each mint.
    seq: u64,
}

impl MintCapability {
    /// Create a new mint capability at sequence 0 (initial capability from create).
    pub fn initial(id: TokenFactoryId) -> Self {
        Self { id, seq: 0 }
    }

    /// Create a mint capability with specific sequence number.
    pub fn new(id: TokenFactoryId, seq: u64) -> Self {
        Self { id, seq }
    }

    /// Get the token factory ID.
    pub fn id(&self) -> &TokenFactoryId {
        &self.id
    }

    /// Get the current sequence number.
    pub fn seq(&self) -> u64 {
        self.seq
    }

    /// Advance to the next sequence number.
    ///
    /// Returns an error if the sequence number would overflow.
    pub fn next(&self) -> Result<Self, TokenFactoryError> {
        let next_seq = self
            .seq
            .checked_add(1)
            .ok_or(TokenFactoryError::SequenceOverflow)?;
        Ok(Self {
            id: self.id.clone(),
            seq: next_seq,
        })
    }

    /// Get the denom string for this capability.
    ///
    /// Format: `factory/{hex_id}/mint/{seq}`
    pub fn denom_str(&self) -> String {
        format!(
            "factory/{}/mint/{}",
            hex::encode(self.id.as_bytes()),
            self.seq
        )
    }

    /// Compute the asset ID for this mint capability.
    ///
    /// `expect` is safe here: `TokenFactoryId` is length-validated on
    /// construction, so `denom_str()` is always `factory/{64 hex}/mint/{seq}` and
    /// the derivation cannot fail. Callers with unvalidated input should use the
    /// fallible `asset_id_from_denom` directly. Returns `Id` (not `Result`).
    #[allow(clippy::expect_used)]
    pub fn asset_id(&self) -> penumbra_sdk_asset::asset::Id {
        // Infallible in practice: `TokenFactoryId` is length-validated when it
        // is deserialised, so `denom_str()` is always `factory/{64 hex chars}`.
        // The fallible helper stays fallible for callers with unvalidated input.
        asset_id_from_denom(&self.denom_str()).expect(
            "TokenFactoryId is length-validated on construction (InvalidIdLength), so \
             its derived `factory/{hex}` denom is always well-formed; this cannot fail"
        )
    }

    /// Create a Value representing one unit of this capability.
    pub fn value(&self) -> Value {
        Value {
            asset_id: self.asset_id(),
            amount: Amount::from(1u64),
        }
    }
}

impl DomainType for MintCapability {
    type Proto = pb::MintCapability;
}

impl From<MintCapability> for pb::MintCapability {
    fn from(cap: MintCapability) -> Self {
        Self {
            id: Some(cap.id.into()),
            seq: cap.seq,
        }
    }
}

impl TryFrom<pb::MintCapability> for MintCapability {
    type Error = anyhow::Error;

    fn try_from(proto: pb::MintCapability) -> Result<Self, Self::Error> {
        let id = proto
            .id
            .ok_or_else(|| anyhow!("missing id in MintCapability"))?
            .try_into()
            .context("invalid token factory id")?;
        Ok(Self { id, seq: proto.seq })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_increment() {
        let id = TokenFactoryId::new([0u8; 32]);
        let cap = MintCapability::initial(id);
        assert_eq!(cap.seq(), 0);

        let next = cap.next().unwrap();
        assert_eq!(next.seq(), 1);
    }

    #[test]
    fn test_sequence_overflow() {
        let id = TokenFactoryId::new([0u8; 32]);
        let cap = MintCapability::new(id, u64::MAX);
        assert!(cap.next().is_err());
    }

    #[test]
    fn test_denom_format() {
        let id = TokenFactoryId::new([0xab; 32]);
        let cap = MintCapability::new(id, 42);
        let denom = cap.denom_str();
        assert!(denom.starts_with("factory/"));
        assert!(denom.ends_with("/mint/42"));
    }
}
