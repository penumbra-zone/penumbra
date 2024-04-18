#![allow(unused)] // TODO: remove this when filling in the RPCs

use penumbra_proto::{
    core::component::auction::v1alpha1::{
        query_service_server::QueryService, AuctionStateByIdRequest, AuctionStateByIdResponse,
    },
    DomainType,
};

use async_stream::try_stream;
use cnidarium::Storage;
use futures::{StreamExt, TryStreamExt};
use std::pin::Pin;
use tonic::Status;
use tracing::instrument;

use super::AuctionStoreRead;

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

        let auction_data = state.get_raw_auction().await.map_err(|e| {
            tonic::Status::unknown(format!(
                "could not get stateful auction data for specified auction id: {e}"
            ))
        })?;

        Ok(tonic::Response::new(AuctionStateByIdResponse {
            auction: todo!(),
            positions: todo!(),
        }))
    }
}
