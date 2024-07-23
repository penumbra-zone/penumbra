use async_trait::async_trait;
pub use sqlx::PgPool;
use sqlx::{Postgres, Transaction};

use crate::ContextualizedEvent;

pub type PgTransaction<'a> = Transaction<'a, Postgres>;

/// Represents a specific index of raw event data.
#[async_trait]
pub trait AppView: std::fmt::Debug {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error>;

    fn is_relevant(&self, type_str: &str) -> bool;

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
        src_db: &PgPool,
    ) -> Result<(), anyhow::Error>;
}
