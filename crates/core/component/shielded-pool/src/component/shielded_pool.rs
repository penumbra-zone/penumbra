use std::sync::Arc;

use crate::params::ShieldedPoolParameters;
use crate::{fmd, genesis, state_key};
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::Component;
use penumbra_proto::StateReadProto as _;
use penumbra_proto::StateWriteProto as _;
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
                // TODO(erwan): the handling of those parameters is a bit weird.
                // rationalize it before merging
                state.put_shielded_pool_params(genesis.shielded_pool_params.clone());
                state.put_current_fmd_parameters(fmd::Parameters::default());
                state.put_previous_fmd_parameters(fmd::Parameters::default());

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
/// Extension trait providing read access to shielded pool data.
#[async_trait]
pub trait StateReadExt: StateRead {
    async fn get_current_fmd_parameters(&self) -> Result<fmd::Parameters> {
        self.get(fmd::state_key::parameters::current())
            .await?
            .ok_or_else(|| anyhow!("Missing FmdParameters"))
    }

    /// Gets the previous FMD parameters from the JMT.
    async fn get_previous_fmd_parameters(&self) -> Result<fmd::Parameters> {
        self.get(fmd::state_key::parameters::previous())
            .await?
            .ok_or_else(|| anyhow!("Missing FmdParameters"))
    }

    async fn get_shielded_pool_params(&self) -> Result<ShieldedPoolParameters> {
        self.get(state_key::shielded_pool_params())
            .await?
            .ok_or_else(|| anyhow!("Missing ShieldedPoolParameters"))
    }

    fn shielded_pool_params_updated(&self) -> bool {
        self.object_get::<()>(state_key::shielded_pool_params_updated())
            .is_some()
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

/// Extension trait providing write access to shielded pool data.
#[async_trait]
pub trait StateWriteExt: StateWrite + StateReadExt {
    fn put_shielded_pool_params(&mut self, params: ShieldedPoolParameters) {
        self.object_put(crate::state_key::shielded_pool_params_updated(), ());
        self.put(crate::state_key::shielded_pool_params().into(), params)
    }

    /// Writes the current FMD parameters to the JMT.
    fn put_current_fmd_parameters(&mut self, params: fmd::Parameters) {
        self.put(fmd::state_key::parameters::current().into(), params)
    }

    /// Writes the previous FMD parameters to the JMT.
    fn put_previous_fmd_parameters(&mut self, params: fmd::Parameters) {
        self.put(fmd::state_key::parameters::previous().into(), params)
    }
}

impl<T: StateWrite> StateWriteExt for T {}
