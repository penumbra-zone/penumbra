use crate::Component;
use async_trait::async_trait;
use penumbra_chain::{genesis, NoteSource, StateReadExt as _};
use penumbra_crypto::{asset, Value};
use penumbra_storage::StateWrite;
use tendermint::abci;

use crate::compactblock::view::{StateReadExt as _, StateWriteExt as _};
use crate::sct::view::{StateReadExt as _, StateWriteExt as _};

use super::{NoteManager, SupplyWrite};

pub struct ShieldedPool {}

#[async_trait]
impl Component for ShieldedPool {
    // #[instrument(name = "shielded_pool", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: &genesis::AppState) {
        // Register a denom for each asset in the genesis state
        for allocation in &app_state.allocations {
            tracing::debug!(?allocation, "processing allocation");

            assert_ne!(
                allocation.amount, 0u64,
                "Genesis allocations contain empty note",
            );

            let unit = asset::REGISTRY.parse_unit(&allocation.denom);

            state.register_denom(&unit.base()).await.unwrap();
            state
                .mint_note(
                    Value {
                        amount: (allocation.amount * 10u64.pow(unit.exponent().into())).into(),
                        asset_id: unit.id(),
                    },
                    &allocation.address,
                    NoteSource::Genesis,
                )
                .await
                .unwrap();
        }

        let mut compact_block = state.stub_compact_block();
        let mut state_commitment_tree = state.stub_state_commitment_tree().await;

        // Hard-coded to zero because we are in the genesis block
        // Tendermint starts blocks at 1, so this is a "phantom" compact block
        compact_block.height = 0;

        // Add current FMD parameters to the initial block.
        compact_block.fmd_parameters = Some(state.get_current_fmd_parameters().await.unwrap());

        // Close the genesis block

        // TODO: MOVE TO APP
        state
            .finish_sct_block(&mut compact_block, &mut state_commitment_tree)
            .await;

        state.set_compact_block(compact_block.clone());

        state
            .write_sct(
                compact_block.height,
                state_commitment_tree,
                compact_block.block_root,
                compact_block.epoch_root,
            )
            .await;
    }

    // #[instrument(name = "shielded_pool", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite>(_state: S, _begin_block: &abci::request::BeginBlock) {}

    // #[instrument(name = "shielded_pool", skip(state, _end_block))]
    async fn end_block<S: StateWrite>(mut _state: S, _end_block: &abci::request::EndBlock) {}
}
