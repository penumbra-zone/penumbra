//! Token factory component for state management and action handling.
//!
//! This component tracks used nonces to prevent token ID collisions.

pub mod event;
pub mod state_key;

mod action_handler;
mod view;

pub use view::{StateReadExt, StateWriteExt};

use std::sync::Arc;

use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::Component;
use tendermint::v0_37::abci;
use tracing::instrument;

/// The token factory component.
pub struct TokenFactory {}

#[async_trait]
impl Component for TokenFactory {
    type AppState = ();

    #[instrument(name = "token_factory", skip(_state, _app_state))]
    async fn init_chain<S: StateWrite>(_state: S, _app_state: Option<&Self::AppState>) {
        // Token factory has no genesis state to initialize.
    }

    #[instrument(name = "token_factory", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    #[instrument(name = "token_factory", skip(_state, _end_block))]
    async fn end_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
    }

    #[instrument(name = "token_factory", skip(_state))]
    async fn end_epoch<S: StateWrite + 'static>(_state: &mut Arc<S>) -> anyhow::Result<()> {
        Ok(())
    }
}
