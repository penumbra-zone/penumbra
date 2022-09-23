use crate::ibc::component::client::View as _;
use crate::ibc::{event, validate_penumbra_client_state, ConnectionCounter, SUPPORTED_VERSIONS};
use crate::{Component, Context};
use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::client_consensus::ConsensusState;
use ibc::core::ics02_client::client_def::AnyClient;
use ibc::core::ics02_client::client_def::ClientDef;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics03_connection::connection::Counterparty;
use ibc::core::ics03_connection::connection::{ConnectionEnd, State as ConnectionState};
use ibc::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
use ibc::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
use ibc::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use ibc::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use ibc::core::ics03_connection::version::{pick_version, Version};
use ibc::core::ics24_host::identifier::ConnectionId;
use ibc::Height as IBCHeight;
use penumbra_chain::{genesis, View as _};
use penumbra_proto::core::ibc::v1alpha1::ibc_action::Action::{
    ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry,
};
use penumbra_storage::{State, StateExt};
use penumbra_transaction::Transaction;
use tendermint::abci;
use tracing::instrument;

use super::state_key;

mod execution;
mod stateful;
mod stateless;

pub struct ConnectionComponent {
    state: State,
}

impl ConnectionComponent {
    #[instrument(name = "ibc_connection", skip(state))]
    pub async fn new(state: State) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Component for ConnectionComponent {
    #[instrument(name = "ibc_connection", skip(self, _app_state))]
    async fn init_chain(&mut self, _app_state: &genesis::AppState) {}

    #[instrument(name = "ibc_connection", skip(self, _ctx, _begin_block))]
    async fn begin_block(&mut self, _ctx: Context, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "ibc_connection", skip(_ctx, tx))]
    fn check_tx_stateless(_ctx: Context, tx: &Transaction) -> Result<()> {
        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                Some(ConnectionOpenInit(msg)) => {
                    use stateless::connection_open_init::*;
                    let msg = MsgConnectionOpenInit::try_from(msg.clone())?;

                    version_is_supported(&msg)?;
                }
                Some(ConnectionOpenTry(msg)) => {
                    use stateless::connection_open_try::*;
                    let msg = MsgConnectionOpenTry::try_from(msg.clone())?;

                    has_client_state(&msg)?;
                    has_client_proof(&msg)?;
                    has_consensus_proof(&msg)?;
                }
                Some(ConnectionOpenAck(msg)) => {
                    use stateless::connection_open_ack::*;
                    let msg = MsgConnectionOpenAck::try_from(msg.clone())?;

                    has_client_state(&msg)?;
                    has_client_proof(&msg)?;
                    has_consensus_proof(&msg)?;
                }

                Some(ConnectionOpenConfirm(msg)) => {
                    // NOTE: other than that the message is a well formed ConnectionOpenConfirm,
                    // there is no other stateless validation to perform.
                    let _ = MsgConnectionOpenConfirm::try_from(msg.clone())?;
                }

                _ => {}
            }
        }

        Ok(())
    }

    #[instrument(name = "ibc_connection", skip(self, _ctx, tx))]
    async fn check_tx_stateful(&self, _ctx: Context, tx: &Transaction) -> Result<()> {
        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                Some(ConnectionOpenInit(msg)) => {
                    use stateful::connection_open_init::ConnectionOpenInitCheck;
                    let msg = MsgConnectionOpenInit::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                }
                Some(ConnectionOpenTry(msg)) => {
                    use stateful::connection_open_try::ConnectionOpenTryCheck;
                    let msg = MsgConnectionOpenTry::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                }
                Some(ConnectionOpenAck(msg)) => {
                    use stateful::connection_open_ack::ConnectionOpenAckCheck;
                    let msg = MsgConnectionOpenAck::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                }
                Some(ConnectionOpenConfirm(msg)) => {
                    use stateful::connection_open_confirm::ConnectionOpenConfirmCheck;
                    let msg = MsgConnectionOpenConfirm::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    #[instrument(name = "ibc_connection", skip(self, ctx, tx))]
    async fn execute_tx(&mut self, ctx: Context, tx: &Transaction) {
        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                Some(ConnectionOpenInit(msg)) => {
                    use execution::connection_open_init::ConnectionOpenInitExecute;
                    let msg_connection_open_init =
                        MsgConnectionOpenInit::try_from(msg.clone()).unwrap();
                    self.state
                        .execute(ctx.clone(), &msg_connection_open_init)
                        .await;
                }

                Some(ConnectionOpenTry(raw_msg)) => {
                    use execution::connection_open_try::ConnectionOpenTryExecute;
                    let msg = MsgConnectionOpenTry::try_from(raw_msg.clone()).unwrap();
                    self.state.execute(ctx.clone(), &msg).await;
                }

                Some(ConnectionOpenAck(raw_msg)) => {
                    use execution::connection_open_ack::ConnectionOpenAckExecute;
                    let msg = MsgConnectionOpenAck::try_from(raw_msg.clone()).unwrap();
                    self.state.execute(ctx.clone(), &msg).await;
                }

                Some(ConnectionOpenConfirm(raw_msg)) => {
                    use execution::connection_open_confirm::ConnectionOpenConfirmExecute;
                    let msg = MsgConnectionOpenConfirm::try_from(raw_msg.clone()).unwrap();
                    self.state.execute(ctx.clone(), &msg).await;
                }

                _ => {}
            }
        }
    }

    #[instrument(name = "ibc_connection", skip(self, _ctx, _end_block))]
    async fn end_block(&mut self, _ctx: Context, _end_block: &abci::request::EndBlock) {}
}

#[async_trait]
pub trait View: StateExt + Send + Sync {
    async fn get_connection_counter(&self) -> Result<ConnectionCounter> {
        self.get_domain(state_key::connection_counter().into())
            .await
            .map(|counter| counter.unwrap_or(ConnectionCounter(0)))
    }

    async fn put_connection_counter(&self, counter: ConnectionCounter) {
        self.put_domain(state_key::connection_counter().into(), counter)
            .await;
    }

    // puts a new connection into the state, updating the connections associated with the client,
    // and incrementing the client counter.
    async fn put_new_connection(
        &mut self,
        connection_id: &ConnectionId,
        connection: ConnectionEnd,
    ) -> Result<()> {
        self.put_domain(
            state_key::connection(connection_id).into(),
            connection.clone(),
        )
        .await;
        let counter = self
            .get_connection_counter()
            .await
            .unwrap_or(ConnectionCounter(0));
        self.put_connection_counter(ConnectionCounter(counter.0 + 1))
            .await;

        self.add_connection_to_client(connection.client_id(), connection_id)
            .await?;

        return Ok(());
    }

    async fn get_connection(&self, connection_id: &ConnectionId) -> Result<Option<ConnectionEnd>> {
        self.get_domain(state_key::connection(connection_id).into())
            .await
    }

    async fn update_connection(&self, connection_id: &ConnectionId, connection: ConnectionEnd) {
        self.put_domain(state_key::connection(connection_id).into(), connection)
            .await;
    }
}

impl<T: StateExt + Send + Sync> View for T {}
