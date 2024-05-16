use async_trait::async_trait;
use ibc_proto::ibc::applications::transfer::v1::query_server::Query as TransferQuery;
use ibc_proto::ibc::apps::transfer::v1::{
    QueryDenomHashRequest, QueryDenomHashResponse, QueryDenomTraceRequest, QueryDenomTraceResponse,
    QueryDenomTracesRequest, QueryDenomTracesResponse, QueryEscrowAddressRequest,
    QueryEscrowAddressResponse, QueryParamsRequest, QueryParamsResponse,
    QueryTotalEscrowForDenomRequest, QueryTotalEscrowForDenomResponse,
};
use penumbra_proto::StateReadProto as _;
use penumbra_shielded_pool::state_key;

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
        let snapshot = self.storage.latest_snapshot();
        let s = snapshot.prefix(state_key::denom_by_asset::prefix());
        Ok(tonic::Response::new(
            s.filter_map(
                move |i: anyhow::Result<(String, SwapExecution)>| async move {
                    if i.is_err() {
                        return Some(Err(tonic::Status::unavailable(format!(
                            "error getting prefix value from storage: {}",
                            i.expect_err("i is_err")
                        ))));
                    }

                    let (key, arb_execution) = i.expect("i is Ok");
                    let height = key
                        .split('/')
                        .last()
                        .expect("arb execution key has height as last part")
                        .parse()
                        .expect("height is a number");

                    // TODO: would be great to start iteration at start_height
                    // and stop at end_height rather than touching _every_
                    // key, but the current storage implementation doesn't make this
                    // easy.
                    if height < start_height || height > end_height {
                        None
                    } else {
                        Some(Ok(ArbExecutionsResponse {
                            swap_execution: Some(arb_execution.into()),
                            height,
                        }))
                    }
                },
            )
            // TODO: how do we instrument a Stream
            //.instrument(Span::current())
            .boxed(),
        ))
    }
}
