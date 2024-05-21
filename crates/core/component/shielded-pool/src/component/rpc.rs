use std::{collections::BTreeMap, pin::Pin};

use anyhow::Context as _;
use async_stream::try_stream;
use cnidarium::Storage;
use futures::{StreamExt as _, TryStreamExt as _};
use penumbra_asset::asset::{self, Metadata};
use penumbra_ibc::component::state_key as ibc_state_key;
use penumbra_num::Amount;
use penumbra_proto::{
    core::component::shielded_pool::v1::{
        query_service_server::QueryService, AssetMetadataByIdRequest, AssetMetadataByIdResponse,
        AssetMetadataByIdsRequest, AssetMetadataByIdsResponse, TotalSupplyRequest,
        TotalSupplyResponse,
    },
    StateReadProto as _,
};

use tonic::Status;
use tracing::instrument;

use crate::state_key;

use super::AssetRegistryRead;

mod transfer_query;

// TODO: Hide this and only expose a Router?
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
    type AssetMetadataByIdsStream = Pin<
        Box<dyn futures::Stream<Item = Result<AssetMetadataByIdsResponse, tonic::Status>> + Send>,
    >;
    type TotalSupplyStream =
        Pin<Box<dyn futures::Stream<Item = Result<TotalSupplyResponse, tonic::Status>> + Send>>;

    /// Returns the total supply for all IBC assets.
    /// Internally-minted assets (Penumbra tokens, LP tokens, delegation tokens, etc.)
    /// are also included but the supplies are hardcoded at 0 for now.
    ///
    /// TODO: Implement a way to fetch the total supply for these assets.
    #[instrument(skip(self, _request))]
    async fn total_supply(
        &self,
        _request: tonic::Request<TotalSupplyRequest>,
    ) -> Result<tonic::Response<Self::TotalSupplyStream>, tonic::Status> {
        let snapshot = self.storage.latest_snapshot();

        // Find every non-IBC known asset
        let s = snapshot.prefix(state_key::denom_metadata_by_asset::prefix());
        let mut total_supply = s
            .filter_map(move |i: anyhow::Result<(String, Metadata)>| async move {
                if i.is_err() {
                    return Some(Err(i.context("bad denom in state").err().unwrap()));
                }
                let (_key, denom_metadata) = i.expect("should not be an error");

                // Return a hardcoded 0 supply for now
                Some(Ok((denom_metadata, Amount::from(0u32))))
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .into_iter()
            .collect::<BTreeMap<_, _>>();

        let s = snapshot.prefix(ibc_state_key::ics20_value_balance::prefix());
        let ibc_amounts = s
            .filter_map(move |i: anyhow::Result<(String, Amount)>| async move {
                if i.is_err() {
                    return Some(Err(i.context("bad amount in state").err().unwrap()));
                }
                let (key, amount) = i.expect("should not be an error");

                // Extract the asset ID from the key
                let asset_id = key.split('/').last();
                if asset_id.is_none() {
                    return Some(Err(asset_id
                        .context("bad IBC ics20 value balance key in state")
                        .err()
                        .unwrap()));
                }
                let asset_id = asset_id.expect("should not be an error");

                // Parse the asset ID
                let asset_id = asset_id.parse::<asset::Id>();
                if asset_id.is_err() {
                    return Some(Err(asset_id
                        .context("invalid IBC ics20 value balance asset ID in state")
                        .err()
                        .unwrap()));
                }
                let asset_id = asset_id.expect("should not be an error");

                Some(Ok((asset_id, amount)))
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        // Fetch the denoms associated with the IBC asset IDs
        for (asset_id, amount) in ibc_amounts {
            let denom_metadata = snapshot.denom_metadata_by_asset(&asset_id).await;
            if denom_metadata.is_none() {
                return Err(tonic::Status::internal(
                    "bad IBC ics20 value balance key in state".to_string(),
                ));
            }
            let denom_metadata = denom_metadata.expect("should not be an error");

            total_supply.insert(denom_metadata, amount);
        }

        // Kind of a boneheaded streaming approach due to the various Collect calls above
        // but works for now.
        let stream = try_stream! {
            for (denom_metadata, amount) in total_supply {
                yield TotalSupplyResponse {
                    denom_metadata: Some(denom_metadata.into()),
                    amount: Some(amount.into()),
                };
            }
        };

        Ok(tonic::Response::new(
            stream
                .map_err(|e: anyhow::Error| {
                    tonic::Status::unavailable(format!(
                        "error getting position value from storage: {e}"
                    ))
                })
                // TODO: how do we instrument a Stream
                //.instrument(Span::current())
                .boxed(),
        ))
    }

    #[instrument(skip(self, request))]
    async fn asset_metadata_by_id(
        &self,
        request: tonic::Request<AssetMetadataByIdRequest>,
    ) -> Result<tonic::Response<AssetMetadataByIdResponse>, Status> {
        let state = self.storage.latest_snapshot();

        let request = request.into_inner();
        let id: asset::Id = request
            .asset_id
            .ok_or_else(|| Status::invalid_argument("missing asset_id"))?
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("could not parse asset_id: {e}")))?;

        let denom_metadata = state.denom_metadata_by_asset(&id).await;

        let rsp = match denom_metadata {
            Some(denom_metadata) => {
                tracing::debug!(?id, ?denom_metadata, "found denom metadata");
                AssetMetadataByIdResponse {
                    denom_metadata: Some(denom_metadata.into()),
                }
            }
            None => {
                tracing::debug!(?id, "unknown asset id");
                Default::default()
            }
        };

        Ok(tonic::Response::new(rsp))
    }

    async fn asset_metadata_by_ids(
        &self,
        _request: tonic::Request<AssetMetadataByIdsRequest>,
    ) -> Result<tonic::Response<Self::AssetMetadataByIdsStream>, tonic::Status> {
        unimplemented!("asset_metadata_by_ids not yet implemented")
    }
}
