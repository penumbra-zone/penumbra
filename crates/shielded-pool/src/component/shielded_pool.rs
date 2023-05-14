use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::{genesis, NoteSource, SpendInfo};
use penumbra_component::Component;
use penumbra_crypto::{asset, Value};
use penumbra_crypto::{note, Nullifier};
use penumbra_proto::StateReadProto;
use penumbra_storage::StateRead;
use penumbra_storage::StateWrite;
use tendermint::abci;

use crate::state_key;

use super::{NoteManager, SupplyWrite};

pub struct ShieldedPool {}

#[async_trait]
impl Component for ShieldedPool {
    type AppState = genesis::AppState;

    // #[instrument(name = "shielded_pool", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: &genesis::AppState) {
        // Register a denom for each asset in the genesis state
        for allocation in &app_state.allocations {
            tracing::debug!(?allocation, "processing allocation");

            assert_ne!(
                allocation.amount,
                0u128.into(),
                "Genesis allocations contain empty note",
            );

            let unit = asset::REGISTRY.parse_unit(&allocation.denom);

            state.register_denom(&unit.base()).await.unwrap();
            state
                .mint_note(
                    Value {
                        amount: (u128::from(allocation.amount)
                            * 10u128.pow(unit.exponent().into()))
                        .into(),
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
    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    // #[instrument(name = "shielded_pool", skip(state, _end_block))]
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
    async fn note_source(&self, note_commitment: note::Commitment) -> Result<Option<NoteSource>> {
        self.get(&state_key::note_source(&note_commitment)).await
    }

    async fn check_nullifier_unspent(&self, nullifier: Nullifier) -> Result<()> {
        if let Some(info) = self
            .get::<SpendInfo>(&state_key::spent_nullifier_lookup(&nullifier))
            .await?
        {
            return Err(anyhow!(
                "nullifier {} was already spent in {:?}",
                nullifier,
                info.note_source,
            ));
        }
        Ok(())
    }
    async fn spend_info(&self, nullifier: Nullifier) -> Result<Option<SpendInfo>> {
        self.get(&state_key::spent_nullifier_lookup(&nullifier))
            .await
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}
