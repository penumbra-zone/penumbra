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

use crate::{asset_id_from_denom, error::TokenFactoryError, MintCapability, TokenFactoryId, MAX_MINT_AMOUNT};
use anyhow::anyhow;
use penumbra_sdk_asset::{Balance, Value};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{core::component::token_factory::v1 as pb, DomainType};
use penumbra_sdk_txhash::{EffectHash, EffectingData};
use serde::{Deserialize, Serialize};

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
    ///
    /// Fields are PRIVATE deliberately. They were public, which meant an
    /// instance could be built directly with `amount = 0`, `amount >
    /// MAX_MINT_AMOUNT`, or `current_seq = u64::MAX`, bypassing every check in
    /// `new()` — and `produced_capability()` then panics on overflow. Keeping
    /// them private makes `new()`/`TryFrom<proto>` the only constructors, so
    /// the validated invariants hold for every value that exists.
    token_id: TokenFactoryId,
    /// The current sequence number of the mint capability being consumed.
    current_seq: u64,
    /// Amount to mint.
    amount: Amount,
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
            return Err(TokenFactoryError::ZeroMintAmount);
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

    /// The token factory this action mints from.
    pub fn token_id(&self) -> &TokenFactoryId {
        &self.token_id
    }

    /// Sequence number of the capability being consumed.
    pub fn current_seq(&self) -> u64 {
        self.current_seq
    }

    /// Amount minted by this action.
    pub fn amount(&self) -> Amount {
        self.amount
    }

    /// Get the mint capability being consumed.
    pub fn consumed_capability(&self) -> MintCapability {
        MintCapability::new(self.token_id.clone(), self.current_seq)
    }

    /// Get the mint capability being produced (next sequence).
    ///
    /// `expect` is unreachable: fields are private and `new()` rejects
    /// `current_seq == u64::MAX`, and `check_stateless` re-asserts it. This
    /// returns `MintCapability` (not `Result`), so the invariant is asserted here.
    #[allow(clippy::expect_used)]
    pub fn produced_capability(&self) -> MintCapability {
        self.consumed_capability()
            .next()
            .expect("new() rejects current_seq == u64::MAX; fields are private")
    }

    /// Compute the value balance for this action.
    ///
    /// Returns:
    /// - Negative: MintCapability(seq=N) consumed
    /// - Positive: MintCapability(seq=N+1) + minted tokens produced
    // `expect` on an always-well-formed derived denom (length-validated token_id);
    // `balance()` returns `Balance` and cannot propagate a `Result`.
    #[allow(clippy::expect_used)]
    pub fn balance(&self) -> Balance {
        let consumed_cap = self.consumed_capability();
        let produced_cap = self.produced_capability();

        let token_denom = self.token_id.denom();
        let token_asset_id = asset_id_from_denom(&token_denom).expect(
            "TokenFactoryId is length-validated on construction (InvalidIdLength), so \
             its derived `factory/{hex}` denom is always well-formed; this cannot fail"
        );

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
