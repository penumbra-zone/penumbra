use anyhow::{Context as _, Result};
use async_trait::async_trait;
use penumbra_chain::genesis;
use penumbra_storage::{State, StateExt};
use penumbra_transaction::Transaction;
use tendermint::abci;
use tracing::instrument;

use crate::{Component, Context};

use super::{check, event, metrics, state_key};

pub struct Governance {
    state: State,
}

#[async_trait]
impl Component for Governance {
    #[instrument(name = "governance", skip(self, app_state))]
    async fn init_chain(&mut self, app_state: &genesis::AppState) {}

    #[instrument(name = "governance", skip(self, _ctx, _begin_block))]
    async fn begin_block(&mut self, _ctx: Context, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "governance", skip(_ctx, tx))]
    fn check_tx_stateless(_ctx: Context, tx: &Transaction) -> Result<()> {
        Ok(())
    }

    #[instrument(name = "governance", skip(self, _ctx, tx))]
    async fn check_tx_stateful(&self, _ctx: Context, tx: &Transaction) -> Result<()> {
        Ok(())
    }

    #[instrument(name = "governance", skip(self, ctx, tx))]
    async fn execute_tx(&mut self, ctx: Context, tx: &Transaction) {}

    #[instrument(name = "governance", skip(self, _ctx, _end_block))]
    async fn end_block(&mut self, _ctx: Context, _end_block: &abci::request::EndBlock) {}
}

impl Governance {}

#[async_trait]
pub trait View: StateExt {}

impl<T: StateExt> View for T {}
