use std::sync::Arc;

use async_trait::async_trait;
pub use sqlx::PgPool;
use sqlx::{Postgres, Transaction};

use crate::ContextualizedEvent;

pub type PgTransaction<'a> = Transaction<'a, Postgres>;

/// Represents all of the events in a given block
#[derive(Clone, Debug)]
pub struct BlockEvents {
    /// The height of this block.
    pub height: u64,
    /// The events contained in this block, in order.
    pub events: Vec<ContextualizedEvent>,
}

#[derive(Clone, Debug)]
pub struct EventBatch {
    pub first_height: u64,
    pub last_height: u64,
    /// The batch of events, ordered by increasing height.
    ///
    /// The heights are guaranteed to be increasing, and to be contiguous.
    pub by_height: Arc<Vec<BlockEvents>>,
}

impl EventBatch {
    pub fn events(&self) -> impl Iterator<Item = &'_ ContextualizedEvent> {
        self.by_height.iter().flat_map(|x| x.events.iter())
    }
}

/// Represents a specific index of raw event data.
#[async_trait]
pub trait AppView: Send + Sync {
    /// Return the name of this index.
    ///
    /// This should be unique across all of the indices.
    fn name(&self) -> String;

    /// This will be called once when processing the genesis before the first block.
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error>;

    /// This allows processing a batch of events, over many blocks.
    ///
    /// By using a batch, we can potentially avoid a costly
    async fn index_batch(
        &self,
        dbtx: &mut PgTransaction,
        batch: EventBatch,
    ) -> Result<(), anyhow::Error>;
}
