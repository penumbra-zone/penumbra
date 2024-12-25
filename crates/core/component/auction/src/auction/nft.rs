use crate::auction::id::AuctionId;
use anyhow::{anyhow, Result};
use penumbra_sdk_asset::asset::{self, Metadata};
use penumbra_sdk_proto::{core::component::auction::v1 as pb, DomainType};
use regex::Regex;

/// An non-fungible token (NFT) tracking the state and ownership of an auction.
#[derive(Debug, Clone)]
pub struct AuctionNft {
    /// The unique identifier for the auction this nft resolves to.
    pub id: AuctionId,
    /// The state of an auction, its specific semantics depend on the
    /// type of auction the NFT resolves to.
    pub seq: u64,
    /// The metadata corresponding to the nft denom.
    pub metadata: asset::Metadata,
}

impl AuctionNft {
    pub fn new(id: AuctionId, seq: u64) -> AuctionNft {
        let metadata = asset::REGISTRY
            .parse_denom(&format!("auctionnft_{seq}_{id}"))
            .expect("auction nft denom is valid");
        AuctionNft { id, seq, metadata }
    }

    pub fn asset_id(&self) -> asset::Id {
        self.metadata.id()
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

impl TryFrom<Metadata> for AuctionNft {
    type Error = anyhow::Error;

    fn try_from(denom: Metadata) -> Result<Self, Self::Error> {
        let regex = Regex::new(
            "^auctionnft_(?P<seq_num>[0-9]+)_(?P<auction_id>pauctid1[a-zA-HJ-NP-Z0-9]+)$",
        )
        .expect("regex is valid");

        let denom_string = denom.to_string();

        let captures = regex
            .captures(&denom_string)
            .ok_or_else(|| anyhow!("denom {} is not a valid auction nft", denom))?;

        let seq_num = captures
            .name("seq_num")
            .ok_or_else(|| anyhow!("sequence number not found"))?
            .as_str();
        let auction_id = captures
            .name("auction_id")
            .ok_or_else(|| anyhow!("auction ID not found"))?
            .as_str();

        let seq_num: u64 = seq_num
            .parse()
            .map_err(|_| anyhow!("Failed to parse seq_num to u64"))?;

        let auction_id: AuctionId = auction_id
            .parse()
            .map_err(|_| anyhow!("Failed to parse auction_id to AuctionId"))?;

        Ok(AuctionNft::new(auction_id, seq_num))
    }
}
