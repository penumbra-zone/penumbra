//! Mint authority NFT for token factory.
//!
//! The [`MintNft`] represents the authority to mint additional tokens for a
//! specific token factory token. It uses a stateful NFT pattern where each
//! minting operation consumes the current NFT and produces a new one with
//! an incremented sequence number.
//!
//! # Security Model
//!
//! ## Replay Protection
//!
//! The sequence number provides replay protection: once an NFT with seq=N is
//! consumed, only an NFT with seq=N+1 can be used for the next mint. This
//! prevents:
//!
//! - Double-spending of minting authority
//! - Replay attacks using old mint transactions
//!
//! ## Privacy
//!
//! The mint NFT flows through Penumbra's shielded pool like any other asset,
//! providing unlinkability between:
//!
//! - Token creation and minting operations
//! - Different minting operations by the same authority
//! - Authority transfers
//!
//! ## Authority Transfer
//!
//! Minting rights can be transferred by simply transferring the NFT. The new
//! holder can mint tokens without any on-chain record of the transfer.

use crate::{error::TokenFactoryError, id::TokenFactoryId};
use once_cell::sync::Lazy;
use penumbra_sdk_asset::asset::{self, Metadata};
use penumbra_sdk_proto::{core::asset::v1 as pb_asset, DomainType};
use regex::Regex;

/// Compiled regex for parsing mint NFT denoms.
///
/// Format: `factory_mint_{seq}_{hex_id}` where:
/// - `seq` is a decimal sequence number (u64)
/// - `hex_id` is a 64-character hex string (32 bytes)
static MINT_NFT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^factory_mint_(?P<seq>[0-9]+)_(?P<id>[a-fA-F0-9]{64})$")
        .expect("mint NFT regex is valid by construction")
});

/// Compute asset ID from a raw denom string using the public proto interface.
///
/// # Panics
///
/// This function cannot panic because the proto conversion always succeeds
/// when alt_base_denom is non-empty, and we always provide a non-empty denom.
#[allow(clippy::expect_used)]
pub(crate) fn asset_id_from_denom(denom: &str) -> asset::Id {
    let proto = pb_asset::AssetId {
        inner: vec![],
        alt_bech32m: String::new(),
        alt_base_denom: denom.to_string(),
    };
    // Safety: This conversion cannot fail when alt_base_denom is non-empty.
    // The proto interface computes the asset ID via blake2b hash of the denom.
    asset::Id::try_from(proto).expect("asset id computation from valid denom cannot fail")
}

/// A non-fungible token tracking minting authority for a factory token.
///
/// The denom format is `factory_mint_{seq}_{hex_id}` where:
/// - `seq` is the sequence number (increments on each mint)
/// - `hex_id` is the hex-encoded token factory ID
#[derive(Debug, Clone)]
pub struct MintNft {
    /// The token factory this NFT controls.
    pub id: TokenFactoryId,
    /// Sequence number - increments on each mint action.
    pub seq: u64,
}

impl MintNft {
    /// Create a new mint authority NFT.
    ///
    /// # Parameters
    ///
    /// - `id`: The token factory this NFT controls
    /// - `seq`: The sequence number (0 for newly created tokens)
    pub fn new(id: TokenFactoryId, seq: u64) -> Self {
        Self { id, seq }
    }

    /// Get the asset ID for this NFT.
    pub fn asset_id(&self) -> asset::Id {
        asset_id_from_denom(&self.denom())
    }

    /// Get the denom string for this NFT.
    pub fn denom(&self) -> String {
        format!("factory_mint_{}_{}", self.seq, hex::encode(self.id.as_bytes()))
    }

    /// Create the next NFT in the sequence (seq + 1).
    ///
    /// Used when processing a mint action to produce the new mint authority.
    ///
    /// # Errors
    ///
    /// Returns [`TokenFactoryError::SequenceOverflow`] if the sequence number
    /// would overflow. This is practically unreachable (would require 2^64 mints)
    /// but we handle it defensively.
    pub fn next(&self) -> Result<Self, TokenFactoryError> {
        let next_seq = self
            .seq
            .checked_add(1)
            .ok_or(TokenFactoryError::SequenceOverflow)?;
        Ok(Self::new(self.id, next_seq))
    }
}

impl TryFrom<Metadata> for MintNft {
    type Error = TokenFactoryError;

    fn try_from(denom: Metadata) -> Result<Self, Self::Error> {
        let denom_string = denom.to_string();

        let captures = MINT_NFT_REGEX
            .captures(&denom_string)
            .ok_or(TokenFactoryError::InvalidMintNftDenom)?;

        let seq_str = captures
            .name("seq")
            .ok_or(TokenFactoryError::InvalidMintNftDenom)?
            .as_str();
        let id_str = captures
            .name("id")
            .ok_or(TokenFactoryError::InvalidMintNftDenom)?
            .as_str();

        let seq: u64 = seq_str
            .parse()
            .map_err(|_| TokenFactoryError::InvalidMintNftDenom)?;

        let id: TokenFactoryId = format!("factory/{}", id_str)
            .parse()
            .map_err(|_| TokenFactoryError::InvalidMintNftDenom)?;

        Ok(MintNft::new(id, seq))
    }
}

// Protobuf conversion
impl DomainType for MintNft {
    type Proto = penumbra_sdk_proto::core::component::token_factory::v1::MintNft;
}

impl From<MintNft> for penumbra_sdk_proto::core::component::token_factory::v1::MintNft {
    fn from(nft: MintNft) -> Self {
        Self {
            id: Some(nft.id.into()),
            seq: nft.seq,
        }
    }
}

impl TryFrom<penumbra_sdk_proto::core::component::token_factory::v1::MintNft> for MintNft {
    type Error = TokenFactoryError;

    fn try_from(proto: penumbra_sdk_proto::core::component::token_factory::v1::MintNft) -> Result<Self, Self::Error> {
        let id: TokenFactoryId = proto
            .id
            .ok_or(TokenFactoryError::InvalidMintNftDenom)?
            .try_into()?;
        Ok(MintNft::new(id, proto.seq))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mint_nft_sequence() {
        let id = TokenFactoryId::new([0x42; 32]);
        let nft0 = MintNft::new(id, 0);
        let nft1 = nft0.next().expect("next should succeed");
        let nft2 = nft1.next().expect("next should succeed");

        assert_eq!(nft0.seq, 0);
        assert_eq!(nft1.seq, 1);
        assert_eq!(nft2.seq, 2);
    }

    #[test]
    fn test_mint_nft_sequence_overflow() {
        let id = TokenFactoryId::new([0x42; 32]);
        let nft_max = MintNft::new(id, u64::MAX);

        // Should fail when trying to increment past max
        let result = nft_max.next();
        assert!(matches!(result, Err(TokenFactoryError::SequenceOverflow)));
    }

    #[test]
    fn test_mint_nft_denom_format() {
        let id = TokenFactoryId::new([0xab; 32]);
        let nft = MintNft::new(id, 42);
        let denom = nft.denom();

        assert!(denom.starts_with("factory_mint_42_"));
        assert!(denom.contains(&hex::encode([0xab; 32])));
    }

    #[test]
    fn test_different_seq_different_asset_id() {
        let id = TokenFactoryId::new([0x42; 32]);
        let nft0 = MintNft::new(id, 0);
        let nft1 = MintNft::new(id, 1);

        // Different sequence numbers should produce different asset IDs
        assert_ne!(nft0.asset_id(), nft1.asset_id());
    }
}
