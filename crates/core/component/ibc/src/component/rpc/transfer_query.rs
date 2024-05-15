use async_trait::async_trait;
use ibc_proto::ibc::applications::transfer::v1::query_server::Query as TransferQuery;
use ibc_proto::ibc::apps::transfer::v1::{
    QueryDenomHashRequest, QueryDenomHashResponse, QueryDenomTraceRequest, QueryDenomTraceResponse,
    QueryDenomTracesRequest, QueryDenomTracesResponse, QueryEscrowAddressRequest,
    QueryEscrowAddressResponse, QueryParamsRequest, QueryParamsResponse,
    QueryTotalEscrowForDenomRequest, QueryTotalEscrowForDenomResponse,
};

use crate::component::HostInterface;

use super::IbcQuery;

#[async_trait]
impl<HI: HostInterface + Send + Sync + 'static> TransferQuery for IbcQuery<HI> {
    async fn total_escrow_for_denom(
        &self,
        _: tonic::Request<QueryTotalEscrowForDenomRequest>,
    ) -> std::result::Result<tonic::Response<QueryTotalEscrowForDenomResponse>, tonic::Status> {
        unimplemented!()
    }

    async fn escrow_address(
        &self,
        _: tonic::Request<QueryEscrowAddressRequest>,
    ) -> std::result::Result<tonic::Response<QueryEscrowAddressResponse>, tonic::Status> {
        unimplemented!()
    }

    async fn denom_hash(
        &self,
        _: tonic::Request<QueryDenomHashRequest>,
    ) -> std::result::Result<tonic::Response<QueryDenomHashResponse>, tonic::Status> {
        unimplemented!()
    }

    async fn params(
        &self,
        _: tonic::Request<QueryParamsRequest>,
    ) -> std::result::Result<tonic::Response<QueryParamsResponse>, tonic::Status> {
        unimplemented!()
    }

    async fn denom_trace(
        &self,
        _: tonic::Request<QueryDenomTraceRequest>,
    ) -> std::result::Result<tonic::Response<QueryDenomTraceResponse>, tonic::Status> {
        unimplemented!()
    }

    async fn denom_traces(
        &self,
        _: tonic::Request<QueryDenomTracesRequest>,
    ) -> std::result::Result<tonic::Response<QueryDenomTracesResponse>, tonic::Status> {
        unimplemented!()
    }
}
