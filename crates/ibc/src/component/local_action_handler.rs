use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};

/// Vendored copy of ActionHandler to work around orphan rules -- TODO: reshape into ibc-specific trait for ibc-async with just check_stateless and execute?
#[async_trait]
pub(crate) trait ActionHandler {
    type CheckStatelessContext: Clone + Send + Sync + 'static;
    async fn check_stateless(&self, context: Self::CheckStatelessContext) -> Result<()>;
    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()>;
    async fn execute<S: StateWrite>(&self, state: S) -> Result<()>;
}
