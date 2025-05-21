use penumbra_sdk_asset::{asset, Value};
use penumbra_sdk_proto::core::{
    asset::v1 as pb,
    component::community_pool::v1::{
        query_service_server::QueryService, CommunityPoolAssetBalancesRequest,
        CommunityPoolAssetBalancesResponse,
    },
};

use async_stream::try_stream;
use futures::{StreamExt, TryStreamExt};
use std::pin::Pin;
use tonic::Status;
use tracing::instrument;

use cnidarium::Storage;

use super::StateReadExt;

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
    /// Stream of asset balance info within the CommunityPool, to satisfy the "repeated"
    /// field in the protobuf spec.
    type CommunityPoolAssetBalancesStream = Pin<
        Box<
            dyn futures::Stream<Item = Result<CommunityPoolAssetBalancesResponse, tonic::Status>>
                + Send,
        >,
    >;

    #[instrument(skip(self, request))]
    async fn community_pool_asset_balances(
        &self,
        request: tonic::Request<CommunityPoolAssetBalancesRequest>,
    ) -> Result<tonic::Response<Self::CommunityPoolAssetBalancesStream>, Status> {
        let state = self.storage.latest_snapshot();
        let request = request.into_inner();

        // Asset IDs are optional in the req; if none set, return all balances.
        let asset_ids = request
            .asset_ids
            .into_iter()
            .map(asset::Id::try_from)
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(|_| tonic::Status::invalid_argument("failed to parse asset filter"))?;

        // Get all balances; we can filter later.
        let asset_balances = state.community_pool_balance().await.or_else(|_| {
            Err(tonic::Status::internal(
                "failed to find community pool balances",
            ))
        })?;

        let s = try_stream! {
            for (asset_id, amount) in asset_balances {
                let v = Value { asset_id, amount };
                // Check whether a filter was requested
                if asset_ids.is_empty() || asset_ids.contains(&asset_id){
                    yield pb::Value::from(v);
                }
            }
        };
        Ok(tonic::Response::new(
            s.map_ok(|value| CommunityPoolAssetBalancesResponse {
                balance: Some(value),
            })
            .map_err(|e: anyhow::Error| {
                tonic::Status::unavailable(format!(
                    "error getting balances for community pool: {e}"
                ))
            })
            .boxed(),
        ))
    }
}
