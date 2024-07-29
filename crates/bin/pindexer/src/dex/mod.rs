use std::collections::HashSet;

use anyhow::{anyhow, Context};
use cometindex::async_trait;
use penumbra_asset::asset::Id as AssetId;
use penumbra_dex::lp::position::{Id, Position};
use penumbra_dex::lp::{self, TradingFunction};
use penumbra_dex::{DirectedTradingPair, SwapExecution};
use penumbra_num::Amount;
use penumbra_proto::core::component::dex::v1::BatchSwapOutputData;
use penumbra_proto::{event::ProtoEvent, penumbra::core::component::dex::v1 as pb};
use sqlx::{PgPool, Postgres, Transaction};

use crate::sql::Sql;
use crate::{AppView, ContextualizedEvent, PgTransaction};

/// One of the possible events that we care about.
#[derive(Clone, Debug)]
enum Event {
    /// A parsed version of [pb::EventValueCircuitBreakerCredit].
    CircuitBreakerCredit {
        asset_id: AssetId,
        previous_balance: Amount,
        new_balance: Amount,
    },
    /// A parsed version of [pb::EventValueCircuitBreakerDebit]
    CircuitBreakerDebit {
        asset_id: AssetId,
        previous_balance: Amount,
        new_balance: Amount,
    },
    /// A parsed version of [pb::EventArbExecution]
    ArbExecution {
        height: u64,
        execution: SwapExecution,
    },
    /// A parsed version of [pb::EventBatchSwap]
    Swap {
        height: u64,
        execution: SwapExecution,
    },
    /// A parsed version of [pb::EventPositionOpen]
    PositionOpen { height: u64, position: Position },
    /// A parsed version of [pb::EventPositionWithdraw]
    PositionWithdraw {
        height: u64,
        position_id: Id,
        reserves_1: Amount,
        reserves_2: Amount,
        sequence: u64,
    },
    /// A parsed version of [pb::EventPositionClose]
    PositionClose { height: u64, position_id: Id },
    /// A parsed version of [pb::EventPositionExecution]
    PositionExecution {
        height: u64,
        position_id: Id,
        reserves_1: Amount,
        reserves_2: Amount,
        prev_reserves_1: Amount,
        prev_reserves_2: Amount,
        context: DirectedTradingPair,
    },
}

impl Event {
    const NAMES: [&'static str; 8] = [
        "penumbra.core.component.dex.v1.EventValueCircuitBreakerCredit",
        "penumbra.core.component.dex.v1.EventValueCircuitBreakerDebit",
        "penumbra.core.component.dex.v1.EventArbExecution",
        "penumbra.core.component.dex.v1.EventBatchSwap",
        "penumbra.core.component.dex.v1.EventPositionWithdraw",
        "penumbra.core.component.dex.v1.EventPositionOpen",
        "penumbra.core.component.dex.v1.EventPositionClose",
        "penumbra.core.component.dex.v1.EventPositionExecution",
    ];

