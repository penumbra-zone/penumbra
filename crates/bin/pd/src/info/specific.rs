use std::pin::Pin;
use std::sync::Arc;

use async_stream::try_stream;
use futures::StreamExt;
use futures::TryStreamExt;
use penumbra_app::governance::StateReadExt as _;
use penumbra_chain::component::AppHashRead;
use penumbra_chain::component::StateReadExt as _;
use penumbra_crypto::asset::{self};
use penumbra_dex::component::router::RouteAndFill;
use penumbra_dex::component::router::RoutingParams;
use penumbra_dex::{
    component::{PositionRead, StateReadExt},
    lp::{position, position::Position},
    DirectedTradingPair, SwapExecution, TradingPair,
};
use penumbra_proto::{
    self as proto,
    client::v1alpha1::{
        specific_query_service_server::SpecificQueryService, BatchSwapOutputDataRequest,
        DenomMetadataByIdRequest, KeyValueRequest, KeyValueResponse, ProposalInfoRequest,
        ProposalInfoResponse, ProposalRateDataRequest, ProposalRateDataResponse,
        ValidatorStatusRequest,
    },
    StateReadProto as _,
};
use penumbra_sct::component::StateReadExt as _;
use penumbra_shielded_pool::component::SupplyRead as _;
use penumbra_stake::rate::RateData;
use penumbra_stake::StateReadExt as _;

