#![allow(unused)] // TODO: remove this when filling in the RPCs

use penumbra_sdk_dex::{component::PositionRead, lp::position};
use penumbra_sdk_proto::{
    core::component::auction::v1 as pb,
    core::component::auction::v1::{
        query_service_server::QueryService, AuctionStateByIdRequest, AuctionStateByIdResponse,
        AuctionStateByIdsRequest, AuctionStateByIdsResponse, DutchAuctionState,
    },
    DomainType,
};

use async_stream::try_stream;
use futures::{StreamExt, TryStreamExt};
use penumbra_sdk_proto::Message;
use prost::Name;
use std::pin::Pin;
use tonic::Status;
use tracing::instrument;

use crate::auction::dutch::DutchAuction;

use super::{action_handler::dutch, AuctionStoreRead};
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

        let raw_auction = state
            .get_raw_auction(id)
            .await
            .ok_or_else(|| tonic::Status::not_found("auction data not found for specified id"))?;

        // Note: we can easily optimize this by adding a lookup table for auction_id -> position id and
        // save on deserialization or needing to "support" things in this rpc.
        let maybe_lp = if raw_auction.type_url == pb::DutchAuction::type_url() {
            let dutch_auction = DutchAuction::decode(raw_auction.value.as_ref())
                .map_err(|_| tonic::Status::internal("error deserializing auction state"))?;

            dutch_auction.state.current_position
        } else {
            return Err(tonic::Status::unimplemented("unrecognized auction type"));
        };

        let positions = match maybe_lp {
            Some(id) => state
                .position_by_id(&id)
                .await
                .map_err(|_| tonic::Status::internal("error fetching position state"))?
                .into_iter()
                .map(Into::into)
                .collect(),
            None => Vec::new(),
        };

        Ok(tonic::Response::new(AuctionStateByIdResponse {
            auction: Some(raw_auction),
            positions,
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
