use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::Component;
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};
use tendermint::v0_37::abci;
use tracing::instrument;

use crate::{epoch::Epoch, genesis, params::SctParameters, state_key};

use super::clock::EpochManager;

pub struct Sct {}

#[async_trait]
impl Component for Sct {
    type AppState = genesis::Content;

    #[instrument(name = "sct_component", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            Some(genesis) => {
                state.put_sct_params(genesis.sct_params.clone());
                state.put_block_height(0);
                state.put_epoch_by_height(
                    0,
                    Epoch {
                        index: 0,
                        start_height: 0,
                    },
                );

                // We need to set the epoch for the first block as well, since we set
                // the epoch by height in end_block, and end_block isn't called after init_chain.
                state.put_epoch_by_height(
                    1,
                    Epoch {
                        index: 0,
                        start_height: 0,
                    },
                );
            }
            None => { /* no-op until an upgrade occurs */ }
        }
    }

    #[instrument(name = "sct_component", skip(state, begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        state: &mut Arc<S>,
        begin_block: &abci::request::BeginBlock,
    ) {
        let state = Arc::get_mut(state).expect("there's only one reference to the state");
        state.put_block_height(begin_block.header.height.into());
        state.put_block_timestamp(begin_block.header.height.into(), begin_block.header.time);
    }

    #[instrument(name = "sct_component", skip(_state, _end_block))]
    async fn end_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
    }

    #[instrument(name = "sct_component", skip(_state))]
    async fn end_epoch<S: StateWrite + 'static>(_state: &mut Arc<S>) -> anyhow::Result<()> {
        Ok(())
    }
}

/// This trait provides read access to configuration data for the component.
#[async_trait]
pub trait StateReadExt: StateRead {
    /// Gets the SCT parameters from the state.
    async fn get_sct_params(&self) -> Result<SctParameters> {
        self.get(state_key::config::sct_params())
            .await?
            .ok_or_else(|| anyhow!("Missing SctParameters"))
    }

    /// Fetch the epoch duration parameter (measured in blocks).
    ///
    /// # Errors
    /// Returns an error if the Sct parameters are missing.
    async fn get_epoch_duration_parameter(&self) -> Result<u64> {
        self.get_sct_params()
            .await
            .map(|params| params.epoch_duration)
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

/// This trait provides write access to configuration data for the component.
#[async_trait]
pub trait StateWriteExt: StateWrite {
    fn put_sct_params(&mut self, params: SctParameters) {
        self.put(state_key::config::sct_params().to_string(), params);
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
