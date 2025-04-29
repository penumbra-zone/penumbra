use anyhow::Context;
use async_trait::async_trait;
use futures::StreamExt;
use ibc_proto::ibc::applications::transfer::v1::query_server::Query as TransferQuery;
use ibc_proto::ibc::apps::transfer::v1::{
    DenomTrace, QueryDenomHashRequest, QueryDenomHashResponse, QueryDenomTraceRequest,
    QueryDenomTraceResponse, QueryDenomTracesRequest, QueryDenomTracesResponse,
    QueryEscrowAddressRequest, QueryEscrowAddressResponse, QueryParamsRequest, QueryParamsResponse,
    QueryTotalEscrowForDenomRequest, QueryTotalEscrowForDenomResponse,
};
use penumbra_sdk_asset::asset::Metadata;
use penumbra_sdk_proto::StateReadProto as _;

use crate::state_key;

use super::Server;

#[async_trait]
impl TransferQuery for Server {
    async fn total_escrow_for_denom(
        &self,
        _: tonic::Request<QueryTotalEscrowForDenomRequest>,
    ) -> std::result::Result<tonic::Response<QueryTotalEscrowForDenomResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn escrow_address(
        &self,
        _: tonic::Request<QueryEscrowAddressRequest>,
    ) -> std::result::Result<tonic::Response<QueryEscrowAddressResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn denom_hash(
        &self,
        _: tonic::Request<QueryDenomHashRequest>,
    ) -> std::result::Result<tonic::Response<QueryDenomHashResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn params(
        &self,
        _: tonic::Request<QueryParamsRequest>,
    ) -> std::result::Result<tonic::Response<QueryParamsResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn denom_trace(
        &self,
        _: tonic::Request<QueryDenomTraceRequest>,
    ) -> std::result::Result<tonic::Response<QueryDenomTraceResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn denom_traces(
        &self,
        _: tonic::Request<QueryDenomTracesRequest>,
    ) -> std::result::Result<tonic::Response<QueryDenomTracesResponse>, tonic::Status> {
        // TODO: Currently pagination is ignored and all denom traces are returned at once.
        // Since this API isn't streaming, this may be something useful to implement later.
        let snapshot = self.storage.latest_snapshot();
        let s = snapshot.prefix(state_key::denom_metadata_by_asset::prefix());
        let denom_traces = s
            .filter_map(move |i: anyhow::Result<(String, Metadata)>| async move {
                if i.is_err() {
                    return Some(Err(i.context("bad denom in state").err().unwrap()));
                }
                let (_key, denom) = i.expect("should not be an error");

                // Convert the key to an IBC asset path
                match denom.best_effort_ibc_transfer_parse() {
                    None => return None,
                    Some((path, base_denom)) => Some(Ok(DenomTrace { path, base_denom })),
                }
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(QueryDenomTracesResponse {
            denom_traces,
            // pagination disabled for now
            pagination: None,
        }))
    }
}
