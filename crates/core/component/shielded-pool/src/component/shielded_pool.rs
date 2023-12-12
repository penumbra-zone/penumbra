use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateRead;
use cnidarium::StateWrite;
use cnidarium_component::Component;
use penumbra_chain::{NoteSource, SpendInfo};
use penumbra_proto::StateReadProto;
use penumbra_sct::Nullifier;
use tendermint::v0_37::abci;
use tracing::instrument;

use crate::genesis::Content as GenesisContent;
use crate::state_key;

use super::{NoteManager, SupplyWrite};

pub struct ShieldedPool {}

#[async_trait]
impl Component for ShieldedPool {
    type AppState = GenesisContent;

    #[instrument(name = "shielded_pool", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&GenesisContent>) {
        match app_state {
            None => { /* Checkpoint -- no-op */ }
            Some(app_state) => {
                // Register a denom for each asset in the genesis state
                for allocation in &app_state.allocations {
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
                        .mint_note(allocation.value(), &allocation.address, NoteSource::Genesis)
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

#[async_trait]
pub trait StateReadExt: StateRead {
    async fn check_nullifier_unspent(&self, nullifier: Nullifier) -> Result<()> {
        if let Some(info) = self
            .get::<SpendInfo>(&state_key::spent_nullifier_lookup(&nullifier))
            .await?
        {
            anyhow::bail!(
                "nullifier {} was already spent in {:?}",
                nullifier,
                info.note_source,
            );
        }
        Ok(())
    }
    async fn spend_info(&self, nullifier: Nullifier) -> Result<Option<SpendInfo>> {
        self.get(&state_key::spent_nullifier_lookup(&nullifier))
            .await
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}
