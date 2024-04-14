use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::Component;
use std::sync::Arc;
use tendermint::v0_37::abci;
use tracing::instrument;

pub struct Auction {}
impl Auction {}

#[async_trait]
impl Component for Auction {
    type AppState = ();

    #[instrument(name = "auction", skip(_state, app_state))]
    async fn init_chain<S: StateWrite>(_state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            None => { /* perform upgrade specific check */ }
            Some(&()) => {}
        }
    }

    #[instrument(name = "auction", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    #[instrument(name = "auction", skip(_state, _end_block))]
    async fn end_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
    }

    #[instrument(name = "auction", skip(_state))]
    async fn end_epoch<S: StateWrite + 'static>(_state: &mut Arc<S>) -> Result<()> {
        Ok(())
    }
}

/// Extension trait providing read access to auction data.
#[async_trait]
pub trait StateReadExt: StateRead {}

impl<T: StateRead + ?Sized> StateReadExt for T {}

/// Extension trait providing write access to auction data.
#[async_trait]
pub trait StateWriteExt: StateWrite {}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}

#[cfg(tests)]
mod tests {}
