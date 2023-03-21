use crate::Component;
use async_trait::async_trait;
use penumbra_chain::{genesis, NoteSource};
use penumbra_crypto::{asset, Value};
use penumbra_storage::StateWrite;
use tendermint::abci;

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
    }

    // #[instrument(name = "shielded_pool", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite>(_state: S, _begin_block: &abci::request::BeginBlock) {}

    // #[instrument(name = "shielded_pool", skip(state, _end_block))]
    async fn end_block<S: StateWrite>(mut _state: S, _end_block: &abci::request::EndBlock) {}
}
