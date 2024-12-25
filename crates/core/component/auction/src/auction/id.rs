use anyhow::{bail, Context};
use penumbra_sdk_proto::{
    penumbra::core::component::auction::v1 as pb, serializers::bech32str, DomainType,
};
use serde::{Deserialize, Serialize};

/// A unique identifier for an auction, obtained from hashing a domain separator
/// and an immutable auction description.
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::AuctionId", into = "pb::AuctionId")]
pub struct AuctionId(pub [u8; 32]);

/* Basic impls */
impl std::str::FromStr for AuctionId {
    type Err = anyhow::Error;

    // IMPORTANT: changing this is state-breaking.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = bech32str::decode(s, bech32str::auction_id::BECH32_PREFIX, bech32str::Bech32m)?;
        pb::AuctionId { inner }.try_into()
    }
}

impl std::fmt::Debug for AuctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Display for AuctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // IMPORTANT: changing this is state-breaking.
        f.write_str(&bech32str::encode(
            &self.0,
            bech32str::auction_id::BECH32_PREFIX,
            bech32str::Bech32m,
        ))
    }
}

/* Protobuf impls */
impl From<AuctionId> for pb::AuctionId {
    fn from(domain: AuctionId) -> Self {
        Self {
            inner: domain.0.to_vec(),
        }
    }
}

impl DomainType for AuctionId {
    type Proto = pb::AuctionId;
}

impl TryFrom<pb::AuctionId> for AuctionId {
    type Error = anyhow::Error;

    fn try_from(msg: pb::AuctionId) -> Result<Self, Self::Error> {
        if msg.inner.is_empty() {
            bail!("AuctionId proto message is empty")
        } else {
            let raw_id: [u8; 32] = msg
                .inner
                .as_slice()
                .try_into()
                .context("raw AuctionId must be 32 bytes")?;
            Ok(AuctionId(raw_id))
        }
    }
}
