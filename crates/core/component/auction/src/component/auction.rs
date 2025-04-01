use crate::component::dutch_auction::HandleDutchTriggers;
use crate::event;
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::Component;
use penumbra_sdk_asset::asset;
use penumbra_sdk_asset::Value;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::StateReadProto;
use penumbra_sdk_proto::StateWriteProto;
use std::sync::Arc;
use tap::Tap;
use tendermint::v0_37::abci;
use tracing::instrument;

use crate::{params::AuctionParameters, state_key};

pub struct Auction {}

#[async_trait]
impl Component for Auction {
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

    #[instrument(name = "auction", skip(state, end_block))]
    async fn end_block<S: StateWrite + 'static>(
        state: &mut Arc<S>,
        end_block: &abci::request::EndBlock,
    ) {
        let state: &mut S = Arc::get_mut(state).expect("state should be unique");
        let _ = state.process_triggers(end_block.height as u64).await;
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

    /// Fetch the current balance of the auction circuit breaker for a given asset,
    /// returning zero if no balance is tracked yet.
    #[instrument(skip(self))]
    async fn get_auction_value_balance_for(&self, asset_id: &asset::Id) -> Amount {
        self.get(&state_key::value_balance::for_asset(asset_id))
            .await
            .expect("failed to fetch auction value breaker balance")
            .unwrap_or_else(Amount::zero)
            .tap(|vcb| tracing::trace!(?vcb))
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

/// Internal trait implementing value flow tracking.
/// # Overview
///
///                                                               
///                                                  ║            
///                                                  ║            
///                                           User initiated      
///      Auction                                     ║            
///     component                                    ║            
///   ┏━━━━━━━━━━━┓                                  ▼            
///   ┃┌─────────┐┃                     ╔════════════════════════╗
///   ┃│    A    │┃◀━━value in━━━━━━━━━━║   Schedule auction A   ║
///   ┃└─────────┘┃                     ╚════════════════════════╝
///   ┃           ┃                                               
///   ┃     │     ┃                                               
///   ┃           ┃                     ■■■■■■■■■■■■■■■■■■■■■■■■■■
///   ┃  closed   ┃━━━value out by ━━━━▶■■■■■■Dex□black□box■■■■■■■
///   ┃   then    ┃   creating lp       ■■■■■■■■■■■■■■■■■■■■■■■■■■
///   ┃withdrawn  ┃                                  ┃            
///   ┃           ┃                                  ┃            
///   ┃     │     ┃                value in by       ┃            
///   ┃           ┃◀━━━━━━━━━━━━━━━withdrawing lp━━━━┛            
///   ┃     │     ┃                                               
///   ┃     ▼     ┃                                               
///   ┃┌ ─ ─ ─ ─ ┐┃                     ╔════════════════════════╗
///   ┃     A     ┃━━━value out━━━━━━━━▶║   Withdraw auction A   ║
///   ┃└ ─ ─ ─ ─ ┘┃                     ╚════════════════════════╝
///   ┗━━━━━━━━━━━┛                                  ▲            
///                                                  ║            
///                                                  ║            
///                                           User initiated      
///                                              withdraw        
///                                                  ║            
///                                                  ║            
///                                                  ║            
///
pub(crate) trait AuctionCircuitBreaker: StateWrite {
    /// Credit a deposit into the auction component.
    #[instrument(skip(self))]
    async fn auction_vcb_credit(&mut self, value: Value) -> Result<()> {
        if value.amount == Amount::zero() {
            tracing::trace!("short-circuit crediting zero-value");
            return Ok(());
        }

        let prev_balance = self.get_auction_value_balance_for(&value.asset_id).await;
        let new_balance = prev_balance.checked_add(&value.amount).ok_or_else(|| {
            anyhow::anyhow!("overflowed balance while crediting auction circuit breaker (prev balance: {prev_balance:?}, credit: {value:?}")
        })?;

        tracing::trace!(
            ?prev_balance,
            ?new_balance,
            "crediting the auction component VCB"
        );

        // Write the new balance to the chain state.
        self.put(
            state_key::value_balance::for_asset(&value.asset_id),
            new_balance,
        );
        // And emit an event to trace the value flow.
        self.record_proto(event::auction_vcb_credit(
            value.asset_id,
            prev_balance,
            new_balance,
        ));
        Ok(())
    }

    /// Debit a balance from the auction component.
    #[instrument(skip(self))]
    async fn auction_vcb_debit(&mut self, value: Value) -> Result<()> {
        if value.amount == Amount::zero() {
            tracing::trace!("short-circuit debiting zero-value");
            return Ok(());
        }

        let prev_balance = self.get_auction_value_balance_for(&value.asset_id).await;
        let new_balance = prev_balance.checked_sub(&value.amount).ok_or_else(|| {
            anyhow::anyhow!("underflowed balance while debiting auction circuit breaker (prev balance: {prev_balance:?}, debit={value:?}")
        })?;

        tracing::trace!(
            ?prev_balance,
            ?new_balance,
            "debiting the auction component VCB"
        );

        self.put(
            state_key::value_balance::for_asset(&value.asset_id),
            new_balance,
        );
        // And emit an event to trace the value flow out of the component.
        self.record_proto(event::auction_vcb_debit(
            value.asset_id,
            prev_balance,
            new_balance,
        ));
        Ok(())
    }
}

impl<T: StateWrite + ?Sized> AuctionCircuitBreaker for T {}

#[cfg(test)]
mod tests {}
