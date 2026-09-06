//! ActionTokenFactoryCreate - create a token, optionally with minting capability.
//!
//! # Value Balance
//!
//! ```text
//! ┌────────────────────┬──────────────────────────────────────────┐
//! │     Consumed       │              Produced                    │
//! ├────────────────────┼──────────────────────────────────────────┤
//! │      nothing       │  initial supply + (optional) capability  │
//! └────────────────────┴──────────────────────────────────────────┘
//! ```
//!
//! # Token Types
//!
//! - **Fairlaunch** (`enable_mint = false`): Fixed supply, no minting ever.
//! - **Mintable** (`enable_mint = true`): Also outputs MintCapability(seq=0).
//!
//! # Metadata Validation
//!
//! The metadata's `base` field MUST match the denom derived from the nonce
//! (`factory/{hex_nonce}`). This prevents impersonation attacks.

use crate::{
    asset_id_from_denom, error::TokenFactoryError, MintCapability, TokenFactoryId,
    MAX_INITIAL_SUPPLY,
};
use anyhow::anyhow;
use penumbra_sdk_asset::{asset, Balance, Value};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{core::component::token_factory::v1 as pb, DomainType};
use penumbra_sdk_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};

/// Maximum length for token name in bytes.
pub const MAX_NAME_LENGTH: usize = 128;

/// Maximum length for token symbol in bytes.
pub const MAX_SYMBOL_LENGTH: usize = 32;

/// Maximum length for token description in bytes.
pub const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// Action to create a new token.
///
/// Creates a token with the specified initial supply. If `enable_mint` is true,
/// also outputs a MintCapability that allows minting additional tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ActionTokenFactoryCreate",
    into = "pb::ActionTokenFactoryCreate"
)]
pub struct ActionTokenFactoryCreate {
    /// Random nonce defining the token ID.
    pub nonce: TokenFactoryId,
    /// Token metadata.
    pub metadata: asset::Metadata,
    /// Initial supply for this token (can be 0 for bridge tokens).
    pub initial_supply: Amount,
    /// If true, also outputs a MintCapability for future minting.
    pub enable_mint: bool,
}

impl ActionTokenFactoryCreate {
    /// Create a new token creation action.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The initial supply exceeds MAX_INITIAL_SUPPLY
    /// - The metadata's base denom doesn't match the nonce
    /// - Any metadata field exceeds its length limit
    pub fn new(
        nonce: TokenFactoryId,
        metadata: asset::Metadata,
        initial_supply: Amount,
        enable_mint: bool,
    ) -> Result<Self, TokenFactoryError> {
        // Validate initial supply
        if initial_supply.value() > MAX_INITIAL_SUPPLY {
            return Err(TokenFactoryError::InitialSupplyTooLarge(
                initial_supply.value(),
                MAX_INITIAL_SUPPLY,
            ));
        }

        // Validate metadata base matches nonce
        let expected_base = nonce.denom();
        let actual_base = metadata.base_denom().denom.clone();
        if actual_base != expected_base {
            return Err(TokenFactoryError::MetadataBaseMismatch {
                expected: expected_base,
                actual: actual_base,
            });
        }

        // Validate metadata field lengths
        Self::validate_metadata_lengths(&metadata)?;

        Ok(Self {
            nonce,
            metadata,
            initial_supply,
            enable_mint,
        })
    }

    /// Create a fairlaunch token (fixed supply, no minting).
    pub fn fairlaunch(
        nonce: TokenFactoryId,
        metadata: asset::Metadata,
        initial_supply: Amount,
    ) -> Result<Self, TokenFactoryError> {
        Self::new(nonce, metadata, initial_supply, false)
    }

    /// Create a mintable token (for bridges, etc.).
    pub fn mintable(
        nonce: TokenFactoryId,
        metadata: asset::Metadata,
        initial_supply: Amount,
    ) -> Result<Self, TokenFactoryError> {
        Self::new(nonce, metadata, initial_supply, true)
    }

    /// Validate that metadata fields don't exceed length limits.
    fn validate_metadata_lengths(metadata: &asset::Metadata) -> Result<(), TokenFactoryError> {
        let proto: penumbra_sdk_proto::core::asset::v1::Metadata = metadata.clone().into();

        if proto.name.len() > MAX_NAME_LENGTH {
            return Err(TokenFactoryError::MetadataFieldTooLong {
                field: "name",
                max_len: MAX_NAME_LENGTH,
            });
        }

        if proto.symbol.len() > MAX_SYMBOL_LENGTH {
            return Err(TokenFactoryError::MetadataFieldTooLong {
                field: "symbol",
                max_len: MAX_SYMBOL_LENGTH,
            });
        }

        if proto.description.len() > MAX_DESCRIPTION_LENGTH {
            return Err(TokenFactoryError::MetadataFieldTooLong {
                field: "description",
                max_len: MAX_DESCRIPTION_LENGTH,
            });
        }

        Ok(())
    }

    /// Compute the value balance for this action.
    ///
    /// Returns:
    /// - The initial supply as positive balance (tokens produced)
    /// - If `enable_mint`, also the MintCapability(seq=0) as positive balance
    // `expect` is on an always-well-formed derived denom: `nonce` is a
    // length-validated `TokenFactoryId`, so `factory/{hex}` always derives a valid
    // asset id. `balance()` returns `Balance` and cannot propagate a `Result`.
    #[allow(clippy::expect_used)]
    pub fn balance(&self) -> Balance {
        let token_denom = self.nonce.denom();
        let asset_id = asset_id_from_denom(&token_denom).expect(
            "TokenFactoryId is length-validated, so its factory/{hex} denom always \
             derives a valid asset id",
        );

        let mut balance = Balance::default();

        // Add initial supply (if non-zero)
        if self.initial_supply.value() > 0 {
            let supply_value = Value {
                asset_id,
                amount: self.initial_supply,
            };
            balance = balance + Balance::from(supply_value);
        }

        // Add mint capability if enabled
        if self.enable_mint {
            let capability = MintCapability::initial(self.nonce.clone());
            balance = balance + Balance::from(capability.value());
        }

        balance
    }
}

impl EffectingData for ActionTokenFactoryCreate {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl DomainType for ActionTokenFactoryCreate {
    type Proto = pb::ActionTokenFactoryCreate;
}

impl From<ActionTokenFactoryCreate> for pb::ActionTokenFactoryCreate {
    fn from(action: ActionTokenFactoryCreate) -> Self {
        Self {
            nonce: Some(action.nonce.into()),
            metadata: Some(action.metadata.into()),
            initial_supply: Some(action.initial_supply.into()),
            enable_mint: action.enable_mint,
        }
    }
}

impl TryFrom<pb::ActionTokenFactoryCreate> for ActionTokenFactoryCreate {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ActionTokenFactoryCreate) -> Result<Self, Self::Error> {
        let nonce: TokenFactoryId = proto
            .nonce
            .ok_or_else(|| anyhow!("missing nonce"))?
            .try_into()
            .map_err(|e: TokenFactoryError| anyhow!(e))?;

        let metadata: asset::Metadata = proto
            .metadata
            .ok_or_else(|| anyhow!("missing metadata"))?
            .try_into()?;

        let initial_supply: Amount = proto
            .initial_supply
            .ok_or_else(|| anyhow!("missing initial_supply"))?
            .try_into()?;

        Self::new(nonce, metadata, initial_supply, proto.enable_mint).map_err(|e| anyhow!(e))
    }
}
