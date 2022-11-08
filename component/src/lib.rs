#![recursion_limit = "256"] // required for TCT

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_storage2::State;
use penumbra_storage2::StateTransaction;
use penumbra_transaction::Transaction;
use std::sync::Arc;
use tendermint::abci;

pub mod app;
pub mod dex;
pub mod governance;
pub mod ibc;
pub mod shielded_pool;
pub mod stake;

// Scratch -- future direction ?
//
// impl ActionHandler for Transaction { /* does tx-wide checks, then delegates into per-action impls */ }
// impl ActionHandler for Action { /* per-action implementation */ }
/*
#[async_trait]
pub trait ActionHandler {
    /// Performs all of this component's stateless validity checks on the given
    /// [`Transaction`].
    fn check_tx_stateless(&self, ctx: Context) -> Result<()>;

    /// Performs all of this component's stateful validity checks on the given
    /// [`Transaction`].
    ///
    /// # Invariants
    ///
    /// This method should only be called on transactions that have been
    /// checked with [`Component::check_tx_stateless`].
    /// This method can be called before [`Component::begin_block`].
    async fn check_tx_stateful(&self, state: Arc<State>, ctx: Context)
        -> Result<()>;

    /// Executes the given [`Transaction`] against the current state.
    ///
    /// # Invariants
    ///
    /// This method should only be called immediately following a successful
    /// invocation of [`Component::check_tx_stateful`] on the same transaction.
    /// This method can be called before [`Component::begin_block`].
    async fn execute_tx(
        &self,
        state: &mut StateTransaction,

    ) -> Result<()>;
}
*/

/// A component of the Penumbra application.
#[async_trait]
pub trait Component {
    /// Performs initialization, given the genesis state.
    ///
    /// This method is called once per chain, and should only perform
    /// writes, since the backing tree for the [`State`] will
    /// be empty.
    async fn init_chain(state: &mut StateTransaction, app_state: &genesis::AppState);

    /// Begins a new block, optionally inspecting the ABCI
    /// [`BeginBlock`](abci::request::BeginBlock) request.
    async fn begin_block(state: &mut StateTransaction, begin_block: &abci::request::BeginBlock);

    /// Performs all of this component's stateless validity checks on the given
    /// [`Transaction`].
    fn check_tx_stateless(tx: Arc<Transaction>) -> Result<()>;

    /// Performs all of this component's stateful validity checks on the given
    /// [`Transaction`].
    ///
    /// # Invariants
    ///
    /// This method should only be called on transactions that have been
    /// checked with [`Component::check_tx_stateless`].
    /// This method can be called before [`Component::begin_block`].
    async fn check_tx_stateful(state: Arc<State>, tx: Arc<Transaction>) -> Result<()>;

    /// Executes the given [`Transaction`] against the current state.
    ///
    /// # Invariants
    ///
    /// This method should only be called immediately following a successful
    /// invocation of [`Component::check_tx_stateful`] on the same transaction.
    /// This method can be called before [`Component::begin_block`].
    async fn execute_tx(state: &mut StateTransaction, tx: Arc<Transaction>) -> Result<()>;

    /// Ends the block, optionally inspecting the ABCI
    /// [`EndBlock`](abci::request::EndBlock) request, and performing any batch
    /// processing.
    ///
    /// # Invariants
    ///
    /// This method should only be called after [`Component::begin_block`].
    /// No methods should be called following this method.
    async fn end_block(state: &mut StateTransaction, end_block: &abci::request::EndBlock);
}
