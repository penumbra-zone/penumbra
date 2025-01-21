use std::sync::Arc;

use super::fmd::ClueManagerInternal as _;
use crate::fmd::should_update_fmd_params;
use crate::params::ShieldedPoolParameters;
use crate::{fmd, genesis, state_key};
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::Component;
use penumbra_sdk_proto::StateReadProto as _;
use penumbra_sdk_proto::StateWriteProto as _;
use penumbra_sdk_sct::CommitmentSource;
use tendermint::v0_37::abci;
use tracing::instrument;

use super::{AssetRegistry, NoteManager};

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

                    state.register_denom(&allocation.denom()).await;
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

    #[instrument(name = "shielded_pool", skip_all)]
    async fn end_block<S: StateWrite + 'static>(
        state: &mut Arc<S>,
        end_block: &abci::request::EndBlock,
    ) {
        let height: u64 = end_block
            .height
            .try_into()
            .expect("height should not be negative");
        let state = Arc::get_mut(state).expect("the state should not be shared");
        let meta_params = state
            .get_shielded_pool_params()
            .await
            .expect("should be able to read state")
            .fmd_meta_params;
        if should_update_fmd_params(meta_params.fmd_grace_period_blocks, height) {
            let old = state
                .get_current_fmd_parameters()
                .await
                .expect("should be able to read state");
            let clue_count_delta = state
                .flush_clue_count()
                .await
                .expect("should be able to read state");
            let algorithm_state = state
                .get_fmd_algorithm_state()
                .await
                .expect("should be able to read state");
            let (new, algorithm_state) =
                meta_params.updated_fmd_params(&old, algorithm_state, height, clue_count_delta);
            state.put_previous_fmd_parameters(old);
            state.put_current_fmd_parameters(new);
            state.put_fmd_algorithm_state(algorithm_state);
        }
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

    async fn get_fmd_algorithm_state(&self) -> Result<fmd::MetaParametersAlgorithmState> {
        Ok(self
            .get(fmd::state_key::meta_parameters::algorithm_state())
            .await?
            .unwrap_or_default())
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

/// Extension trait providing write access to shielded pool data.
#[async_trait]
pub trait StateWriteExt: StateWrite + StateReadExt {
    fn put_shielded_pool_params(&mut self, params: ShieldedPoolParameters) {
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

    fn put_fmd_algorithm_state(&mut self, state: fmd::MetaParametersAlgorithmState) {
        self.put(
            fmd::state_key::meta_parameters::algorithm_state().into(),
            state,
        )
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
