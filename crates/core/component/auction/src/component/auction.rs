use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::Component;
use penumbra_proto::StateReadProto;
use penumbra_proto::StateWriteProto;
use std::sync::Arc;
use tendermint::v0_37::abci;
use tracing::instrument;

use crate::{params::AuctionParameters, state_key};

pub struct Auction {}

#[async_trait]
impl Component for Auction {
    // Note: this is currently empty, but will make future
    // addition easy to do.
    type AppState = crate::genesis::Content;

    #[instrument(name = "auction", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            None => { /* perform upgrade specific check */ }
            Some(content) => {
                state.put_auction_params(content.auction_params.clone());
            }
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
pub trait StateReadExt: StateRead {
    async fn get_auction_params(&self) -> Result<AuctionParameters> {
        self.get(state_key::parameters::key())
            .await
            .expect("no deserialization errors")
            .ok_or_else(|| anyhow::anyhow!("Missing AuctionParameters"))
    }

    fn auction_params_updated(&self) -> bool {
        self.object_get::<()>(state_key::parameters::updated_flag())
            .is_some()
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

/// Extension trait providing write access to auction data.
#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Writes the provided auction parameters to the chain state.
    fn put_auction_params(&mut self, params: AuctionParameters) {
        self.object_put(state_key::parameters::updated_flag(), ());
        self.put(state_key::parameters::key().into(), params)
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}

#[cfg(tests)]
mod tests {}
