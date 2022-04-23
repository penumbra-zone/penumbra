use anyhow::Result;
use async_trait::async_trait;
use penumbra_transaction::Transaction;
use tendermint::abci;

use crate::genesis;
use crate::Overlay;

/// A component of the Penumbra application.
///
/// Each component is a thin wrapper around a shared [`Overlay`], over a
/// Jellyfish tree held in persistent [`Storage`].  The Jellyfish tree is a
/// generic, byte-oriented key/value store.  Components can read from and write
/// to the tree, and all components in the same [`Application`] instance will
/// see each others' writes when they perform reads.  However, those writes are
/// buffered in the [`Overlay`] until it commits a batch of changes to the
/// persistent [`Storage`], making it possible to maintain and evolve multiple
/// copies of the application state, as each [`Application`] is effectively its
/// own copy-on-write instance of the chain state.
///
/// The data and execution flow looks like:
/// ```ascii,no_run
/// ┌────────────┐          ┌───────────┐                           
/// │WriteOverlay│          │ Component │                           
/// │  ::new()   │═════════▶│  ::new()  │      ═══▶ Execution Flow  
/// └────────────┘          └───────────┘           (Approximate)   
///        │                      ║            ───▶ Data Flow       
///        ▼               ╔══════╩══════╗                          
/// ┌────────────┐         ▼             ║                          
/// │            │   ┌───────────┐       ║         ┌───────────────┐
/// │            │◀──│init_chain │◀──────╬─────────│ Genesis State │
/// │            │   └───────────┘       ║         └───────────────┘
/// │            │         ║             ▼                          
/// │            │         ║       ┌───────────┐   ┌───────────────┐
/// │            │◀────────╬──────▶│begin_block│◀──│ABCI BeginBlock│
/// │            │         ║       └───────────┘   └───────────────┘
/// │            │         ║             ║                          
/// │            │         ║    ╔═══════▶║                          
/// │            │         ║    ║        ▼                          
/// │            │         ║    ║  ┌───────────┐   ┌───────────────┐
/// │            │         ║    ║  │check_tx   │ ┌─│  Transaction  │
/// │            │         ║    ║  │_stateless │◀┤ └───────────────┘
/// │WriteOverlay│         ║    ║  └───────────┘ │                  
/// │            │         ║    ║  ┌───────────┐ │                  
/// │            │         ║    ║  │check_tx   │ │                  
/// │            │─────────╬────╬─▶│_stateful  │◀┤                  
/// │            │         ║    ║  └───────────┘ │                  
/// │            │         ║    ║  ┌───────────┐ │                  
/// │            │◀────────╬────╬─▶│execute_tx │◀┘                  
/// │            │         ║    ║  └───────────┘                    
/// │            │         ║    ║        ║                          
/// │            │         ║    ╚════════╣                          
/// │            │         ║             ║                          
/// │            │         ║             ▼                          
/// │            │         ║       ┌───────────┐   ┌───────────────┐
/// │            │◀────────╬──────▶│end_block  │◀──│ ABCI EndBlock │
/// └────────────┘         ║       └───────────┘   └───────────────┘
///        │               ║             ║                          
///        ▼               ║             ║                          
/// ┌────────────┐         ║             ║                          
/// │WriteOverlay│         ║             ║                          
/// │ ::commit() │◀════════╩═════════════╝                          
/// └────────────┘                                                  
/// ```
#[async_trait]
pub trait Component: Sized {
    /// Initializes the component relative to a shared state.
    ///
    /// This method should be called every time the [`Overlay`] is
    /// re-initialized.
    async fn new(overlay: Overlay) -> Self;

    /// Performs initialization, given the genesis state.
    ///
    /// This method is called once per chain, and should only perform
    /// writes, since the backing tree for the [`WriteOverlay`] will
    /// be empty.
    ///
    /// # Invariants
    ///
    /// This method should only be called immediately after [`Component::new`].
    /// No methods should be called following this method.
    async fn init_chain(&mut self, app_state: &genesis::AppState);

    /// Begins a new block, optionally inspecting the ABCI
    /// [`BeginBlock`](abci::request::BeginBlock) request.
    ///
    /// # Invariants
    ///
    /// This method should only be called immediately after [`Component::new`].
    /// This method need not be called before [`Component::execute_tx`] (e.g.,
    /// in order to simulate executing a transaction in the mempool).
    async fn begin_block(&mut self, begin_block: &abci::request::BeginBlock);

    /// Performs all of this component's stateless validity checks on the given
    /// [`Transaction`].
    fn check_tx_stateless(tx: &Transaction) -> Result<()>;

    /// Performs all of this component's stateful validity checks on the given
    /// [`Transaction`].
    ///
    /// # Invariants
    ///
    /// This method should only be called on transactions that have been
    /// checked with [`Component::check_tx_stateless`].
    /// This method can be called before [`Component::begin_block`].
    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()>;

    /// Executes the given [`Transaction`] against the current state.
    ///
    /// # Invariants
    ///
    /// This method should only be called immediately following a successful
    /// invocation of [`Component::check_tx_stateful`] on the same transaction.
    /// This method can be called before [`Component::begin_block`].
    async fn execute_tx(&mut self, tx: &Transaction);

    /// Ends the block, optionally inspecting the ABCI
    /// [`EndBlock`](abci::request::EndBlock) request, and performing any batch
    /// processing.
    ///
    /// # Invariants
    ///
    /// This method should only be called after [`Component::begin_block`].
    /// No methods should be called following this method.
    async fn end_block(&mut self, end_block: &abci::request::EndBlock);
}
