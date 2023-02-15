use crate::ibc::{event, validate_penumbra_client_state, ConnectionCounter, SUPPORTED_VERSIONS};
use crate::Component;
use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics02_client::consensus_state::ConsensusState;
// TODO(erwan): remove in polish MERGEBLOCK
// use ibc::core::ics02_client::client_def::AnyClient;
// use ibc::core::ics02_client::client_def::ClientDef;
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
use penumbra_storage::{StateRead, StateWrite};
use tendermint::abci;
use tracing::instrument;

use super::state_key;
use penumbra_proto::{StateReadProto, StateWriteProto};

pub(crate) mod execution;
pub(crate) mod stateful;
pub(crate) mod stateless;

pub struct ConnectionComponent {}

#[async_trait]
impl Component for ConnectionComponent {
    #[instrument(name = "ibc_connection", skip(_state, _app_state))]
    async fn init_chain<S: StateWrite>(_state: S, _app_state: &genesis::AppState) {}

    #[instrument(name = "ibc_connection", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite>(_state: S, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "ibc_connection", skip(_state, _end_block))]
    async fn end_block<S: StateWrite>(_state: S, _end_block: &abci::request::EndBlock) {}
}

#[async_trait]
pub trait StateWriteExt: StateWrite {
    fn put_connection_counter(&mut self, counter: ConnectionCounter) {
        self.put(state_key::connections::counter().into(), counter);
    }

    // puts a new connection into the state, updating the connections associated with the client,
    // and incrementing the client counter.
    async fn put_new_connection(
        &mut self,
        connection_id: &ConnectionId,
        connection: ConnectionEnd,
    ) -> Result<()> {
        self.put(
            state_key::connections::by_connection_id(connection_id),
            connection.clone(),
        );
        self.put(
            state_key::connections::by_client_id(connection.client_id(), connection_id),
            connection.clone(),
        );
        let counter = self
            .get_connection_counter()
            .await
            .unwrap_or(ConnectionCounter(0));
        self.put_connection_counter(ConnectionCounter(counter.0 + 1));

        return Ok(());
    }

    fn update_connection(&mut self, connection_id: &ConnectionId, connection: ConnectionEnd) {
        self.put(
            state_key::connections::by_connection_id(connection_id),
            connection.clone(),
        );
        self.put(
            state_key::connections::by_client_id(connection.client_id(), connection_id),
            connection,
        );
    }
}

impl<T: StateWrite> StateWriteExt for T {}

#[async_trait]
pub trait StateReadExt: StateRead {
    async fn get_connection_counter(&self) -> Result<ConnectionCounter> {
        self.get(state_key::connections::counter())
            .await
            .map(|counter| counter.unwrap_or(ConnectionCounter(0)))
    }

    async fn get_connection(&self, connection_id: &ConnectionId) -> Result<Option<ConnectionEnd>> {
        self.get(&state_key::connections::by_connection_id(connection_id))
            .await
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}
