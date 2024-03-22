use std::{pin::Pin, sync::Arc};

use async_stream::try_stream;
use futures::{StreamExt, TryStreamExt};
use tonic::Status;
use tracing::instrument;

use cnidarium::{StateDelta, Storage};
use penumbra_asset::{asset, Value};
use penumbra_proto::{
    core::component::dex::v1::{
        query_service_server::QueryService, simulate_trade_request::routing,
        simulate_trade_request::routing::Setting, simulate_trade_request::Routing,
        simulation_service_server::SimulationService, ArbExecutionRequest, ArbExecutionResponse,
        ArbExecutionsRequest, ArbExecutionsResponse, BatchSwapOutputDataRequest,
        BatchSwapOutputDataResponse, LiquidityPositionByIdRequest, LiquidityPositionByIdResponse,
        LiquidityPositionsByIdRequest, LiquidityPositionsByIdResponse,
        LiquidityPositionsByPriceRequest, LiquidityPositionsByPriceResponse,
        LiquidityPositionsRequest, LiquidityPositionsResponse, SimulateTradeRequest,
        SimulateTradeResponse, SpreadRequest, SpreadResponse, SwapExecutionRequest,
        SwapExecutionResponse, SwapExecutionsRequest, SwapExecutionsResponse,
    },
    DomainType, StateReadProto,
};

use crate::ExecutionCircuitBreaker;
use crate::{
    lp::position::{self, Position},
    state_key, DirectedTradingPair, SwapExecution, TradingPair,
};

use super::{router::RouteAndFill, PositionRead, StateReadExt};

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

    #[instrument(skip(self, request))]
    async fn arb_execution(
        &self,
        request: tonic::Request<ArbExecutionRequest>,
    ) -> Result<tonic::Response<ArbExecutionResponse>, Status> {
        let state = self.storage.latest_snapshot();
        let request_inner = request.into_inner();
        let height = request_inner.height;

        let arb_execution = state
            .arb_execution(height)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        match arb_execution {
            Some(arb_execution) => Ok(tonic::Response::new(ArbExecutionResponse {
                swap_execution: Some(arb_execution.into()),
                height,
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
        let request_inner = request.into_inner();
        let start_height = request_inner.start_height;
        let end_height = request_inner.end_height;

        let s = state.prefix(state_key::arb_executions());
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

    #[instrument(skip(self, request))]
    /// Get the batch swap data associated with a given trading pair and height.
    async fn batch_swap_output_data(
        &self,
        request: tonic::Request<BatchSwapOutputDataRequest>,
    ) -> Result<tonic::Response<BatchSwapOutputDataResponse>, Status> {
        let state = self.storage.latest_snapshot();

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

        let request_inner = request.into_inner();
        let start_height = request_inner.start_height;
        let end_height = request_inner.end_height;
        let trading_pair = request_inner.trading_pair;

        // Convert to domain type ahead of time if necessary
        let trading_pair: Option<DirectedTradingPair> =
            trading_pair.map(|trading_pair| trading_pair.try_into().expect("invalid trading pair"));

        let s = state.prefix(state_key::swap_executions());
        Ok(tonic::Response::new(
            s.filter_map(move |i: anyhow::Result<(String, SwapExecution)>| {
                async move {
                    if i.is_err() {
                        return Some(Err(tonic::Status::unavailable(format!(
                            "error getting prefix value from storage: {}",
                            i.expect_err("i is_err")
                        ))));
                    }

                    let (key, swap_execution) = i.expect("i is Ok");
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
                            height,
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

    async fn spread(
        &self,
        request: tonic::Request<SpreadRequest>,
    ) -> Result<tonic::Response<SpreadResponse>, Status> {
        let state = self.storage.latest_snapshot();
        let request = request.into_inner();

        let pair: TradingPair = request
            .trading_pair
            .ok_or_else(|| tonic::Status::invalid_argument("missing trading pair"))?
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
            .ok_or_else(|| tonic::Status::invalid_argument("missing directed trading pair"))?
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
                use crate::lp::position::State;
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
            s.map_ok(|p: penumbra_proto::core::component::dex::v1::Position| {
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
}

#[tonic::async_trait]
impl SimulationService for Server {
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

        let input: Value = request
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

        let state = self.storage.latest_snapshot();

        let mut routing_params = state.routing_params().await.unwrap();
        match routing_strategy {
            Setting::SingleHop(_) => {
                routing_params.max_hops = 1;
            }
            Setting::Default(_) => {
                // no-op, use the default
            }
        }

        let mut state_tx = Arc::new(StateDelta::new(state));
        let execution_circuit_breaker = ExecutionCircuitBreaker::default();
        let swap_execution = state_tx
            .route_and_fill(
                input.asset_id,
                output_id,
                input.amount,
                routing_params,
                execution_circuit_breaker,
            )
            .await
            .map_err(|e| tonic::Status::internal(format!("error simulating trade: {:#}", e)))?;

        let unfilled = Value {
            amount: input
                .amount
                .checked_sub(&swap_execution.input.amount)
                .ok_or_else(|| {
                    tonic::Status::failed_precondition(
                        "swap execution input amount is larger than request input amount"
                            .to_string(),
                    )
                })?,
            asset_id: input.asset_id,
        };

        Ok(tonic::Response::new(SimulateTradeResponse {
            unfilled: Some(unfilled.into()),
            output: Some(swap_execution.into()),
        }))
    }
}
