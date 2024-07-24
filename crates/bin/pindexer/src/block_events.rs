use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgTransaction};
use penumbra_proto::{core::component::{sct::v1 as sct_pb, auction::v1 as auction_pb, dex::v1 as dex_pb, stake::v1 as stake_pb}, event::ProtoEvent};
use sqlx::types::chrono::DateTime;
use anyhow::Result;

#[derive(Debug)]
pub struct BlockEvents {}

#[async_trait]
impl AppView for BlockEvents {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS block_events (
                id SERIAL PRIMARY KEY,
                height INT8 NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                events JSONB NOT NULL
            );",
        )
        .execute(dbtx.as_mut())
        .await?;

        sqlx::query(
            "CREATE INDEX idx_height ON block_events(height DESC);",
        )
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        // Current known set of ABCI events emitted that can be block events
        match type_str {
            "block" => true,
            "penumbra.core.component.auction.v1.EventDutchAuctionEnded" => true,
            "penumbra.core.component.auction.v1.EventDutchAuctionUpdated" => true,
            "penumbra.core.component.auction.v1.EventValueCircuitBreakerCredit" => true,
            "penumbra.core.component.auction.v1.EventValueCircuitBreakerDebit" => true,
            "penumbra.core.component.dex.v1.EventArbExecution" => true,
            "penumbra.core.component.dex.v1.EventBatchSwap" => true,
            "penumbra.core.component.dex.v1.EventPositionClose" => true,
            "penumbra.core.component.dex.v1.EventPositionExecution" => true,
            "penumbra.core.component.dex.v1.EventPositionOpen" => true,
            "penumbra.core.component.dex.v1.EventPositionWithdraw" => true,
            "penumbra.core.component.dex.v1.EventValueCircuitBreakerCredit" => true,
            "penumbra.core.component.dex.v1.EventValueCircuitBreakerDebit" => true,
            "penumbra.core.component.sct.v1.EventAnchor" => true,
            "penumbra.core.component.sct.v1.EventBlockRoot" => true,
            "penumbra.core.component.sct.v1.EventCommitment" => true,
            "penumbra.core.component.sct.v1.EventEpochRoot" => true,
            "penumbra.core.component.stake.v1.EventTombstoneValidator" => true,
            _ => false
        }
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
    ) -> Result<(), anyhow::Error> {
        // Transaction Event, not a Block Event.
        if event.tx_hash.is_some() {
            return Ok(())
        }

        match event.event.kind.as_str() {
            // This event type isn't real as far as I can tell. Not sure what to do with it.
            "block" => {},
            // EventBlockRoot should always be first... Right?
            "penumbra.core.component.sct.v1.EventBlockRoot" => {
                let val = sct_pb::EventBlockRoot::from_event(event.as_ref())?;
                let timestamp = val.timestamp.clone().expect("BlockRoot has no timestamp");
                // Should always be first.
                sqlx::query(
                    "INSERT INTO block_events (height, timestamp, events)
                    VALUES ($1, $2, JSON_ARRAY(JSON_OBJECT($3, $4)))"
                )
                .bind(event.block_height as i64)
                .bind(DateTime::from_timestamp(timestamp.seconds, timestamp.nanos as u32).expect("Could not convert timestamp."))
                .bind(event.event.kind.as_str())
                .bind(serde_json::to_string(&val).expect("Serializable"))
                .execute(dbtx.as_mut())
                .await?;
            },
            "penumbra.core.component.sct.v1.EventAnchor" => handle_block_event::<sct_pb::EventAnchor>(dbtx, event).await?,
            "penumbra.core.component.sct.v1.EventCommitment" => handle_block_event::<sct_pb::EventCommitment>(dbtx, event).await?,
            "penumbra.core.component.sct.v1.EventEpochRoot" => handle_block_event::<sct_pb::EventEpochRoot>(dbtx, event).await?,
            "penumbra.core.component.auction.v1.EventDutchAuctionEnded" => handle_block_event::<auction_pb::EventDutchAuctionEnded>(dbtx, event).await?,
            "penumbra.core.component.auction.v1.EventDutchAuctionUpdated" => handle_block_event::<auction_pb::EventDutchAuctionUpdated>(dbtx, event).await?,
            "penumbra.core.component.auction.v1.EventValueCircuitBreakerCredit" => handle_block_event::<auction_pb::EventValueCircuitBreakerCredit>(dbtx, event).await?,
            "penumbra.core.component.auction.v1.EventValueCircuitBreakerDebit" => handle_block_event::<auction_pb::EventValueCircuitBreakerDebit>(dbtx, event).await?,
            "penumbra.core.component.dex.v1.EventArbExecution" => handle_block_event::<dex_pb::EventArbExecution>(dbtx, event).await?,
            "penumbra.core.component.dex.v1.EventBatchSwap" => handle_block_event::<dex_pb::EventBatchSwap>(dbtx, event).await?,
            "penumbra.core.component.dex.v1.EventPositionClose" => handle_block_event::<dex_pb::EventPositionClose>(dbtx, event).await?,
            "penumbra.core.component.dex.v1.EventPositionExecution" => handle_block_event::<dex_pb::EventPositionExecution>(dbtx, event).await?,
            "penumbra.core.component.dex.v1.EventPositionOpen" => handle_block_event::<dex_pb::EventPositionOpen>(dbtx, event).await?,
            "penumbra.core.component.dex.v1.EventPositionWithdraw" => handle_block_event::<dex_pb::EventPositionWithdraw>(dbtx, event).await?,
            "penumbra.core.component.dex.v1.EventValueCircuitBreakerCredit" => handle_block_event::<dex_pb::EventValueCircuitBreakerCredit>(dbtx, event).await?,
            "penumbra.core.component.dex.v1.EventValueCircuitBreakerDebit" => handle_block_event::<dex_pb::EventValueCircuitBreakerDebit>(dbtx, event).await?,
            "penumbra.core.component.stake.v1.EventTombstoneValidator" => handle_block_event::<stake_pb::EventTombstoneValidator>(dbtx, event).await?,
            _ => {}
        }
        Ok(())
    }
}

async fn handle_block_event<'a, E: ProtoEvent>(
    dbtx: &mut PgTransaction<'a>,
    event: &ContextualizedEvent,
) -> Result<()> {
    let height = event.block_height;
    let pe = E::from_event(event.as_ref())?;
    let affected = sqlx::query(
        "UPDATE block_events
        SET events = JSONB_INSERT(events, '{0}', JSON_OBJECT($2, $3))
        WHERE height=$1"
    )
    .bind(height as i64)
    .bind(event.event.kind.as_str())
    .bind(serde_json::to_string(&pe).expect("Serializable"))
    .execute(dbtx.as_mut())
    .await?
    .rows_affected();

    if affected == 0 {
        anyhow::bail!("No block found for this event!");
    }

    Ok(())
}
