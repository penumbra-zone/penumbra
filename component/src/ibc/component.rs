// Many of the IBC message types are enums, where the number of variants differs
// depending on the build configuration, meaning that the fallthrough case gets
// marked as unreachable only when not building in test configuration.
#![allow(unreachable_patterns)]

pub(crate) mod channel;
pub(crate) mod client;
pub(crate) mod connection;
pub(crate) mod state_key;

use std::sync::Arc;

use crate::ibc::ibc_handler::AppRouter;
use crate::ibc::transfer::Ics20Transfer;
use crate::Component;
use anyhow::Result;
use async_trait::async_trait;
use client::Ics2Client;
use ibc::core::ics24_host::identifier::PortId;
use penumbra_chain::{genesis, StateReadExt as _};
use penumbra_storage2::{State, StateTransaction};
use penumbra_transaction::{Action, Transaction};
use tendermint::abci;
use tracing::instrument;

pub struct IBCComponent {
    // client: client::Ics2Client,
    // connection: connection::ConnectionComponent,
    // channel: channel::Ics4Channel,
    // transfer: Ics20Transfer,
}

// impl IBCComponent {
//     #[instrument(name = "ibc")]
//     pub async fn new() -> Self {
//         let client = Ics2Client::new().await;
//         let connection = connection::ConnectionComponent::new().await;

//         let mut router = AppRouter::new();
//         let transfer = Ics20Transfer::new();
//         router.bind(PortId::transfer(), Box::new(transfer.clone()));

//         let channel = channel::Ics4Channel::new(Box::new(router)).await;

//         Self {
//             channel,
//             client,
//             connection,
//             transfer,
//         }
//     }
// }

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

    #[instrument(name = "ibc", skip(tx))]
    fn check_tx_stateless(tx: Arc<Transaction>) -> Result<()> {
        client::Ics2Client::check_tx_stateless(tx)?;
        connection::ConnectionComponent::check_tx_stateless(tx)?;
        channel::Ics4Channel::check_tx_stateless(tx)?;
        Ics20Transfer::check_tx_stateless(tx)?;

        Ok(())
    }

    #[instrument(name = "ibc", skip(state, tx))]
    async fn check_tx_stateful(state: Arc<State>, tx: Arc<Transaction>) -> Result<()> {
        if tx.ibc_actions().count() > 0 && !state.get_chain_params().await?.ibc_enabled {
            return Err(anyhow::anyhow!(
                "transaction contains IBC actions, but IBC is not enabled"
            ));
        }

        client::Ics2Client::check_tx_stateful(state.clone(), tx).await?;
        connection::ConnectionComponent::check_tx_stateful(state.clone(), tx).await?;
        channel::Ics4Channel::check_tx_stateful(state.clone(), tx).await?;
        Ics20Transfer::check_tx_stateful(state.clone(), tx).await?;

        Ok(())
    }

    #[instrument(name = "ibc", skip(state, tx))]
    async fn execute_tx(state: &mut StateTransaction, tx: Arc<Transaction>) -> Result<()> {
        client::Ics2Client::execute_tx(state, tx).await;
        connection::ConnectionComponent::execute_tx(state, tx).await;
        channel::Ics4Channel::execute_tx(state, tx).await;
        Ics20Transfer::execute_tx(state, tx).await;

        Ok(())
    }

    #[instrument(name = "ibc", skip(state, end_block))]
    async fn end_block(state: &mut StateTransaction, end_block: &abci::request::EndBlock) {
        client::Ics2Client::end_block(state, end_block).await;
        connection::ConnectionComponent::end_block(state, end_block).await;
        channel::Ics4Channel::end_block(state, end_block).await;
        Ics20Transfer::end_block(state, end_block).await;
    }
}
