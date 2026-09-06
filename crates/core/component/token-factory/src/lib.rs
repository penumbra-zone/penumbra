//! Token Factory component for Penumbra.
//!
//! This component enables creation and minting of tokens within Penumbra's
//! shielded pool. It supports two modes:
//!
//! - **Fairlaunch tokens**: Fixed supply at creation, no minting authority.
//! - **Mintable tokens**: Supply can be expanded by the mint capability holder.
//!
//! # Design Philosophy
//!
//! The mint capability is a unique asset that represents the right to mint
//! additional tokens. Whoever holds the capability can mint. The capability
//! uses a sequence number that increments on each mint, creating an auditable
//! chain of mints and preventing replay attacks.
//!
//! # Bridge Use Case
//!
//! For wrapped asset bridges (e.g., wNEAR on Penumbra):
//! 1. Bridge MPC creates token with mint capability
//! 2. When user deposits on source chain, MPC mints wrapped tokens
//! 3. User receives shielded wrapped tokens
//! 4. To exit, user sends to MPC address; MPC releases on source chain
//!
//! # Security Model
//!
//! - **Mint Authority**: Whoever holds the MintCapability asset can mint.
//! - **Sequence Numbers**: Each mint increments the sequence, preventing replay.
//! - **Privacy**: Token recipients are shielded. Creation/mint metadata is public.
//! - **Uniqueness**: The 32-byte nonce ensures each token ID is unique.
//!
//! # Actions
//!
//! - [`ActionTokenFactoryCreate`] - Create a new token, optionally with mint capability.
//! - [`ActionTokenFactoryMint`] - Mint additional tokens (requires capability).

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

mod error;
mod id;
mod mint_capability;

pub mod actions;

#[cfg(feature = "component")]
pub mod component;

pub use error::TokenFactoryError;
pub use id::TokenFactoryId;
pub use mint_capability::MintCapability;

pub use actions::{ActionTokenFactoryCreate, ActionTokenFactoryMint};

#[cfg(feature = "component")]
pub use component::{StateReadExt, StateWriteExt, TokenFactory};

/// Maximum initial supply for a token factory token.
///
/// This limit exists to prevent overflow in balance calculations.
/// The value is 2^112 - 1, which fits within Penumbra's Amount type
/// while providing ample headroom for any practical use case.
pub const MAX_INITIAL_SUPPLY: u128 = (1 << 112) - 1;

/// Maximum amount that can be minted in a single mint action.
///
/// Same limit as initial supply to maintain consistency.
pub const MAX_MINT_AMOUNT: u128 = (1 << 112) - 1;

/// Derive an asset ID from a raw denom string.
///
/// Single definition shared by `mint` and `mint_capability`: these previously
/// had separate copies, and if they ever diverged `mint` would produce a
/// different asset than `create` registered.
///
/// Fallible on purpose. The denom is derived from a `TokenFactoryId` that
/// arrives over the wire, so an `expect()` here is reachable from untrusted
/// input and would panic every validator identically — a chain halt rather
/// than a rejected transaction.
pub fn asset_id_from_denom(
    denom: &str,
) -> Result<penumbra_sdk_asset::asset::Id, crate::error::TokenFactoryError> {
    use penumbra_sdk_proto::core::asset::v1 as pb_asset;
    let proto = pb_asset::AssetId {
        inner: vec![],
        alt_bech32m: String::new(),
        alt_base_denom: denom.to_string(),
    };
    penumbra_sdk_asset::asset::Id::try_from(proto)
        .map_err(|_| crate::error::TokenFactoryError::InvalidDenomAssetId(denom.to_string()))
}
