use std::collections::HashSet;

use anyhow::anyhow;
use cometindex::async_trait;
use penumbra_asset::asset::Id as AssetId;
use penumbra_dex::SwapExecution;
use penumbra_num::Amount;
use penumbra_proto::core::component::dex::v1::{PositionId, TradingPair};
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
    /// A parsed version of [pb::EventPositionOpen]
    PositionOpen {
        height: u64,
        position_id: PositionId,
        trading_pair: TradingPair,
        reserves_1: Amount,
        reserves_2: Amount,
        trading_fee: u32,
    },
    /// A parsed version of [pb::EventPositionWithdraw]
    PositionWithdraw {
        height: u64,
        position_id: PositionId,
        trading_pair: TradingPair,
        reserves_1: Amount,
        reserves_2: Amount,
        sequence: u32,
    },
    /// A parsed version of [pb::EventPositionClose]
    PositionClose {
        height: u64,
        position_id: PositionId,
    },
}

impl Event {
    const NAMES: [&'static str; 6] = [
        "penumbra.core.component.dex.v1.EventValueCircuitBreakerCredit",
        "penumbra.core.component.dex.v1.EventValueCircuitBreakerDebit",
        "penumbra.core.component.dex.v1.EventArbExecution",
        "penumbra.core.component.dex.v1.EventPositionWithdraw",
        "penumbra.core.component.dex.v1.EventPositionOpen",
        "penumbra.core.component.dex.v1.EventPositionClose",
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
                VALUES ($1, $2);
                "#,
                )
                .bind(Sql::from(*asset_id))
                .bind(Sql::from(amount))
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
                VALUES ($1, -$2);
                "#,
                )
                .bind(Sql::from(*asset_id))
                .bind(Sql::from(amount))
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
                        let (id,): (i64,) = sqlx::query_as(
                            r#"INSERT INTO trace_step VALUES (DEFAULT, ($1, $2)) RETURNING id;"#,
                        )
                        .bind(Sql::from(step.amount))
                        .bind(Sql::from(step.asset_id))
                        .fetch_one(dbtx.as_mut())
                        .await?;
                        if let None = step_start {
                            step_start = Some(id);
                        }
                        step_end = Some(id);
                    }
                    let (id,): (i64,) = sqlx::query_as(
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
                sqlx::query(r#"INSERT INTO arb VALUES ($1, ($2, $3), ($4, $5), $6, $7);"#)
                    .bind(i64::try_from(*height)?)
                    .bind(Sql::from(execution.input.amount))
                    .bind(Sql::from(execution.input.asset_id))
                    .bind(Sql::from(execution.output.amount))
                    .bind(Sql::from(execution.output.asset_id))
                    .bind(trace_start)
                    .bind(trace_end)
                    .execute(dbtx.as_mut())
                    .await?;
                Ok(())
            }
            Event::PositionOpen {
                height,
                position_id,
                ..
            } => {
                sqlx::query(
                    "
            INSERT INTO lp_updates (height, type, position_id)
            VALUES ($1, $2, $3)
            ",
                )
                .bind(*height as i64)
                .bind(0)
                .bind(&position_id.inner)
                .execute(dbtx.as_mut())
                .await?;
                Ok(())
            }
            Event::PositionWithdraw {
                height,
                position_id,
                ..
            } => {
                sqlx::query(
                    "
            INSERT INTO lp_updates (height, type, position_id)
            VALUES ($1, $2, $3)
            ",
                )
                .bind(*height as i64)
                .bind(2)
                .bind(&position_id.inner)
                .execute(dbtx.as_mut())
                .await?;
                Ok(())
            }
            Event::PositionClose {
                height,
                position_id,
                ..
            } => {
                sqlx::query(
                    "
            INSERT INTO lp_updates (height, type, position_id)
            VALUES ($1, $2, $3)
            ",
                )
                .bind(*height as i64)
                .bind(1)
                .bind(&position_id.inner)
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
                let trading_pair = pe
                    .trading_pair
                    .ok_or(anyhow!("missing trading pair"))?
                    .try_into()?;
                let reserves_1 = pe
                    .reserves_1
                    .ok_or(anyhow!("missing reserves_1"))?
                    .try_into()?;
                let reserves_2 = pe
                    .reserves_2
                    .ok_or(anyhow!("missing reserves_2"))?
                    .try_into()?;
                let sequence = pe.sequence.try_into()?;
                Ok(Self::PositionWithdraw {
                    height,
                    position_id,
                    trading_pair,
                    reserves_1,
                    reserves_2,
                    sequence,
                })
            }
            // LP Open
            x if x == Event::NAMES[4] => {
                let pe = pb::EventPositionOpen::from_event(event.as_ref())?;
                let height = event.block_height;
                let position_id = pe
                    .position_id
                    .ok_or(anyhow!("missing position id"))?
                    .try_into()?;
                let trading_pair = pe
                    .trading_pair
                    .ok_or(anyhow!("missing trading pair"))?
                    .try_into()?;
                let reserves_1 = pe
                    .reserves_1
                    .ok_or(anyhow!("missing reserves_1"))?
                    .try_into()?;
                let reserves_2 = pe
                    .reserves_2
                    .ok_or(anyhow!("missing reserves_2"))?
                    .try_into()?;
                let trading_fee = pe.trading_fee.try_into()?;
                Ok(Self::PositionOpen {
                    height,
                    position_id,
                    trading_pair,
                    reserves_1,
                    reserves_2,
                    trading_fee,
                })
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
        sqlx::query(include_str!("dex.sql"))
            .execute(dbtx.as_mut())
            .await?;
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
