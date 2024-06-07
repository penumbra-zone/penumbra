use anyhow::Result;
use cometindex::{async_trait, ContextualizedEvent, Index, Indexer, PgTransaction};

// This example is silly because it doesn't do any "compilation" of the raw
// events, so it's only useful as an example of exercising the harness and the
// intended usage: the _downstream_ crate depends on cometindex (generic over
// any event) and has its own app specific logic. But it doesn't have to
// reimplement the binary handling / arg parsing / etc

#[derive(Debug)]
struct FmdCluesExample {}

#[async_trait]
impl Index for FmdCluesExample {
    async fn create_tables(&self, dbtx: &mut PgTransaction) -> Result<(), anyhow::Error> {
        sqlx::query(
            "
CREATE TABLE IF NOT EXISTS fmd_clues_example (
    id SERIAL PRIMARY KEY,
    tx_hash BYTEA NOT NULL,
    fmd_clue VARCHAR NOT NULL
);
",
        )
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        type_str == "penumbra.core.component.shielded_pool.v1.EventBroadcastClue"
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
    ) -> Result<(), anyhow::Error> {
        // this is just an example in the integration tests, so we don't want to do any
        // - queries against existing table state
        // - parsing of the event data into structured data
        // - computations of derived data
        // but these should all be possible
        let clue = event
            .event
            .attributes
            .iter()
            .find(|attr| attr.key == "clue")
            .expect("fmd_clue attribute not found")
            .value
            .clone();
        let tx_hash = event.tx_hash.as_ref().expect("tx_hash not found").to_vec();

        sqlx::query(
            "
            INSERT INTO fmd_clues (tx_hash, fmd_clue)
            VALUES ($1, $2)
            ",
        )
        .bind(&tx_hash)
        .bind(&clue)
        .execute(dbtx.as_mut())
        .await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    Indexer::new()
        .with_default_tracing()
        // add as many indexers as you want
        .with_index(FmdCluesExample {})
        .run()
        .await?;

    Ok(())
}
