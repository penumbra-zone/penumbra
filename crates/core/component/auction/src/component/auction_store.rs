use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateRead;
use pbjson_types::Any;
use penumbra_sdk_proto::core::component::auction::v1 as pb;
use penumbra_sdk_proto::DomainType;
use penumbra_sdk_proto::Name;
use penumbra_sdk_proto::StateReadProto;

use crate::{
    auction::{dutch::DutchAuction, id::AuctionId},
    state_key,
};

/// Provide access to internal auction data.
#[async_trait]
pub trait AuctionStoreRead: StateRead {
    /// Returns whether the supplied `auction_id` exists in the chain state.
    async fn auction_id_exists(&self, auction_id: AuctionId) -> bool {
        self.get_raw_auction(auction_id).await.is_some()
    }

    /// Fetch a [`DutchAuction`] from storage, returning `None` if none
    /// were found with the provided identifier.
    ///
    /// # Errors
    /// This method returns an error if the auction state associated with the
    /// specified `auction_id` is *not* of type `DutchAuction`.
    async fn get_dutch_auction_by_id(&self, auction_id: AuctionId) -> Result<Option<DutchAuction>> {
        let Some(any_auction) = self.get_raw_auction(auction_id).await else {
            return Ok(None);
        };

        let dutch_auction_type_str = pb::DutchAuction::type_url();

        anyhow::ensure!(
            any_auction.type_url == dutch_auction_type_str,
            "error deserializing auction state, expected type to be {}, but got: {}",
            dutch_auction_type_str,
            any_auction.type_url
        );

        Ok(Some(DutchAuction::decode(any_auction.value.as_ref())?))
    }

    /// Returns raw auction data if found under the specified `auction_id`,
    /// and `None` otherwise
    async fn get_raw_auction(&self, auction_id: AuctionId) -> Option<Any> {
        self.get_proto(&state_key::auction_store::by_id(auction_id))
            .await
            .expect("no storage errors")
    }
}

impl<T: StateRead + ?Sized> AuctionStoreRead for T {}

#[cfg(test)]
mod tests {}
