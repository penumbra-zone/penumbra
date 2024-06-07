use async_trait::async_trait;
use sqlx::{Postgres, Transaction};

use crate::ContextualizedEvent;

pub type PgTransaction<'a> = Transaction<'a, Postgres>;

/// Represents a specific index of raw event data.
#[async_trait]
pub trait Index: std::fmt::Debug {
    async fn create_tables(&self, dbtx: &mut PgTransaction) -> Result<(), anyhow::Error>;

    fn is_relevant(&self, type_str: &str) -> bool;

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
    ) -> Result<(), anyhow::Error>;
}
