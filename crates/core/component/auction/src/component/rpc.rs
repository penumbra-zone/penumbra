#![allow(unused)] // TODO: remove this when filling in the RPCs

use penumbra_proto::{
    core::component::auction::v1alpha1::{
        query_service_server::QueryService, AuctionStateByIdRequest, AuctionStateByIdResponse,
        DutchAuctionState,
    },
    DomainType,
};

use async_stream::try_stream;
use futures::{StreamExt, TryStreamExt};
use penumbra_proto::Message;
use std::pin::Pin;
use tonic::Status;
use tracing::instrument;

use super::AuctionStoreRead;
use cnidarium::Storage;

pub struct Server {
    storage: Storage,
}

impl Server {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl QueryService for Server {
    #[instrument(skip(self, request))]
    async fn auction_state_by_id(
        &self,
        request: tonic::Request<AuctionStateByIdRequest>,
    ) -> Result<tonic::Response<AuctionStateByIdResponse>, Status> {
        let state = self.storage.latest_snapshot();
        let request = request.into_inner();

        let id = request
            .id
            .ok_or_else(|| Status::invalid_argument("missing auction id"))?
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid auction id"))?;

        let auction_data = state
            .get_raw_auction(id)
            .await
            .ok_or_else(|| tonic::Status::not_found("auction data not found for specified id"))?;

        if auction_data.type_url
            != format!("penumbra.core.component.auction.v1alpha1.DutchAuctionState")
        {
            return Err(Status::invalid_argument(
                "Auction data type does not contain a `DutchAuctionState`",
            ));
        }

        // Attempt to deserialize value into a `DutchAuctionState` message.
        let auction_state_proto = DutchAuctionState::decode(auction_data.value.as_ref())
            .map_err(|_| Status::invalid_argument("Failed to decode DutchAuctionState"))?;

        Ok(tonic::Response::new(AuctionStateByIdResponse {
            auction: Some(auction_state_proto),
            positions: todo!(),
        }))
    }
}
