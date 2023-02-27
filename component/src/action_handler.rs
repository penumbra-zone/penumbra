use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::Transaction;

mod actions;
mod transaction;

#[async_trait]
/// Defines the interface for handling transaction actions.
///
/// Block-wide execution is performed using the [`Component`](crate::Component)
/// trait.  Per-transaction execution is performed using the `ActionHandler`
/// trait.
///
/// The `ActionHandler` trait has a top-level implementation on [`Transaction`],
/// which performs any transaction-wide checks and then calls the
/// `ActionHandler` implementation for each [`Action`](penumbra_transaction::Action).
///
/// The validation logic in the `ActionHandler` trait is split into three phases:
///
/// * [`ActionHandler::check_stateless`], which has no access to state, only to the transaction the action is part of;
/// * [`ActionHandler::check_stateful`], which has read access to a snapshot of state prior to transaction execution;
/// * [`ActionHandler::execute`], which has write access to the state and read access to its own writes.
///
/// All of these methods are asynchronous and fallible; an error at any level
/// fails the transaction and aborts any in-progress execution.
///
/// These methods are described in more detail below, but in general, as much
/// work as possible should be pushed up the stack, where greater parallelism is
/// available, with checks performed in `execute` only as a last resort.
pub trait ActionHandler {
    /// Performs all of this action's stateless validity checks in the
    /// `context` of some [`Transaction`].
    ///
    /// This method is `async` to make it easy to perform stateless validity
    /// checks in parallel, by allowing `ActionHandler` implementations to
    /// easily spawn tasks internally.
    ///
    /// Supplying the `context` means that stateless checks can use
    /// transaction-wide data like the SCT anchor.  Supplying it as an
    /// `Arc<Transaction>` makes it easy to share the context with internally
    /// spawned tasks.
    ///
    /// As much work as possible should be done in `check_stateless`, as it can
    /// be run in parallel across all transactions in a block.
    async fn check_stateless(&self, context: Arc<Transaction>) -> Result<()>;

    /// Performs all of this action's stateful validity checks.
    ///
    /// This method provides read access to a snapshot of the `State` prior to
    /// transaction execution.  Because it does not give write access, it's safe
    /// to run the `ActionHandler` implementations for all actions in a given
    /// transaction in parallel.
    ///
    /// As much work as possible should be done in `check_stateful`, as it can
    /// be run in parallel across all actions within a transaction.
    ///
    /// # Invariants
    ///
    /// This method should only be called on data that has been checked
    /// with [`ActionHandler::check_stateless`].  This method can be called
    /// before [`Component::begin_block`](crate::Component::begin_block).
    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()>;

    /// Attempts to execute this action against the provided `state`.
    ///
    /// This method provides read and write access to the `state`. It is
    /// fallible, so it's possible to perform checks within the `execute`
    /// implementation and abort execution on error; the [`StateTransaction`]
    /// mechanism ensures that all writes are correctly discarded.
    ///
    /// Because `execute` must run sequentially, whenever possible, checks
    /// should be performed in [`ActionHandler::check_stateful`] or
    /// [`ActionHandler::check_stateless`].  One example of where this is not
    /// possible (in fact, the motivating example) is for IBC, where a
    /// transaction may (1) submit a client update and then (2) relay messages
    /// valid relative to the newly updated state.  In this case, the checks for
    /// (2) must be performed during execution, as they depend on the state
    /// changes written while processing the client update.
    ///
    /// However, this data flow pattern should be avoided whenever possible.
    ///
    /// # Invariants
    ///
    /// This method should only be called immediately after an invocation of
    /// [`ActionHandler::check_stateful`] on the same transaction.  This method
    /// can be called before [`Component::begin_block`](crate::Component::begin_block).
    async fn execute<S: StateWrite>(&self, state: S) -> Result<()>;
}
