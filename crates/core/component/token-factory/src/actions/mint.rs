//! ActionTokenFactoryMint - mint additional tokens using the mint capability.
//!
//! # Value Balance
//!
//! ```text
//! ┌────────────────────────────────┬──────────────────────────────────────┐
//! │           Consumed             │            Produced                  │
//! ├────────────────────────────────┼──────────────────────────────────────┤
//! │  MintCapability(id, seq=N)     │  MintCapability(id, seq=N+1)         │
//! │                                │  + minted tokens                     │
//! └────────────────────────────────┴──────────────────────────────────────┘
//! ```
//!
//! # Security Model
//!
//! The mint capability is a unique asset that must be consumed to mint.
//! This means:
//! - Only the capability holder can mint
//! - The sequence number creates an auditable chain
//! - Each capability can only be used once (consumed and re-emitted)
//!
//! # Bridge Use Case
//!
//! For a wrapped asset bridge:
//! 1. MPC holds the MintCapability for wNEAR
//! 2. User deposits NEAR on source chain
//! 3. MPC submits tx: Spend(capability) + Mint + Output(new capability) + Output(wNEAR to user)
//! 4. User receives shielded wNEAR

use crate::{error::TokenFactoryError, MintCapability, TokenFactoryId, MAX_MINT_AMOUNT};
use anyhow::anyhow;
use penumbra_sdk_asset::{asset, Balance, Value};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{
    core::asset::v1 as pb_asset, core::component::token_factory::v1 as pb, DomainType,
};
use penumbra_sdk_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};

/// Compute asset ID from a raw denom string.
fn asset_id_from_denom(denom: &str) -> asset::Id {
    let proto = pb_asset::AssetId {
        inner: vec![],
        alt_bech32m: String::new(),
        alt_base_denom: denom.to_string(),
    };
    asset::Id::try_from(proto).expect("asset id computation from valid denom cannot fail")
}

/// Action to mint additional tokens.
///
/// Requires consuming MintCapability(seq=N) and outputs MintCapability(seq=N+1)
/// plus the minted tokens. The balance check enforces that the capability is held.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "pb::ActionTokenFactoryMint",
    into = "pb::ActionTokenFactoryMint"
)]
pub struct ActionTokenFactoryMint {
    /// The token factory ID to mint from.
    pub token_id: TokenFactoryId,
    /// The current sequence number of the mint capability being consumed.
    pub current_seq: u64,
    /// Amount to mint.
    pub amount: Amount,
}

impl ActionTokenFactoryMint {
    /// Create a new mint action.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The mint amount is zero
    /// - The mint amount exceeds MAX_MINT_AMOUNT
    /// - The sequence number would overflow on increment
    pub fn new(
        token_id: TokenFactoryId,
        current_seq: u64,
        amount: Amount,
    ) -> Result<Self, TokenFactoryError> {
        // Validate amount is non-zero
        if amount.value() == 0 {
            return Err(TokenFactoryError::MintAmountTooLarge(0, 1));
        }

        // Validate amount doesn't exceed max
        if amount.value() > MAX_MINT_AMOUNT {
            return Err(TokenFactoryError::MintAmountTooLarge(
                amount.value(),
                MAX_MINT_AMOUNT,
            ));
        }

        // Validate sequence won't overflow
        if current_seq == u64::MAX {
            return Err(TokenFactoryError::SequenceOverflow);
        }

        Ok(Self {
            token_id,
            current_seq,
            amount,
        })
    }

    /// Get the mint capability being consumed.
    pub fn consumed_capability(&self) -> MintCapability {
        MintCapability::new(self.token_id.clone(), self.current_seq)
    }

    /// Get the mint capability being produced (next sequence).
    ///
    /// # Panics
    ///
    /// This will not panic because the constructor validates that
    /// the sequence can be incremented.
    pub fn produced_capability(&self) -> MintCapability {
        // Safe because constructor validates current_seq < u64::MAX
        self.consumed_capability()
            .next()
            .expect("constructor validates sequence can increment")
    }

    /// Compute the value balance for this action.
    ///
    /// Returns:
    /// - Negative: MintCapability(seq=N) consumed
    /// - Positive: MintCapability(seq=N+1) + minted tokens produced
    pub fn balance(&self) -> Balance {
        let consumed_cap = self.consumed_capability();
        let produced_cap = self.produced_capability();

        let token_denom = self.token_id.denom();
        let token_asset_id = asset_id_from_denom(&token_denom);

        let minted_tokens = Value {
            asset_id: token_asset_id,
            amount: self.amount,
        };

        // Consume old capability, produce new capability + tokens
        -Balance::from(consumed_cap.value())
            + Balance::from(produced_cap.value())
            + Balance::from(minted_tokens)
    }
}

impl EffectingData for ActionTokenFactoryMint {
    fn effect_hash(&self) -> EffectHash {
        EffectHash::from_proto_effecting_data(&self.to_proto())
    }
}

impl DomainType for ActionTokenFactoryMint {
    type Proto = pb::ActionTokenFactoryMint;
}

impl From<ActionTokenFactoryMint> for pb::ActionTokenFactoryMint {
    fn from(action: ActionTokenFactoryMint) -> Self {
        Self {
            token_id: Some(action.token_id.into()),
            current_seq: action.current_seq,
            amount: Some(action.amount.into()),
        }
    }
}

impl TryFrom<pb::ActionTokenFactoryMint> for ActionTokenFactoryMint {
    type Error = anyhow::Error;

    fn try_from(proto: pb::ActionTokenFactoryMint) -> Result<Self, Self::Error> {
        let token_id: TokenFactoryId = proto
            .token_id
            .ok_or_else(|| anyhow!("missing token_id"))?
            .try_into()
            .map_err(|e: TokenFactoryError| anyhow!(e))?;

        let amount: Amount = proto
            .amount
            .ok_or_else(|| anyhow!("missing amount"))?
            .try_into()?;

        Self::new(token_id, proto.current_seq, amount).map_err(|e| anyhow!(e))
    }
}
