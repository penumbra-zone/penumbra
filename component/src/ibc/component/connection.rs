use std::sync::Arc;

use crate::ibc::{event, validate_penumbra_client_state, ConnectionCounter, SUPPORTED_VERSIONS};
use crate::Component;
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
use penumbra_chain::genesis;
use penumbra_proto::core::ibc::v1alpha1::ibc_action::Action::{
    ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry,
};
use penumbra_storage::{State, StateRead, StateTransaction, StateWrite};
use penumbra_transaction::Transaction;
use tendermint::abci;
use tracing::instrument;

use super::client::StateWriteExt as _;
use super::state_key;
use penumbra_proto::{StateReadProto, StateWriteProto};

mod execution;
mod stateful;
mod stateless;

pub struct ConnectionComponent {}

#[async_trait]
impl Component for ConnectionComponent {
    #[instrument(name = "ibc_connection", skip(_state, _app_state))]
    async fn init_chain(_state: &mut StateTransaction, _app_state: &genesis::AppState) {}

    #[instrument(name = "ibc_connection", skip(_state, _begin_block))]
    async fn begin_block(_state: &mut StateTransaction, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "ibc_connection", skip(tx))]
    fn check_tx_stateless(tx: Arc<Transaction>) -> Result<()> {
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

    #[instrument(name = "ibc_connection", skip(state, tx))]
    async fn check_tx_stateful(state: Arc<State>, tx: Arc<Transaction>) -> Result<()> {
        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                Some(ConnectionOpenInit(msg)) => {
                    use stateful::connection_open_init::ConnectionOpenInitCheck;
                    let msg = MsgConnectionOpenInit::try_from(msg.clone())?;

                    state.validate(&msg).await?;
                }
                Some(ConnectionOpenTry(msg)) => {
                    use stateful::connection_open_try::ConnectionOpenTryCheck;
                    let msg = MsgConnectionOpenTry::try_from(msg.clone())?;

                    state.validate(&msg).await?;
                }
                Some(ConnectionOpenAck(msg)) => {
                    use stateful::connection_open_ack::ConnectionOpenAckCheck;
                    let msg = MsgConnectionOpenAck::try_from(msg.clone())?;

                    state.validate(&msg).await?;
                }
                Some(ConnectionOpenConfirm(msg)) => {
                    use stateful::connection_open_confirm::ConnectionOpenConfirmCheck;
                    let msg = MsgConnectionOpenConfirm::try_from(msg.clone())?;

                    state.validate(&msg).await?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    #[instrument(name = "ibc_connection", skip(state, tx))]
    async fn execute_tx(state: &mut StateTransaction, tx: Arc<Transaction>) -> Result<()> {
        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                Some(ConnectionOpenInit(msg)) => {
                    use execution::connection_open_init::ConnectionOpenInitExecute;
                    let msg_connection_open_init =
                        MsgConnectionOpenInit::try_from(msg.clone()).unwrap();
                    state.execute(&msg_connection_open_init).await;
                }

                Some(ConnectionOpenTry(raw_msg)) => {
                    use execution::connection_open_try::ConnectionOpenTryExecute;
                    let msg = MsgConnectionOpenTry::try_from(raw_msg.clone()).unwrap();
                    state.execute(&msg).await;
                }

                Some(ConnectionOpenAck(raw_msg)) => {
                    use execution::connection_open_ack::ConnectionOpenAckExecute;
                    let msg = MsgConnectionOpenAck::try_from(raw_msg.clone()).unwrap();
                    state.execute(&msg).await;
                }

                Some(ConnectionOpenConfirm(raw_msg)) => {
                    use execution::connection_open_confirm::ConnectionOpenConfirmExecute;
                    let msg = MsgConnectionOpenConfirm::try_from(raw_msg.clone()).unwrap();
                    state.execute(&msg).await;
                }

                _ => {}
            }
        }

        Ok(())
    }

    #[instrument(name = "ibc_connection", skip(_state, _end_block))]
    async fn end_block(_state: &mut StateTransaction, _end_block: &abci::request::EndBlock) {}
}

#[async_trait]
pub trait StateWriteExt: StateWrite {
    fn put_connection_counter(&mut self, counter: ConnectionCounter) {
        self.put(state_key::connection_counter().into(), counter);
    }

    // puts a new connection into the state, updating the connections associated with the client,
    // and incrementing the client counter.
    async fn put_new_connection(
        &mut self,
        connection_id: &ConnectionId,
        connection: ConnectionEnd,
    ) -> Result<()> {
        self.put(state_key::connection(connection_id), connection.clone());
        let counter = self
            .get_connection_counter()
            .await
            .unwrap_or(ConnectionCounter(0));
        self.put_connection_counter(ConnectionCounter(counter.0 + 1));

        self.add_connection_to_client(connection.client_id(), connection_id)
            .await?;

        return Ok(());
    }

    fn update_connection(&mut self, connection_id: &ConnectionId, connection: ConnectionEnd) {
        self.put(state_key::connection(connection_id), connection);
    }
}

impl<T: StateWrite> StateWriteExt for T {}

#[async_trait]
pub trait StateReadExt: StateRead {
    async fn get_connection_counter(&self) -> Result<ConnectionCounter> {
        self.get(state_key::connection_counter())
            .await
            .map(|counter| counter.unwrap_or(ConnectionCounter(0)))
    }

    async fn get_connection(&self, connection_id: &ConnectionId) -> Result<Option<ConnectionEnd>> {
        self.get(&state_key::connection(connection_id)).await
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}
