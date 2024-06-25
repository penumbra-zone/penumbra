/// The Community Pool is a thin component that doesn't have any logic of its own, except for initializing
/// its state and performing post-upgrade checks. It is primarily a collection of state that is modified by
/// [`CommunityPoolSpend`] and [`CommunityPoolDeposit`] actions.
pub mod state_key;

mod action_handler;
mod view;

use std::sync::Arc;

use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::Component;
use tendermint::v0_37::abci;
use tracing::instrument;
pub use view::{StateReadExt, StateWriteExt};

use crate::genesis;

pub struct CommunityPool {}

#[async_trait]
impl Component for CommunityPool {
    type AppState = genesis::Content;

    #[instrument(name = "community_pool", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            Some(genesis) => {
                state.put_community_pool_params(genesis.community_pool_params.clone());
                state.community_pool_deposit(genesis.initial_balance).await;
            }
            None => {}
        }
    }

    #[instrument(name = "community_pool", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    #[instrument(name = "community_pool", skip(_state, _end_block))]
    async fn end_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
    }

    #[instrument(name = "community_pool", skip(_state))]
    async fn end_epoch<S: StateWrite + 'static>(_state: &mut Arc<S>) -> anyhow::Result<()> {
        Ok(())
    }
}
