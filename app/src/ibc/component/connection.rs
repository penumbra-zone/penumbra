use crate::ibc::ConnectionCounter;
use anyhow::Result;
use async_trait::async_trait;
// TODO(erwan): remove in polish MERGEBLOCK
// use ibc_types::core::ics02_client::client_def::AnyClient;
// use ibc_types::core::ics02_client::client_def::ClientDef;
use ibc_types::core::{
    ics03_connection::connection::ConnectionEnd, ics24_host::identifier::ConnectionId,
};
use penumbra_storage::{StateRead, StateWrite};

use super::state_key;
use penumbra_proto::{StateReadProto, StateWriteProto};

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
