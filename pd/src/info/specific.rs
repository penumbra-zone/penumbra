use std::pin::Pin;

use async_stream::try_stream;
use futures::StreamExt;
use futures::TryStreamExt;
use penumbra_chain::AppHashRead;
use penumbra_chain::StateReadExt as _;
use penumbra_component::dex::PositionRead;
use penumbra_component::governance::StateReadExt as _;
use penumbra_component::shielded_pool::StateReadExt as _;
use penumbra_component::shielded_pool::SupplyRead as _;
use penumbra_component::stake::rate::RateData;
use penumbra_component::stake::StateReadExt as _;
use penumbra_component::stubdex::StateReadExt as _;
use penumbra_crypto::asset::{self, Asset};
use penumbra_proto::{
    self as proto,
    client::v1alpha1::{
        specific_query_service_server::SpecificQueryService, AssetInfoRequest, AssetInfoResponse,
        BatchSwapOutputDataRequest, KeyValueRequest, KeyValueResponse, ProposalInfoRequest,
        ProposalInfoResponse, ProposalRateDataRequest, ProposalRateDataResponse,
        StubCpmmReservesRequest, ValidatorStatusRequest,
    },
    StateReadProto as _,
};

use penumbra_storage::StateRead;
use proto::client::v1alpha1::BatchSwapOutputDataResponse;
use proto::client::v1alpha1::LiquidityPositionsRequest;
use proto::client::v1alpha1::LiquidityPositionsResponse;
use proto::client::v1alpha1::NextValidatorRateRequest;
use proto::client::v1alpha1::NextValidatorRateResponse;
use proto::client::v1alpha1::PrefixValueRequest;
use proto::client::v1alpha1::PrefixValueResponse;
use proto::client::v1alpha1::StubCpmmReservesResponse;
use proto::client::v1alpha1::TransactionByNoteRequest;
use proto::client::v1alpha1::TransactionByNoteResponse;
use proto::client::v1alpha1::ValidatorPenaltyRequest;
use proto::client::v1alpha1::ValidatorPenaltyResponse;
use proto::client::v1alpha1::ValidatorStatusResponse;
use tonic::Status;
use tracing::instrument;

// We need to use the tracing-futures version of Instrument,
// because we want to instrument a Stream, and the Stream trait
// isn't in std, and the tracing::Instrument trait only works with
// (stable) std types.
//use tracing_futures::Instrument;

use super::Info;

#[tonic::async_trait]
impl SpecificQueryService for Info {
    type LiquidityPositionsStream = Pin<
        Box<dyn futures::Stream<Item = Result<LiquidityPositionsResponse, tonic::Status>> + Send>,
    >;

    #[instrument(skip(self, request))]
    async fn liquidity_positions(
        &self,
        request: tonic::Request<LiquidityPositionsRequest>,
    ) -> Result<tonic::Response<Self::LiquidityPositionsStream>, Status> {
        let state = self.storage.latest_snapshot();

        let stream_iter = state.all_positions().next().await.into_iter();
        let s = try_stream! {
            for item in stream_iter
                .map(|item| item.map_err(|e| tonic::Status::internal(e.to_string()))) {
                    let item = item.unwrap();
                    if (request.get_ref().only_open && item.state == penumbra_crypto::dex::lp::position::State::Opened) || request.get_ref().only_open == false {
                        yield LiquidityPositionsResponse { data: Some(item.into()) }
                    }
                }
        };

        Ok(tonic::Response::new(
            s.map_err(|e: anyhow::Error| {
                tonic::Status::unavailable(format!("error getting prefix value from storage: {e}"))
            })
            // TODO: how do we instrument a Stream
            //.instrument(Span::current())
            .boxed(),
        ))
    }

    #[instrument(skip(self, request))]
    async fn transaction_by_note(
        &self,
        request: tonic::Request<TransactionByNoteRequest>,
    ) -> Result<tonic::Response<TransactionByNoteResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let cm = request
            .into_inner()
            .note_commitment
            .ok_or_else(|| Status::invalid_argument("empty message"))?
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid commitment"))?;
        let source = state
            .note_source(cm)
            .await
            .map_err(|e| Status::unavailable(format!("error getting note source: {e}")))?
            .ok_or_else(|| Status::not_found("note commitment not found"))?;
        tracing::debug!(?cm, ?source);

