#![allow(unused)] // TODO: remove this when filling in the RPCs

use penumbra_proto::{
    core::component::auction::v1alpha1::{
        query_service_server::QueryService, AuctionStateByIdRequest, AuctionStateByIdResponse,
        AuctionStateByIdsRequest, AuctionStateByIdsResponse, DutchAuctionState,
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

        tracing::debug!("received auction state by id request");

        let id = request
            .id
            .ok_or_else(|| Status::invalid_argument("missing auction id"))?
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid auction id"))?;

        tracing::debug!(?id, "able to parse auction id");

        let raw_auction = state
            .get_raw_auction(id)
            .await
            .ok_or_else(|| tonic::Status::not_found("auction data not found for specified id"))?;

        tracing::debug!(?raw_auction, "found auction data");

        Ok(tonic::Response::new(AuctionStateByIdResponse {
            auction: Some(raw_auction),
            positions: Vec::new(),
        }))
    }

    type AuctionStateByIdsStream = Pin<
        Box<dyn futures::Stream<Item = Result<AuctionStateByIdsResponse, tonic::Status>> + Send>,
    >;

    #[instrument(skip(self, request))]
    async fn auction_state_by_ids(
        &self,
        request: tonic::Request<AuctionStateByIdsRequest>,
    ) -> Result<tonic::Response<Self::AuctionStateByIdsStream>, Status> {
        todo!()
    }
}
