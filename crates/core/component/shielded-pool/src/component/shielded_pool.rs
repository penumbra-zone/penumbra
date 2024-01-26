use std::sync::Arc;

use crate::genesis;
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::Component;
use penumbra_sct::CommitmentSource;
use tendermint::v0_37::abci;
use tracing::instrument;

use super::{NoteManager, SupplyWrite};

pub struct ShieldedPool {}

#[async_trait]
impl Component for ShieldedPool {
    type AppState = genesis::Content;

    #[instrument(name = "shielded_pool", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            None => { /* Checkpoint -- no-op */ }
            Some(genesis) => {
                // Register a denom for each asset in the genesis state
                for allocation in &genesis.allocations {
                    tracing::debug!(?allocation, "processing allocation");
                    assert_ne!(
                        allocation.raw_amount,
                        0u128.into(),
                        "Genesis allocations contain empty note",
                    );

                    state
                        .register_denom(&allocation.denom())
                        .await
                        .expect("able to register denom for genesis allocation");
                    state
                        .mint_note(
                            allocation.value(),
                            &allocation.address,
                            CommitmentSource::Genesis,
                        )
                        .await
                        .expect("able to mint note for genesis allocation");
                }
            }
        }
    }

    #[instrument(name = "shielded_pool", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    #[instrument(name = "shielded_pool", skip(_state, _end_block))]
    async fn end_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
    }

    async fn end_epoch<S: StateWrite + 'static>(mut _state: &mut Arc<S>) -> Result<()> {
        Ok(())
    }
}
