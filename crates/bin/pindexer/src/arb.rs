use anyhow::Result;
use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgTransaction};
use penumbra_num::Amount;
use penumbra_proto::core::asset::v1::AssetId;
use penumbra_proto::{core::component::dex::v1 as pb, event::ProtoEvent};
use sqlx::{Postgres, QueryBuilder};

#[derive(Debug)]
pub struct Arb {}

#[async_trait]
impl AppView for Arb {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(
            "
CREATE TABLE IF NOT EXISTS arbs (
    id SERIAL PRIMARY KEY,
    height INT8 NOT NULL,
    input_amount BIGINT NOT NULL,
    input_asset_id BYTEA NOT NULL,
    output_amount BIGINT NOT NULL,
    output_asset_id BYTEA NOT NULL,
    trace_start INT8,
    trace_end INT8
);
",
        )
        .execute(dbtx.as_mut())
        .await?;

        sqlx::query(
            "
CREATE TABLE IF NOT EXISTS arb_traces (
    id SERIAL PRIMARY KEY,
    steps_start INT8 NOT NULL,
    steps_end INT8 NOT NULL,
);
",
        )
        .execute(dbtx.as_mut())
        .await?;

        sqlx::query(
            "
CREATE TABLE IF NOT EXISTS trace_steps (
    id SERIAL PRIMARY KEY,
    amount BIGINT NOT NULL,
    asset_id BYTEA NOT NULL,
);
",
        )
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        type_str == "penumbra.core.component.dex.v1.EventArbExecution"
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
    ) -> Result<(), anyhow::Error> {
        let pe = pb::EventArbExecution::from_event(event.as_ref())?;
        let swap = pe
            .swap_execution
            .expect("Swap execution missing in arb event");

        let mut trace_ids: Vec<i64> = vec![];

        // Iterate over each trace, insert the steps for that trace, get and save the indexes into the
        // parent trace
        for trace in swap.traces {
            /* Insert the steps */
            let mut query_builder: QueryBuilder<Postgres> =
                QueryBuilder::new("WITH inserted AS ( INSERT INTO trace_steps(amount, asset_id) ");

            query_builder.push_values(trace.value, |mut b, step| {
                let asset_id =
                    AssetId::try_from(step.asset_id.expect("missing step asset id in event"))
                        .unwrap();

                let amount =
                    Amount::try_from(step.amount.expect("missing step amount in event")).unwrap();

                b.push_bind(amount.value() as i64).push_bind(asset_id.inner);
            });

            query_builder.push(
                " RETURNING id
        )
        SELECT MIN(id) AS first_id, MAX(id) AS last_id FROM inserted",
            );

            /*Insert, and get the start and end indexes*/
            let (start, end) = query_builder
                .build_query_as::<(i32, i32)>()
                .fetch_one(dbtx)
                .await?;

            /* Insert the trace, referencing the start and end indexes */
            let trace_id = sqlx::query(
                "
            INSERT INTO arb_traces (steps_start, steps_end)
            VALUES ($1, $2) 
RETURNING id
            ",
            )
            .bind(start)
            .bind(end)
            .build_query_as::<i32>()
            .fetch_one(dbtx.as_mut())
            .await?;

            trace_ids.push(trace_id as i64);
        }

        let input = swap.input.expect("Input is None");
        let input_amount = Amount::try_from(input.amount.expect("missing amount in event"))?;
        let input_asset_id =
            AssetId::try_from(input.asset_id.expect("missing input asset id in event"))?;

        let output = swap.output.expect("Input is None");
        let output_amount = Amount::try_from(output.amount.expect("missing amount in event"))?;
        let output_asset_id =
            AssetId::try_from(output.asset_id.expect("missing output asset id in event"))?;

        sqlx::query(
            "
            INSERT INTO arbs (height, input_amount, input_asset_id, output_amount, output_asset_id, trace_start, trace_end)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ",
        )
        .bind(event.block_height as i64)
        .bind(input_amount.value() as i64)
        .bind(input_asset_id.inner)
        .bind(output_amount.value() as i64)
        .bind(output_asset_id.inner)
        .bind(trace_ids.first())
        .bind(trace_ids.last())
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
    }
}
