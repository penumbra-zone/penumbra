mod fee_pay;
pub mod rpc;
mod view;

use std::sync::Arc;

use crate::{event::EventBlockFees, genesis, Fee};
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::Component;
use penumbra_sdk_proto::state::StateWriteProto as _;
use penumbra_sdk_proto::DomainType as _;
use tendermint::abci;
use tracing::instrument;

pub use fee_pay::FeePay;
pub use view::{StateReadExt, StateWriteExt};

// Fee component
pub struct FeeComponent {}

#[async_trait]
impl Component for FeeComponent {
    type AppState = genesis::Content;

    #[instrument(name = "fee", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            Some(genesis) => {
                state.put_fee_params(genesis.fee_params.clone());
            }
            None => { /* perform upgrade specific check */ }
        }
    }

    #[instrument(name = "fee", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    #[instrument(name = "fee", skip(state, _end_block))]
    async fn end_block<S: StateWrite + 'static>(
        state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
        let state_ref = Arc::get_mut(state).expect("unique ref in end_block");
        // Grab the total fees and use them to emit an event.
        let fees = state_ref.accumulated_base_fees_and_tips();

        let (swapped_base, swapped_tip) = fees
            .get(&penumbra_sdk_asset::STAKING_TOKEN_ASSET_ID)
            .cloned()
            .unwrap_or_default();

        let swapped_total = swapped_base + swapped_tip;

        state_ref.record_proto(
            EventBlockFees {
                swapped_fee_total: Fee::from_staking_token_amount(swapped_total),
                swapped_base_fee_total: Fee::from_staking_token_amount(swapped_base),
                swapped_tip_total: Fee::from_staking_token_amount(swapped_tip),
            }
            .to_proto(),
        );
    }

    #[instrument(name = "fee", skip(_state))]
    async fn end_epoch<S: StateWrite + 'static>(_state: &mut Arc<S>) -> anyhow::Result<()> {
        Ok(())
    }
}
