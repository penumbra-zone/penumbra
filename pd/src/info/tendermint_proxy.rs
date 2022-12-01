use std::pin::Pin;

use async_stream::try_stream;
use futures::StreamExt;
use futures::TryStreamExt;
use penumbra_chain::AppHashRead;
use penumbra_chain::StateReadExt as _;
use penumbra_component::stake::StateReadExt as _;
use penumbra_crypto::asset::{self, Asset};
use penumbra_proto::{
    self as proto, tendermint_proxy::service_server::Service as TendermintService,
};

use penumbra_storage::StateRead;
use proto::tendermint_proxy::AbciQueryRequest;
use proto::tendermint_proxy::AbciQueryResponse;
use proto::tendermint_proxy::GetBlockByHeightRequest;
use proto::tendermint_proxy::GetBlockByHeightResponse;
use proto::tendermint_proxy::GetLatestBlockRequest;
use proto::tendermint_proxy::GetLatestBlockResponse;
use proto::tendermint_proxy::GetLatestValidatorSetRequest;
use proto::tendermint_proxy::GetLatestValidatorSetResponse;
use proto::tendermint_proxy::GetNodeInfoRequest;
use proto::tendermint_proxy::GetNodeInfoResponse;
use proto::tendermint_proxy::GetSyncingRequest;
use proto::tendermint_proxy::GetSyncingResponse;
use proto::tendermint_proxy::GetValidatorSetByHeightRequest;
use proto::tendermint_proxy::GetValidatorSetByHeightResponse;
use tonic::Status;
use tracing::instrument;

// We need to use the tracing-futures version of Instrument,
// because we want to instrument a Stream, and the Stream trait
// isn't in std, and the tracing::Instrument trait only works with
// (stable) std types.
//use tracing_futures::Instrument;

use super::Info;

#[tonic::async_trait]
impl TendermintService for Info {
    async fn abci_query(
        &self,
        req: tonic::Request<AbciQueryRequest>,
    ) -> Result<tonic::Response<AbciQueryResponse>, Status> {
        todo!()
    }

    async fn get_node_info(
        &self,
        req: tonic::Request<GetNodeInfoRequest>,
    ) -> Result<tonic::Response<GetNodeInfoResponse>, Status> {
        todo!()
    }

    async fn get_syncing(
        &self,
        req: tonic::Request<GetSyncingRequest>,
    ) -> Result<tonic::Response<GetSyncingResponse>, Status> {
        todo!()
    }

    async fn get_latest_block(
        &self,
        req: tonic::Request<GetLatestBlockRequest>,
    ) -> Result<tonic::Response<GetLatestBlockResponse>, Status> {
        todo!()
    }

    async fn get_block_by_height(
        &self,
        req: tonic::Request<GetBlockByHeightRequest>,
    ) -> Result<tonic::Response<GetBlockByHeightResponse>, Status> {
        todo!()
    }

    async fn get_latest_validator_set(
        &self,
        req: tonic::Request<GetLatestValidatorSetRequest>,
    ) -> Result<tonic::Response<GetLatestValidatorSetResponse>, Status> {
        todo!()
    }

    async fn get_validator_set_by_height(
        &self,
        req: tonic::Request<GetValidatorSetByHeightRequest>,
    ) -> Result<tonic::Response<GetValidatorSetByHeightResponse>, Status> {
        todo!()
    }
}
