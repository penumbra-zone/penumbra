use cnidarium::Storage;
use pbjson_types::Timestamp;
use penumbra_sdk_proto::core::component::sct::v1::query_service_server::QueryService;
use penumbra_sdk_proto::core::component::sct::v1::{
    AnchorByHeightRequest, AnchorByHeightResponse, EpochByHeightRequest, EpochByHeightResponse,
    SctFrontierRequest, SctFrontierResponse, TimestampByHeightRequest, TimestampByHeightResponse,
};
use penumbra_sdk_proto::crypto::tct::v1 as pb_tct;
use tonic::Status;
use tracing::instrument;

use crate::state_key;

use super::clock::EpochRead;
use super::tree::SctRead;

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
    #[instrument(skip(self, request))]
    async fn epoch_by_height(
        &self,
        request: tonic::Request<EpochByHeightRequest>,
    ) -> Result<tonic::Response<EpochByHeightResponse>, Status> {
        let state = self.storage.latest_snapshot();

        let epoch = state
            .get_epoch_by_height(request.get_ref().height)
            .await
            .map_err(|e| tonic::Status::unknown(format!("could not get epoch for height: {e}")))?;

        Ok(tonic::Response::new(EpochByHeightResponse {
            epoch: Some(epoch.into()),
        }))
    }

    #[instrument(skip(self, request))]
    async fn anchor_by_height(
        &self,
        request: tonic::Request<AnchorByHeightRequest>,
    ) -> Result<tonic::Response<AnchorByHeightResponse>, Status> {
        let state = self.storage.latest_snapshot();

        let height = request.get_ref().height;
        let anchor = state.get_anchor_by_height(height).await.map_err(|e| {
            tonic::Status::unknown(format!("could not get anchor for height {height}: {e}"))
        })?;

        Ok(tonic::Response::new(AnchorByHeightResponse {
            anchor: anchor.map(Into::into),
        }))
    }

    #[instrument(skip(self, request))]
    async fn timestamp_by_height(
        &self,
        request: tonic::Request<TimestampByHeightRequest>,
    ) -> Result<tonic::Response<TimestampByHeightResponse>, Status> {
        let state = self.storage.latest_snapshot();

        let height = request.get_ref().height;
        let block_time = state.get_block_timestamp(height).await.map_err(|e| {
            tonic::Status::unknown(format!("could not get timestamp for height {height}: {e}"))
        })?;
        let timestamp = chrono::DateTime::parse_from_rfc3339(block_time.to_rfc3339().as_str())
            .expect("timestamp should roundtrip to string");

        Ok(tonic::Response::new(TimestampByHeightResponse {
            timestamp: Some(Timestamp {
                seconds: timestamp.timestamp(),
                nanos: timestamp.timestamp_subsec_nanos() as i32,
            }),
        }))
    }

    #[instrument(skip(self, request))]
    async fn sct_frontier(
        &self,
        request: tonic::Request<SctFrontierRequest>,
    ) -> Result<tonic::Response<SctFrontierResponse>, Status> {
        let state = self.storage.latest_snapshot();

        let with_proof = request.get_ref().with_proof;

        let frontier = state.get_sct().await;
        let current_height = state
            .get_block_height()
            .await
            .map_err(|e| tonic::Status::unknown(format!("could not get current height: {e}")))?;

        let (anchor, maybe_proof) = if !with_proof {
            (frontier.root(), None)
        } else {
            let anchor_key = state_key::tree::anchor_by_height(current_height)
                .as_bytes()
                .to_vec();
            let (maybe_raw_anchor, proof) =
                state.get_with_proof(anchor_key).await.map_err(|e| {
                    tonic::Status::unknown(format!(
                        "could not get w/ proof anchor for height {current_height}: {e}"
                    ))
                })?;

            let Some(raw_anchor) = maybe_raw_anchor else {
                return Err(tonic::Status::not_found(format!(
                    "anchor not found for height {current_height}"
                )));
            };

            let proto_anchor: pb_tct::MerkleRoot = pb_tct::MerkleRoot { inner: raw_anchor };
            let anchor: penumbra_sdk_tct::Root = proto_anchor
                .try_into()
                .map_err(|_| tonic::Status::internal("failed to parse anchor"))?;
            (anchor, Some(proof.into()))
        };

        // Sanity check we got the right anchor - redundant if no proof was requested
        let locked_anchor = frontier.root();
        if anchor != locked_anchor {
            return Err(tonic::Status::internal(format!(
                "anchor mismatch: {anchor} != {locked_anchor}"
            )));
        }

        let raw_frontier = bincode::serialize(&frontier)
            .map_err(|e| tonic::Status::internal(format!("failed to serialize SCT: {e}")))?;

        Ok(tonic::Response::new(SctFrontierResponse {
            height: current_height,
            anchor: Some(anchor.into()),
            compact_frontier: raw_frontier,
            proof: maybe_proof,
        }))
    }
}