    /// Index this event, using the handle to the postgres transaction.
    async fn index<'d>(&self, dbtx: &mut Transaction<'d, Postgres>) -> anyhow::Result<()> {
        match self {
            Event::CircuitBreakerCredit {
                asset_id,
                previous_balance,
                new_balance,
            } => {
                let amount = new_balance.checked_sub(&previous_balance).ok_or(anyhow!(
                    "balance decreased after dex credit: previous: {}, new: {}",
                    previous_balance,
                    new_balance
                ))?;
                sqlx::query(
                    r#"
                INSERT INTO dex_value_circuit_breaker_change
                VALUES ($1, CAST($2 AS Amount));
                "#,
                )
                .bind(Sql::from(*asset_id))
                .bind(amount.to_string())
                .execute(dbtx.as_mut())
                .await?;
                Ok(())
            }
            Event::CircuitBreakerDebit {
                asset_id,
                previous_balance,
                new_balance,
            } => {
                let amount = previous_balance.checked_sub(&new_balance).ok_or(anyhow!(
                    "balance increased after dex credit: previous: {}, new: {}",
                    previous_balance,
                    new_balance
                ))?;
                sqlx::query(
                    r#"
                INSERT INTO dex_value_circuit_breaker_change
                VALUES ($1, -(CAST($2 AS Amount)));
                "#,
                )
                .bind(Sql::from(*asset_id))
                .bind(amount.to_string())
                .execute(dbtx.as_mut())
                .await?;
                Ok(())
            }
            Event::ArbExecution { height, execution } => {
                let mut trace_start = None;
                let mut trace_end = None;
                for trace in &execution.traces {
                    let mut step_start = None;
                    let mut step_end = None;
                    for step in trace {
                        let (id,): (i32,) = sqlx::query_as(
                            r#"INSERT INTO dex_trace_step VALUES (DEFAULT, (CAST($1 AS Amount), $2)) RETURNING id;"#,
                        )
                        .bind(step.amount.to_string())
                        .bind(Sql::from(step.asset_id))
                        .fetch_one(dbtx.as_mut())
                        .await?;
                        if let None = step_start {
                            step_start = Some(id);
                        }
                        step_end = Some(id);
                    }
                    let (id,): (i32,) = sqlx::query_as(
                        r#"INSERT INTO dex_trace VALUES (DEFAULT, $1, $2) RETURNING id;"#,
                    )
                    .bind(step_start)
                    .bind(step_end)
                    .fetch_one(dbtx.as_mut())
                    .await?;
                    if let None = trace_start {
                        trace_start = Some(id);
                    }
                    trace_end = Some(id);
                }
                sqlx::query(r#"INSERT INTO dex_arb VALUES ($1, (CAST($2 AS Amount), $3), (CAST($4 AS Amount), $5), $6, $7);"#)
                    .bind(i64::try_from(*height)?)
                    .bind(execution.input.amount.to_string())
                    .bind(Sql::from(execution.input.asset_id))
                    .bind(execution.output.amount.to_string())
                    .bind(Sql::from(execution.output.asset_id))
                    .bind(trace_start)
                    .bind(trace_end)
                    .execute(dbtx.as_mut())
                    .await?;
                Ok(())
            }
            Event::PositionOpen { height, position } => {
                let Position {
                    state,
                    phi: TradingFunction { pair, component },
                    ..
                } = position;
                let id = position.id().0;
                tracing::debug!(
                    p = component.p.to_string(),
                    q = component.q.to_string(),
                    r1 = position.reserves_1().amount.to_string(),
                    r2 = position.reserves_2().amount.to_string()
                );
                sqlx::query(
                    "
                INSERT INTO dex_lp (id, state, asset1, asset2, p, q, fee_bps, close_on_fill, reserves1, reserves2)
                VALUES ($1, $2, $3, $4, CAST($5 as Amount), CAST($6 AS Amount), $7, $8, CAST($9 AS Amount), CAST($10 AS Amount));
                ",
                )
                .bind(id)
                .bind(state.to_string())
                .bind(pair.asset_1().to_bytes())
                .bind(pair.asset_2().to_bytes())
                .bind(component.p.to_string())
                .bind(component.q.to_string())
                .bind(i32::try_from(component.fee)?)
                .bind(position.close_on_fill)
                .bind(position.reserves_1().amount.to_string())
                .bind(position.reserves_2().amount.to_string())
                .execute(dbtx.as_mut())
                .await?;
                sqlx::query(
                    "
                INSERT INTO dex_lp_update (height, position_id, state, reserves1, reserves2)
                VALUES ($1, $2, $3, CAST($4 AS Amount), CAST($5 AS Amount));
                ",
                )
                .bind(i64::try_from(*height)?)
                .bind(id)
                .bind(state.to_string())
                .bind(position.reserves_1().amount.to_string())
                .bind(position.reserves_2().amount.to_string())
                .execute(dbtx.as_mut())
                .await?;
                Ok(())
            }
            Event::PositionClose {
                height,
                position_id,
            } => {
                let state = lp::position::State::Closed;
                sqlx::query(
                    "
                INSERT INTO dex_lp_update (height, position_id, state)
                VALUES ($1, $2, $3)
                ",
                )
                .bind(i64::try_from(*height)?)
                .bind(position_id.0)
                .bind(state.to_string())
                .execute(dbtx.as_mut())
                .await?;
                sqlx::query(
                    "
                UPDATE dex_lp
                SET state = $2
                WHERE id = $1;
                ",
                )
                .bind(position_id.0)
                .bind(state.to_string())
                .execute(dbtx.as_mut())
                .await?;
                Ok(())
            }
            Event::PositionWithdraw {
                height,
                position_id,
                reserves_1: reserves1,
                reserves_2: reserves2,
                sequence,
            } => {
                let state = lp::position::State::Withdrawn {
                    sequence: *sequence,
                };
                let reserves1 = reserves1.to_string();
                let reserves2 = reserves2.to_string();
                sqlx::query(
                    "
                INSERT INTO dex_lp_update (height, position_id, state, reserves1, reserves2)
                VALUES ($1, $2, $3, CAST($4 AS Amount), CAST($5 AS Amount))
                ",
                )
                .bind(i64::try_from(*height)?)
                .bind(position_id.0)
                .bind(state.to_string())
                .bind(&reserves1)
                .bind(&reserves2)
                .execute(dbtx.as_mut())
                .await?;
                sqlx::query(
                    "
                UPDATE dex_lp
                SET state = $2, reserves1 = CAST($3 AS Amount), reserves2 = CAST($4 AS Amount)
                WHERE id = $1;
                ",
                )
                .bind(position_id.0)
                .bind(state.to_string())
                .bind(&reserves1)
                .bind(&reserves2)
                .execute(dbtx.as_mut())
                .await?;
                Ok(())
            }
            Event::PositionExecution {
                height,
                position_id,
                reserves_1: reserves1,
                reserves_2: reserves2,
                context,
                prev_reserves_1: prev_reserves1,
                prev_reserves_2: prev_reserves2,
            } => {
                let state = lp::position::State::Opened;
                let reserves1 = reserves1.to_string();
                let reserves2 = reserves2.to_string();
                let prev_reserves1 = prev_reserves1.to_string();
                let prev_reserves2 = prev_reserves2.to_string();
                let id: i32 = sqlx::query_scalar(
                    "
                INSERT INTO dex_lp_execution (inflow1, inflow2, context_start, context_end)
                VALUES (CAST($1 AS Amount) - CAST($2 AS Amount), CAST($3 AS Amount) - CAST($4 AS Amount), $5, $6)
                RETURNING id;
            ",)
                .bind(&reserves1)
                .bind(prev_reserves1)
                .bind(&reserves2)
                .bind(prev_reserves2)
                .bind(context.start.to_bytes())
                .bind(context.end.to_bytes())
                .fetch_one(dbtx.as_mut()).await?;
                sqlx::query(
                    "
            INSERT INTO dex_lp_update (height, position_id, state, reserves1, reserves2, execution_id)
            VALUES ($1, $2, $3, CAST($4 AS Amount), CAST($5 AS Amount), $6)
            ",
                )
                .bind(i64::try_from(*height)?)
                .bind(position_id.0)
                .bind(state.to_string())
                .bind(&reserves1)
                .bind(&reserves2)
                .bind(id)
                .execute(dbtx.as_mut())
                .await?;
                sqlx::query(
                    "
                UPDATE dex_lp
                SET reserves1 = CAST($2 AS Amount), reserves2 = CAST($3 AS Amount)
                WHERE id = $1;
                ",
                )
                .bind(position_id.0)
                .bind(&reserves1)
                .bind(&reserves2)
                .execute(dbtx.as_mut())
                .await?;
                Ok(())
            }
            Event::Swap { height, execution } => {
                let mut trace_start = None;
                let mut trace_end = None;
                for trace in &execution.traces {
                    let mut step_start = None;
                    let mut step_end = None;
                    for step in trace {
                        let (id,): (i32,) = sqlx::query_as(
                            r#"INSERT INTO trace_step VALUES (DEFAULT, (CAST($1 AS Amount), $2)) RETURNING id;"#,
                        )
                            .bind(step.amount.to_string())
                            .bind(Sql::from(step.asset_id))
                            .fetch_one(dbtx.as_mut())
                            .await?;
                        if let None = step_start {
                            step_start = Some(id);
                        }
                        step_end = Some(id);
                    }
                    let (id,): (i32,) = sqlx::query_as(
                        r#"INSERT INTO trace VALUES (DEFAULT, $1, $2) RETURNING id;"#,
                    )
                    .bind(step_start)
                    .bind(step_end)
                    .fetch_one(dbtx.as_mut())
                    .await?;
                    if let None = trace_start {
                        trace_start = Some(id);
                    }
                    trace_end = Some(id);
                }
                sqlx::query(r#"INSERT INTO swap VALUES ($1, (CAST($2 AS Amount), $3), (CAST($4 AS AMOUNT), $5), $6, $7);"#)
                    .bind(i64::try_from(*height)?)
                    .bind(execution.input.amount.to_string())
                    .bind(Sql::from(execution.input.asset_id))
                    .bind(execution.output.amount.to_string())
                    .bind(Sql::from(execution.output.asset_id))
                    .bind(trace_start)
                    .bind(trace_end)
                    .execute(dbtx.as_mut())
                    .await?;
                Ok(())
            }
        }
    }
}

impl<'a> TryFrom<&'a ContextualizedEvent> for Event {
    type Error = anyhow::Error;

    fn try_from(event: &'a ContextualizedEvent) -> Result<Self, Self::Error> {
        match event.event.kind.as_str() {
            // Credit
            x if x == Event::NAMES[0] => {
                let pe = pb::EventValueCircuitBreakerCredit::from_event(event.as_ref())?;
                let asset_id =
                    AssetId::try_from(pe.asset_id.ok_or(anyhow!("event missing asset_id"))?)?;
                let previous_balance = Amount::try_from(
                    pe.previous_balance
                        .ok_or(anyhow!("event missing previous_balance"))?,
                )?;
                let new_balance =
                    Amount::try_from(pe.new_balance.ok_or(anyhow!("event missing new_balance"))?)?;
                Ok(Self::CircuitBreakerCredit {
                    asset_id,
                    previous_balance,
                    new_balance,
                })
            }
            // Debit
            x if x == Event::NAMES[1] => {
                let pe = pb::EventValueCircuitBreakerDebit::from_event(event.as_ref())?;
                let asset_id =
                    AssetId::try_from(pe.asset_id.ok_or(anyhow!("event missing asset_id"))?)?;
                let previous_balance = Amount::try_from(
                    pe.previous_balance
                        .ok_or(anyhow!("event missing previous_balance"))?,
                )?;
                let new_balance =
                    Amount::try_from(pe.new_balance.ok_or(anyhow!("event missing new_balance"))?)?;
                Ok(Self::CircuitBreakerDebit {
                    asset_id,
                    previous_balance,
                    new_balance,
                })
            }
            // Arb
            x if x == Event::NAMES[2] => {
                let pe = pb::EventArbExecution::from_event(event.as_ref())?;
                let height = pe.height;
                let execution = pe
                    .swap_execution
                    .ok_or(anyhow!("missing swap execution"))?
                    .try_into()?;
                Ok(Self::ArbExecution { height, execution })
            }
            // LP Withdraw
            x if x == Event::NAMES[3] => {
                let pe = pb::EventPositionWithdraw::from_event(event.as_ref())?;
                let height = event.block_height;
                let position_id = pe
                    .position_id
                    .ok_or(anyhow!("missing position id"))?
                    .try_into()?;
                let reserves_1 = pe
                    .reserves_1
                    .ok_or(anyhow!("missing reserves_1"))?
                    .try_into()?;
                let reserves_2 = pe
                    .reserves_2
                    .ok_or(anyhow!("missing reserves_2"))?
                    .try_into()?;
                let sequence = pe.sequence;
                Ok(Self::PositionWithdraw {
                    height,
                    position_id,
                    reserves_1,
                    reserves_2,
                    sequence,
                })
            }
            // LP Open
            x if x == Event::NAMES[4] => {
                let pe = pb::EventPositionOpen::from_event(event.as_ref())?;
                let height = event.block_height;
                let position = pe
                    .position
                    .ok_or(anyhow!("missing position"))
                    .context("(make sure you're using pd >= 0.79.3)")?
                    .try_into()?;
                Ok(Self::PositionOpen { height, position })
            }
            // LP Close
            x if x == Event::NAMES[5] => {
                let pe = pb::EventPositionClose::from_event(event.as_ref())?;
                let height = event.block_height;
                let position_id = pe
                    .position_id
                    .ok_or(anyhow!("missing position id"))?
                    .try_into()?;

                Ok(Self::PositionClose {
                    height,
                    position_id,
                })
            }
            // LP Execution
            x if x == Event::NAMES[6] => {
                let pe = pb::EventPositionExecution::from_event(event.as_ref())?;
                let height = event.block_height;
                let position_id = pe
                    .position_id
                    .ok_or(anyhow!("missing position id"))?
                    .try_into()?;
                let reserves_1 = pe
                    .reserves_1
                    .ok_or(anyhow!("missing reserves_1"))?
                    .try_into()?;
                let reserves_2 = pe
                    .reserves_2
                    .ok_or(anyhow!("missing reserves_2"))?
                    .try_into()?;
                let prev_reserves_1 = pe
                    .prev_reserves_1
                    .ok_or(anyhow!("missing reserves_1"))?
                    .try_into()?;
                let prev_reserves_2 = pe
                    .prev_reserves_2
                    .ok_or(anyhow!("missing reserves_2"))?
                    .try_into()?;
                let context: DirectedTradingPair =
                    pe.context.ok_or(anyhow!("missing context"))?.try_into()?;
                Ok(Self::PositionExecution {
                    height,
                    position_id,
                    reserves_1,
                    reserves_2,
                    prev_reserves_1,
                    prev_reserves_2,
                    context,
                })
            }
            // Batch Swap
            x if x == Event::NAMES[3] => {
                let pe = pb::EventBatchSwap::from_event(event.as_ref())?;
                let height = event.block_height;
                let execution = pe
                    .swap_execution_1_for_2
                    .ok_or(anyhow!("missing swap execution"))?
                    .try_into()?;
                Ok(Self::Swap { height, execution })
            }
            x => Err(anyhow!(format!("unrecognized event kind: {x}"))),
        }
    }
}

#[derive(Debug)]
pub struct Component {
    event_strings: HashSet<&'static str>,
}

impl Component {
    pub fn new() -> Self {
        let event_strings = Event::NAMES.into_iter().collect();
        Self { event_strings }
    }
}

#[async_trait]
impl AppView for Component {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _app_state: &serde_json::Value,
    ) -> anyhow::Result<()> {
        for statement in include_str!("dex.sql").split(";") {
            sqlx::query(statement).execute(dbtx.as_mut()).await?;
        }
        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        self.event_strings.contains(type_str)
    }

    #[tracing::instrument(skip_all, fields(height = event.block_height, name = event.event.kind.as_str()))]
    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
        _src_db: &PgPool,
    ) -> anyhow::Result<()> {
        Event::try_from(event)?.index(dbtx).await
    }
}
