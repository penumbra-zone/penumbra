use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use ibc_types::{
    core::{
        client::ClientId,
        connection::ConnectionId,
        connection::{ClientPaths, ConnectionEnd},
    },
    path::{ClientConnectionPath, ConnectionPath},
};
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};

use crate::{prefix::MerklePrefixExt, IBC_COMMITMENT_PREFIX};

use super::{connection_counter::ConnectionCounter, state_key};

#[async_trait]
pub trait StateWriteExt: StateWrite {
    fn put_connection_counter(&mut self, counter: ConnectionCounter) {
        self.put(state_key::counter().into(), counter);
    }
    fn put_client_connection(&mut self, client_id: &ClientId, paths: ClientPaths) {
        self.put(
            IBC_COMMITMENT_PREFIX.apply_string(ClientConnectionPath::new(client_id).to_string()),
            paths.clone(),
        );
    }

    // puts a new connection into the state, updating the connections associated with the client,
    // and incrementing the client counter.
    async fn put_new_connection(
        &mut self,
        connection_id: &ConnectionId,
        connection: ConnectionEnd,
    ) -> Result<()> {
        self.put(
            IBC_COMMITMENT_PREFIX.apply_string(ConnectionPath::new(connection_id).to_string()),
            connection.clone(),
        );

        let mut client_paths = self.get_client_connections(&connection.client_id).await?;
        client_paths.paths.push(connection_id.clone());
        self.put_client_connection(&connection.client_id, client_paths);

        let counter = self
            .get_connection_counter()
            .await
            .unwrap_or(ConnectionCounter(0));
        self.put_connection_counter(ConnectionCounter(counter.0 + 1));

        return Ok(());
    }

    fn update_connection(&mut self, connection_id: &ConnectionId, connection: ConnectionEnd) {
        self.put(
            IBC_COMMITMENT_PREFIX.apply_string(ConnectionPath::new(connection_id).to_string()),
            connection.clone(),
        );
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}

#[async_trait]
pub trait StateReadExt: StateRead {
    async fn get_connection_counter(&self) -> Result<ConnectionCounter> {
        self.get(state_key::counter())
            .await
            .map(|counter| counter.unwrap_or(ConnectionCounter(0)))
    }

    async fn get_connection(&self, connection_id: &ConnectionId) -> Result<Option<ConnectionEnd>> {
        self.get(
            &IBC_COMMITMENT_PREFIX.apply_string(ConnectionPath::new(connection_id).to_string()),
        )
        .await
    }

    async fn get_client_connections(&self, client_id: &ClientId) -> Result<ClientPaths> {
        self.get(
            &IBC_COMMITMENT_PREFIX.apply_string(ClientConnectionPath::new(client_id).to_string()),
        )
        .await
        .map(|paths| paths.unwrap_or(ClientPaths { paths: vec![] }))
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}