        Ok(tonic::Response::new(TransactionByNoteResponse {
            note_source: Some(source.into()),
        }))
    }
    #[instrument(skip(self, request))]
    async fn validator_status(
        &self,
        request: tonic::Request<ValidatorStatusRequest>,
    ) -> Result<tonic::Response<ValidatorStatusResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;

        let id = request
            .into_inner()
            .identity_key
            .ok_or_else(|| Status::invalid_argument("missing identity key"))?
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid identity key"))?;

        let status = state
            .validator_status(&id)
            .await
            .map_err(|e| Status::unavailable(format!("error getting validator status: {e}")))?
            .ok_or_else(|| Status::not_found("validator not found"))?;

        Ok(tonic::Response::new(ValidatorStatusResponse {
            status: Some(status.into()),
        }))
    }

    #[instrument(skip(self, request))]
    async fn validator_penalty(
        &self,
        request: tonic::Request<ValidatorPenaltyRequest>,
    ) -> Result<tonic::Response<ValidatorPenaltyResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;

        let request = request.into_inner();
        let id = request
            .identity_key
            .ok_or_else(|| Status::invalid_argument("missing identity key"))?
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid identity key"))?;

        let penalty = state
            .compounded_penalty_over_range(&id, request.start_epoch_index, request.end_epoch_index)
            .await
            .map_err(|e| Status::unavailable(format!("error getting validator penalty: {e}")))?;

        Ok(tonic::Response::new(ValidatorPenaltyResponse {
            penalty: Some(penalty.into()),
        }))
    }

    #[instrument(skip(self, request))]
    async fn next_validator_rate(
        &self,
        request: tonic::Request<NextValidatorRateRequest>,
    ) -> Result<tonic::Response<NextValidatorRateResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let identity_key = request
            .into_inner()
            .identity_key
            .ok_or_else(|| tonic::Status::invalid_argument("empty message"))?
            .try_into()
            .map_err(|_| tonic::Status::invalid_argument("invalid identity key"))?;

        let rate_data = state
            .next_validator_rate(&identity_key)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        match rate_data {
            Some(r) => Ok(tonic::Response::new(NextValidatorRateResponse {
                data: Some(r.into()),
            })),
            None => Err(Status::not_found("next validator rate not found")),
        }
    }

    #[instrument(skip(self, request))]
    /// Get the batch swap data associated with a given trading pair and height.
    async fn batch_swap_output_data(
        &self,
        request: tonic::Request<BatchSwapOutputDataRequest>,
    ) -> Result<tonic::Response<BatchSwapOutputDataResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let request_inner = request.into_inner();
        let height = request_inner.height;
        let trading_pair = request_inner
            .trading_pair
            .ok_or_else(|| Status::invalid_argument("missing trading_pair"))?
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid trading_pair"))?;

        let output_data = state
            .output_data(height, trading_pair)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        match output_data {
            Some(data) => Ok(tonic::Response::new(BatchSwapOutputDataResponse {
                data: Some(data.into()),
            })),
            None => Err(Status::not_found("batch swap output data not found")),
        }
    }

    #[instrument(skip(self, request))]
    /// Get the batch swap data associated with a given trading pair and height.
    async fn stub_cpmm_reserves(
        &self,
        request: tonic::Request<StubCpmmReservesRequest>,
    ) -> Result<tonic::Response<StubCpmmReservesResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let request_inner = request.into_inner();
        let trading_pair = request_inner
            .trading_pair
            .ok_or_else(|| Status::invalid_argument("missing trading_pair"))?
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid trading_pair"))?;

        let cpmm_reserves = state
            .stub_cpmm_reserves(&trading_pair)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        match cpmm_reserves {
            Some(reserves) => Ok(tonic::Response::new(StubCpmmReservesResponse {
                reserves: Some(reserves.into()),
            })),
            None => Err(Status::not_found("CPMM reserves not found")),
        }
    }

    #[instrument(skip(self, request))]
    async fn asset_info(
        &self,
        request: tonic::Request<AssetInfoRequest>,
    ) -> Result<tonic::Response<AssetInfoResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;

        let request = request.into_inner();
        let id: asset::Id = request
            .asset_id
            .ok_or_else(|| Status::invalid_argument("missing asset_id"))?
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("could not parse asset_id: {e}")))?;

        let denom = state
            .denom_by_asset(&id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let rsp = match denom {
            Some(denom) => {
                tracing::debug!(?id, ?denom, "found denom");
                AssetInfoResponse {
                    asset: Some(Asset { id, denom }.into()),
                }
            }
            None => {
                tracing::debug!(?id, "unknown asset id");
                Default::default()
            }
        };

        Ok(tonic::Response::new(rsp))
    }

    #[instrument(skip(self, request))]
    async fn proposal_info(
        &self,
        request: tonic::Request<ProposalInfoRequest>,
    ) -> Result<tonic::Response<ProposalInfoResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let proposal_id = request.into_inner().proposal_id;

        let start_block_height = state
            .proposal_voting_start(proposal_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or_else(|| tonic::Status::unknown(format!("proposal {proposal_id} not found")))?;

        let start_position = state
            .proposal_voting_start_position(proposal_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or_else(|| tonic::Status::unknown(format!("proposal {proposal_id} not found")))?;

        Ok(tonic::Response::new(ProposalInfoResponse {
            start_block_height,
            start_position: start_position.into(),
        }))
    }

    type ProposalRateDataStream = Pin<
        Box<dyn futures::Stream<Item = Result<ProposalRateDataResponse, tonic::Status>> + Send>,
    >;

    #[instrument(skip(self, request))]
    async fn proposal_rate_data(
        &self,
        request: tonic::Request<ProposalRateDataRequest>,
    ) -> Result<tonic::Response<Self::ProposalRateDataStream>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let proposal_id = request.into_inner().proposal_id;

        use penumbra_component::governance::state_key;
        let stream_iter = state
            .prefix(&state_key::all_rate_data_at_proposal_start(proposal_id))
            .next()
            .await
            .into_iter();
        let s = try_stream! {
            for item in stream_iter
                .map(|item| item.map_err(|e| tonic::Status::internal(e.to_string()))) {
                    yield item
                }
        };

        Ok(tonic::Response::new(
            s.map_ok(|i: Result<(String, RateData), tonic::Status>| {
                let (_key, rate_data) = i.unwrap();
                ProposalRateDataResponse {
                    rate_data: Some(rate_data.into()),
                }
            })
            .map_err(|e: anyhow::Error| {
                tonic::Status::unavailable(format!("error getting prefix value from storage: {e}"))
            })
            // TODO: how do we instrument a Stream
            //.instrument(Span::current())
            .boxed(),
        ))
    }

    #[instrument(skip(self, request))]
    async fn key_value(
        &self,
        request: tonic::Request<KeyValueRequest>,
    ) -> Result<tonic::Response<KeyValueResponse>, Status> {
        let state = self.storage.latest_snapshot();
        // We map the error here to avoid including `tonic` as a dependency
        // in the `chain` crate, to support its compilation to wasm.
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;

        let request = request.into_inner();
        tracing::debug!(?request);

        if request.key.is_empty() {
            return Err(Status::invalid_argument("key is empty"));
        }

        // TODO: we are unconditionally generating the proof here; we shouldn't do that if the
        // request doesn't ask for it
        let (value, proof) = state
            .get_with_proof_to_apphash(request.key.into_bytes())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(KeyValueResponse {
            value,
            proof: if request.proof {
                Some(ibc_proto::ibc::core::commitment::v1::MerkleProof {
                    proofs: proof
                        .proofs
                        .into_iter()
                        .map(|p| {
                            let mut encoded = Vec::new();
                            prost::Message::encode(&p, &mut encoded).unwrap();
                            prost::Message::decode(&*encoded).unwrap()
                        })
                        .collect(),
                })
            } else {
                None
            },
        }))
    }

    type PrefixValueStream =
        Pin<Box<dyn futures::Stream<Item = Result<PrefixValueResponse, tonic::Status>> + Send>>;

    #[instrument(skip(self, request))]
    async fn prefix_value(
        &self,
        request: tonic::Request<PrefixValueRequest>,
    ) -> Result<tonic::Response<Self::PrefixValueStream>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let request = request.into_inner();
        tracing::debug!(?request);

        if request.prefix.is_empty() {
            return Err(Status::invalid_argument("prefix is empty"));
        }

        let stream_iter = state.prefix_raw(&request.prefix).next().await.into_iter();
        let s = try_stream! {
            for item in stream_iter
                .map(|item| item.map_err(|e| tonic::Status::internal(e.to_string()))) {
                    yield item
                }
        };

        Ok(tonic::Response::new(
            s.map_ok(|i: Result<(String, Vec<u8>), tonic::Status>| {
                let (key, value) = i.unwrap();
                PrefixValueResponse { key, value }
            })
            .map_err(|e: anyhow::Error| {
                tonic::Status::unavailable(format!("error getting prefix value from storage: {e}"))
            })
            // TODO: how do we instrument a Stream
            //.instrument(Span::current())
            .boxed(),
        ))
    }
}
