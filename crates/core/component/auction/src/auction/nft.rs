use crate::auction::id::AuctionId;
use anyhow::{anyhow, Result};
use penumbra_asset::asset::{self};
use penumbra_proto::{core::component::auction::v1alpha1 as pb, DomainType};

/// An non-fungible token (NFT) tracking the state and ownership of an auction.
#[derive(Debug, Clone)]
pub struct AuctionNft {
    /// The unique identifier for the auction this nft resolves to.
    pub id: AuctionId,
    /// The state of an auction, its specific semantics depend on the
    /// type of auction the NFT resolves to.
    pub seq: u64,
    /// The metadata corresponding to the Nft's denomination.
    pub metadata: asset::Metadata,
}

impl AuctionNft {
    pub fn new(id: AuctionId, seq: u64) -> AuctionNft {
        let metadata = asset::REGISTRY
            .parse_denom(&format!("auctionnft_{seq}_{id}"))
            .expect("auction nft denom is valid");
        AuctionNft { id, seq, metadata }
    }
}

/* Protobuf impls ;*/
impl DomainType for AuctionNft {
    type Proto = pb::AuctionNft;
}

impl From<AuctionNft> for pb::AuctionNft {
    fn from(domain: AuctionNft) -> Self {
        Self {
            id: Some(domain.id.into()),
            seq: domain.seq,
        }
    }
}

impl TryFrom<pb::AuctionNft> for AuctionNft {
    type Error = anyhow::Error;

    fn try_from(msg: pb::AuctionNft) -> Result<Self, Self::Error> {
        let id: AuctionId = msg
            .id
            .ok_or_else(|| anyhow!("AuctionNft message is missing an auction id"))?
            .try_into()?;
        let seq = msg.seq;
        Ok(AuctionNft::new(id, seq))
    }
}