use penumbra_proto::DomainType;
use penumbra_storage::StateDelta;
use penumbra_storage::StateRead;
use proto::client::v1alpha1::simulate_trade_request::routing;
use proto::client::v1alpha1::simulate_trade_request::routing::Setting;
use proto::client::v1alpha1::simulate_trade_request::Routing;
use proto::client::v1alpha1::ArbExecutionRequest;
use proto::client::v1alpha1::ArbExecutionResponse;
use proto::client::v1alpha1::ArbExecutionsRequest;
use proto::client::v1alpha1::ArbExecutionsResponse;
use proto::client::v1alpha1::BatchSwapOutputDataResponse;
use proto::client::v1alpha1::DenomMetadataByIdResponse;
use proto::client::v1alpha1::LiquidityPositionByIdRequest;
use proto::client::v1alpha1::LiquidityPositionByIdResponse;
use proto::client::v1alpha1::LiquidityPositionsByIdRequest;
use proto::client::v1alpha1::LiquidityPositionsByIdResponse;
use proto::client::v1alpha1::LiquidityPositionsByPriceRequest;
use proto::client::v1alpha1::LiquidityPositionsByPriceResponse;
use proto::client::v1alpha1::LiquidityPositionsRequest;
use proto::client::v1alpha1::LiquidityPositionsResponse;
use proto::client::v1alpha1::NextValidatorRateRequest;
use proto::client::v1alpha1::NextValidatorRateResponse;
use proto::client::v1alpha1::PrefixValueRequest;
use proto::client::v1alpha1::PrefixValueResponse;
use proto::client::v1alpha1::SimulateTradeRequest;
use proto::client::v1alpha1::SimulateTradeResponse;
use proto::client::v1alpha1::SpreadRequest;
use proto::client::v1alpha1::SpreadResponse;
use proto::client::v1alpha1::SwapExecutionRequest;
use proto::client::v1alpha1::SwapExecutionResponse;
use proto::client::v1alpha1::SwapExecutionsRequest;
use proto::client::v1alpha1::SwapExecutionsResponse;
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
    type LiquidityPositionsByPriceStream = Pin<
        Box<
            dyn futures::Stream<Item = Result<LiquidityPositionsByPriceResponse, tonic::Status>>
                + Send,
        >,
    >;
    type LiquidityPositionsByIdStream = Pin<
        Box<
            dyn futures::Stream<Item = Result<LiquidityPositionsByIdResponse, tonic::Status>>
                + Send,
        >,
    >;
    type ArbExecutionsStream =
        Pin<Box<dyn futures::Stream<Item = Result<ArbExecutionsResponse, tonic::Status>> + Send>>;
    type SwapExecutionsStream =
        Pin<Box<dyn futures::Stream<Item = Result<SwapExecutionsResponse, tonic::Status>> + Send>>;

    async fn simulate_trade(
        &self,
        request: tonic::Request<SimulateTradeRequest>,
    ) -> Result<tonic::Response<SimulateTradeResponse>, Status> {
        let request = request.into_inner();
        let routing_stategy = match request.routing {
            None => Routing {
                setting: Some(Setting::Default(routing::Default {})),
            },
            Some(routing) => routing,
        };

        let routing_strategy = match routing_stategy.setting {
            None => Setting::Default(routing::Default {}),
            Some(setting) => setting,
        };

        let input: penumbra_crypto::Value = request
            .input
            .ok_or_else(|| tonic::Status::invalid_argument("missing input parameter"))?
            .try_into()
            .map_err(|e| {
                tonic::Status::invalid_argument(format!("error parsing input: {:#}", e))
            })?;

        let output_id = request
            .output
            .ok_or_else(|| tonic::Status::invalid_argument("missing output id parameter"))?
            .try_into()
            .map_err(|e| {
                tonic::Status::invalid_argument(format!("error parsing output id: {:#}", e))
            })?;

        let routing_params = match routing_strategy {
            Setting::Default(_) => RoutingParams::default(),
            Setting::SingleHop(_) => RoutingParams {
                max_hops: 1,
                ..RoutingParams::default()
            },
        };

        let state = self.storage.latest_snapshot();
        let mut state_tx = Arc::new(StateDelta::new(state));
        let swap_execution = state_tx
            .route_and_fill(input.asset_id, output_id, input.amount, routing_params)
            .await
            .map_err(|e| tonic::Status::internal(format!("error simulating trade: {:#}", e)))?;

        Ok(tonic::Response::new(SimulateTradeResponse {
            output: Some(swap_execution.into()),
        }))
    }

    async fn spread(
        &self,
        request: tonic::Request<SpreadRequest>,
    ) -> Result<tonic::Response<SpreadResponse>, Status> {
        let state = self.storage.latest_snapshot();
        let request = request.into_inner();

        let pair: TradingPair = request
            .trading_pair
            .ok_or_else(|| tonic::Status::invalid_argument(format!("missing trading pair")))?
            .try_into()
            .map_err(|e| {
                tonic::Status::invalid_argument(format!("error parsing trading pair: {:#}", e))
            })?;

        let pair12 = DirectedTradingPair {
            start: pair.asset_1(),
            end: pair.asset_2(),
        };
        let pair21 = DirectedTradingPair {
            start: pair.asset_2(),
            end: pair.asset_1(),
        };
        let best_1_to_2_position = state.best_position(&pair12).await.map_err(|e| {
            tonic::Status::internal(format!(
                "error finding best position for {:?}: {:#}",
                pair12, e
            ))
        })?;
        let best_2_to_1_position = state.best_position(&pair12).await.map_err(|e| {
            tonic::Status::internal(format!(
                "error finding best position for {:?}: {:#}",
                pair21, e
            ))
        })?;

        let approx_effective_price_1_to_2 = best_1_to_2_position
            .as_ref()
            .map(|p| {
                p.phi
                    .orient_start(pair.asset_1())
                    .expect("position has one end = asset 1")
                    .effective_price()
                    .into()
            })
            .unwrap_or_default();

        let approx_effective_price_2_to_1 = best_2_to_1_position
            .as_ref()
            .map(|p| {
                p.phi
                    .orient_start(pair.asset_2())
                    .expect("position has one end = asset 2")
                    .effective_price()
                    .into()
            })
            .unwrap_or_default();

        Ok(tonic::Response::new(SpreadResponse {
            best_1_to_2_position: best_1_to_2_position.map(Into::into),
            best_2_to_1_position: best_2_to_1_position.map(Into::into),
            approx_effective_price_1_to_2,
            approx_effective_price_2_to_1,
        }))
    }

    #[instrument(skip(self, request))]
    async fn liquidity_positions_by_price(
        &self,
        request: tonic::Request<LiquidityPositionsByPriceRequest>,
    ) -> Result<tonic::Response<Self::LiquidityPositionsByPriceStream>, Status> {
        let state = self.storage.latest_snapshot();
        let request = request.into_inner();

        let pair: DirectedTradingPair = request
            .trading_pair
            .ok_or_else(|| {
                tonic::Status::invalid_argument(format!("missing directed trading pair"))
            })?
            .try_into()
            .map_err(|e| {
                tonic::Status::invalid_argument(format!(
                    "error parsing directed trading pair: {:#}",
                    e
                ))
            })?;

        let limit = if request.limit != 0 {
            request.limit as usize
        } else {
            usize::MAX
        };

        let s = state
            .positions_by_price(&pair)
            .take(limit)
            .and_then(move |id| {
                let state2 = state.clone();
                async move {
                    let position = state2.position_by_id(&id).await?.ok_or_else(|| {
                        anyhow::anyhow!("indexed position not found in state: {}", id)
                    })?;
                    anyhow::Ok(position)
                }
            })
            .map_ok(|position| LiquidityPositionsByPriceResponse {
                data: Some(position.into()),
            })
            .map_err(|e: anyhow::Error| {
                tonic::Status::internal(format!("error retrieving positions: {:#}", e))
            });
        // TODO: how do we instrument a Stream
        Ok(tonic::Response::new(s.boxed()))
    }

    #[instrument(skip(self, request))]
    async fn liquidity_positions(
        &self,
        request: tonic::Request<LiquidityPositionsRequest>,
    ) -> Result<tonic::Response<Self::LiquidityPositionsStream>, Status> {
        let state = self.storage.latest_snapshot();

        let include_closed = request.get_ref().include_closed;
        let s = state.all_positions();
        Ok(tonic::Response::new(
            s.filter(move |item| {
                use penumbra_dex::lp::position::State;
                let keep = match item {
                    Ok(position) => {
                        if position.state == State::Opened {
                            true
                        } else {
                            include_closed
                        }
                    }
                    Err(_) => false,
                };
                futures::future::ready(keep)
            })
            .map_ok(|i: Position| LiquidityPositionsResponse {
                data: Some(i.into()),
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
    async fn liquidity_position_by_id(
        &self,
        request: tonic::Request<LiquidityPositionByIdRequest>,
    ) -> Result<tonic::Response<LiquidityPositionByIdResponse>, Status> {
        let state = self.storage.latest_snapshot();

        let position_id: position::Id = request
            .into_inner()
            .position_id
            .ok_or_else(|| Status::invalid_argument("empty message"))?
            .try_into()
            .map_err(|e: anyhow::Error| {
                tonic::Status::invalid_argument(format!("error converting position_id: {e}"))
            })?;

        let position = state
            .position_by_id(&position_id)
            .await
            .map_err(|e: anyhow::Error| {
                tonic::Status::unavailable(format!("error fetching position from storage: {e}"))
            })?
            .ok_or_else(|| Status::not_found("position not found"))?;

        Ok(tonic::Response::new(LiquidityPositionByIdResponse {
            data: Some(position.into()),
        }))
    }

    #[instrument(skip(self, request))]
    async fn liquidity_positions_by_id(
        &self,
        request: tonic::Request<LiquidityPositionsByIdRequest>,
    ) -> Result<tonic::Response<Self::LiquidityPositionsByIdStream>, Status> {
        let state = self.storage.latest_snapshot();

        let position_ids: Vec<position::Id> = request
            .into_inner()
            .position_id
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(|e: anyhow::Error| {
                tonic::Status::invalid_argument(format!("error converting position_id: {e}"))
            })?;

        let s = try_stream! {
            for position_id in position_ids {
                let position = state
                    .position_by_id(&position_id)
                    .await
                    .map_err(|e: anyhow::Error| {
                        tonic::Status::unavailable(format!("error fetching position from storage: {e}"))
                    })?
                    .ok_or_else(|| Status::not_found("position not found"))?;

                yield position.to_proto();
            }
        };
        Ok(tonic::Response::new(
            s.map_ok(|p: penumbra_proto::core::dex::v1alpha1::Position| {
                LiquidityPositionsByIdResponse { data: Some(p) }
            })
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
    async fn swap_execution(
        &self,
        request: tonic::Request<SwapExecutionRequest>,
    ) -> Result<tonic::Response<SwapExecutionResponse>, Status> {
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

        let swap_execution = state
            .swap_execution(height, trading_pair)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        match swap_execution {
            Some(swap_execution) => Ok(tonic::Response::new(SwapExecutionResponse {
                swap_execution: Some(swap_execution.into()),
            })),
            None => Err(Status::not_found("batch swap output data not found")),
        }
    }

    #[instrument(skip(self, request))]
    async fn swap_executions(
        &self,
        request: tonic::Request<SwapExecutionsRequest>,
    ) -> Result<tonic::Response<Self::SwapExecutionsStream>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let request_inner = request.into_inner();
        let start_height = request_inner.start_height;
        let end_height = request_inner.end_height;
        let trading_pair = request_inner.trading_pair;

        // Convert to domain type ahead of time if necessary
        let trading_pair: Option<DirectedTradingPair> = if let Some(trading_pair) = trading_pair {
            Some(trading_pair.try_into().expect("invalid trading pair"))
        } else {
            None
        };

        use penumbra_dex::state_key;

        let s = state.prefix(&state_key::swap_executions());
        Ok(tonic::Response::new(
            s.filter_map(move |i: anyhow::Result<(String, SwapExecution)>| {
                async move {
                    if i.is_err() {
                        return Some(Err(tonic::Status::unavailable(format!(
                            "error getting prefix value from storage: {}",
                            i.err().unwrap()
                        ))));
                    }

                    let (key, swap_execution) = i.unwrap();
                    let parts = key.split('/').collect::<Vec<_>>();
                    let height = parts[2].parse().expect("height is not a number");
                    let asset_1: asset::Id =
                        parts[3].parse().expect("asset id formatted improperly");
                    let asset_2: asset::Id =
                        parts[4].parse().expect("asset id formatted improperly");

                    let swap_trading_pair = DirectedTradingPair::new(asset_1, asset_2);

                    if let Some(trading_pair) = trading_pair {
                        // filter by trading pair

                        if swap_trading_pair != trading_pair {
                            return None;
                        }
                    }

                    // TODO: would be great to start iteration at start_height
                    // and stop at end_height rather than touching _every_
                    // key, but the current storage implementation doesn't make this
                    // easy.
                    if height < start_height || height > end_height {
                        None
                    } else {
                        Some(Ok(SwapExecutionsResponse {
                            swap_execution: Some(swap_execution.into()),
                            height: height,
                            trading_pair: Some(swap_trading_pair.into()),
                        }))
                    }
                }
            })
            // TODO: how do we instrument a Stream
            //.instrument(Span::current())
            .boxed(),
        ))
    }

    #[instrument(skip(self, request))]
    async fn denom_metadata_by_id(
        &self,
        request: tonic::Request<DenomMetadataByIdRequest>,
    ) -> Result<tonic::Response<DenomMetadataByIdResponse>, Status> {
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
                DenomMetadataByIdResponse {
                    denom_metadata: Some(denom.into()),
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

        use penumbra_app::governance::state_key;

        let s = state.prefix(&state_key::all_rate_data_at_proposal_start(proposal_id));
        Ok(tonic::Response::new(
            s.map_ok(|i: (String, RateData)| {
                let (_key, rate_data) = i;
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

        // TODO(erwan): we are unconditionally generating the proof here; we shouldn't do that if the
        // request doesn't ask for it
        let (some_value, proof) = state
            .get_with_proof_to_apphash(request.key.into_bytes())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(KeyValueResponse {
            value: some_value.map(Into::into),
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

        Ok(tonic::Response::new(
            state
                .prefix_raw(&request.prefix)
                .map_ok(|i: (String, Vec<u8>)| {
                    let (key, value) = i;
                    PrefixValueResponse { key, value }
                })
                .map_err(|e: anyhow::Error| {
                    tonic::Status::unavailable(format!(
                        "error getting prefix value from storage: {e}"
                    ))
                })
                // TODO: how do we instrument a Stream
                //.instrument(Span::current())
                .boxed(),
        ))
    }

    #[instrument(skip(self, request))]
    async fn arb_execution(
        &self,
        request: tonic::Request<ArbExecutionRequest>,
    ) -> Result<tonic::Response<ArbExecutionResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let request_inner = request.into_inner();
        let height = request_inner.height;

        let arb_execution = state
            .arb_execution(height)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        match arb_execution {
            Some(arb_execution) => Ok(tonic::Response::new(ArbExecutionResponse {
                swap_execution: Some(arb_execution.into()),
                height: height.into(),
            })),
            None => Err(Status::not_found("arb execution data not found")),
        }
    }

    #[instrument(skip(self, request))]
    async fn arb_executions(
        &self,
        request: tonic::Request<ArbExecutionsRequest>,
    ) -> Result<tonic::Response<Self::ArbExecutionsStream>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let request_inner = request.into_inner();
        let start_height = request_inner.start_height;
        let end_height = request_inner.end_height;

        use penumbra_dex::state_key;

        let s = state.prefix(&state_key::arb_executions());
        Ok(tonic::Response::new(
            s.filter_map(
                move |i: anyhow::Result<(String, SwapExecution)>| async move {
                    if i.is_err() {
                        return Some(Err(tonic::Status::unavailable(format!(
                            "error getting prefix value from storage: {}",
                            i.err().unwrap()
                        ))));
                    }

                    let (key, arb_execution) = i.unwrap();
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
