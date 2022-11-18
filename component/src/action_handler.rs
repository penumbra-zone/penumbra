use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::Transaction;

mod actions;
mod transaction;

#[async_trait]
/// Defines the interface for handling actions. This allows us to split the logic between
/// ABCI message-handling logic in the [`Component`](crate::Component)-derived traits, and the logic for handling
/// actions within a transaction in the `ActionHandler`-derived traits.
pub trait ActionHandler {
    /// Performs all of this component's stateless validity checks in the context of
    /// the given [`Transaction`].
    fn check_stateless(&self, context: Arc<Transaction>) -> Result<()>;

    /// Performs all of this component's stateful validity checks in the context of the given
    /// [`Transaction`].
    ///
    /// # Invariants
    ///
    /// This method should only be called on transactions that have been
    /// checked with [`Component::check_tx_stateless`].
    /// This method can be called before [`Component::begin_block`].
    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()>;

    /// Executes the given [`Transaction`] against the current state.
    ///
    /// # Invariants
    ///
    /// This method should only be called immediately following a successful
    /// invocation of [`Component::check_tx_stateful`] on the same transaction.
    /// This method can be called before [`Component::begin_block`].
    async fn execute(&self, state: &mut StateTransaction) -> Result<()>;
}
