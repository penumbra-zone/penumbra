// Many of the IBC message types are enums, where the number of variants differs
// depending on the build configuration, meaning that the fallthrough case gets
// marked as unreachable only when not building in test configuration.
#![allow(unreachable_patterns)]

pub(crate) mod channel;
pub(crate) mod client;
pub(crate) mod connection;
pub(crate) mod state_key;

use std::sync::Arc;

use crate::ibc::transfer::Ics20Transfer;
use crate::Component;
use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::{genesis, StateReadExt as _};
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::Transaction;
use tendermint::abci;
use tracing::instrument;

pub struct IBCComponent {}

#[async_trait]
impl Component for IBCComponent {
    #[instrument(name = "ibc", skip(state, app_state))]
    async fn init_chain(state: &mut StateTransaction, app_state: &genesis::AppState) {
        client::Ics2Client::init_chain(state, app_state).await;
        connection::ConnectionComponent::init_chain(state, app_state).await;
        channel::Ics4Channel::init_chain(state, app_state).await;
        Ics20Transfer::init_chain(state, app_state).await;
    }

    #[instrument(name = "ibc", skip(state, begin_block))]
    async fn begin_block(state: &mut StateTransaction, begin_block: &abci::request::BeginBlock) {
        client::Ics2Client::begin_block(state, begin_block).await;
        connection::ConnectionComponent::begin_block(state, begin_block).await;
        channel::Ics4Channel::begin_block(state, begin_block).await;
        Ics20Transfer::begin_block(state, begin_block).await;
    }

    #[instrument(name = "ibc", skip(state, end_block))]
    async fn end_block(state: &mut StateTransaction, end_block: &abci::request::EndBlock) {
        client::Ics2Client::end_block(state, end_block).await;
        connection::ConnectionComponent::end_block(state, end_block).await;
        channel::Ics4Channel::end_block(state, end_block).await;
        Ics20Transfer::end_block(state, end_block).await;
    }
}
