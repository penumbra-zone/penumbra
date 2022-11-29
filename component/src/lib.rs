use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_storage::StateTransaction;
use tendermint::abci;

mod action_handler;

mod temp_storage_ext;
pub use temp_storage_ext::TempStorageExt;

pub mod app;
pub mod dex;
pub mod governance;
pub mod ibc;
pub mod shielded_pool;
pub mod stake;

pub use action_handler::ActionHandler;

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
