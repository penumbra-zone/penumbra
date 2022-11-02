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
use crate::ibc::transfer::ICS20Transfer;
use crate::{Component, Context};
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
    client: client::Ics2Client,
    connection: connection::ConnectionComponent,
    channel: channel::ICS4Channel,
    transfer: ICS20Transfer,
}

impl IBCComponent {
    #[instrument(name = "ibc")]
    pub async fn new() -> Self {
        let client = Ics2Client::new().await;
        let connection = connection::ConnectionComponent::new().await;

        let mut router = AppRouter::new();
        let transfer = ICS20Transfer::new();
        router.bind(PortId::transfer(), Box::new(transfer.clone()));

        let channel = channel::ICS4Channel::new(Box::new(router)).await;

        Self {
            channel,
            client,
            connection,
            transfer,
        }
    }
}

#[async_trait]
impl Component for IBCComponent {
    #[instrument(name = "ibc", skip(app_state))]
    async fn init_chain(state: &mut StateTransaction, app_state: &genesis::AppState) {
        client::Ics2Client::init_chain(app_state).await;
        connection::ConnectionComponent::init_chain(app_state).await;
        channel::ICS4Channel::init_chain(app_state).await;
        ICS20Transfer::init_chain(app_state).await;
    }

    #[instrument(name = "ibc", skip(begin_block, ctx))]
    async fn begin_block(
        state: &mut StateTransaction,
        ctx: Context,
        begin_block: &abci::request::BeginBlock,
    ) {
        client::Ics2Client::begin_block(ctx.clone(), begin_block).await;
        connection::ConnectionComponent::begin_block(ctx.clone(), begin_block).await;
        channel::ICS4Channel::begin_block(ctx.clone(), begin_block).await;
        ICS20Transfer::begin_block(ctx.clone(), begin_block).await;
    }

    #[instrument(name = "ibc", skip(tx, ctx))]
    fn check_tx_stateless(ctx: Context, tx: Arc<Transaction>) -> Result<()> {
        client::Ics2Client::check_tx_stateless(ctx.clone(), tx)?;
        connection::ConnectionComponent::check_tx_stateless(ctx.clone(), tx)?;
        channel::ICS4Channel::check_tx_stateless(ctx.clone(), tx)?;
        ICS20Transfer::check_tx_stateless(ctx.clone(), tx)?;

        Ok(())
    }

    #[instrument(name = "ibc", skip(ctx, tx))]
    async fn check_tx_stateful(
        state: Arc<State>,
        ctx: Context,
        tx: Arc<Transaction>,
    ) -> Result<()> {
        if tx.ibc_actions().count() > 0 && !state.get_chain_params().await?.ibc_enabled {
            return Err(anyhow::anyhow!(
                "transaction contains IBC actions, but IBC is not enabled"
            ));
        }

        client::Ics2Client::check_tx_stateful(ctx.clone(), tx, state.clone()).await?;
        connection::ConnectionComponent::check_tx_stateful(ctx.clone(), tx, state.clone()).await?;
        channel::ICS4Channel::check_tx_stateful(ctx.clone(), tx, state.clone()).await?;
        ICS20Transfer::check_tx_stateful(ctx.clone(), tx, state.clone()).await?;

        Ok(())
    }

    #[instrument(name = "ibc", skip(state, ctx, tx))]
    async fn execute_tx(
        state: &mut StateTransaction,
        ctx: Context,
        tx: Arc<Transaction>,
    ) -> Result<()> {
        client::Ics2Client::execute_tx(ctx.clone(), tx, state).await;
        connection::ConnectionComponent::execute_tx(ctx.clone(), tx, state).await;
        channel::ICS4Channel::execute_tx(ctx.clone(), tx, state).await;
        ICS20Transfer::execute_tx(ctx.clone(), tx, state).await;

        Ok(())
    }

    #[instrument(name = "ibc", skip(state, ctx, end_block))]
    async fn end_block(
        state: &mut StateTransaction,
        ctx: Context,
        end_block: &abci::request::EndBlock,
    ) {
        client::Ics2Client::end_block(ctx.clone(), end_block, state).await;
        connection::ConnectionComponent::end_block(ctx.clone(), end_block, state).await;
        channel::ICS4Channel::end_block(ctx.clone(), end_block, state).await;
        ICS20Transfer::end_block(ctx.clone(), end_block, state).await;
    }
}
