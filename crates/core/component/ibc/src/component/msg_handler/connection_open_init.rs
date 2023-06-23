use anyhow::Result;
use async_trait::async_trait;
use ibc_types2::core::connection::{
    msgs::MsgConnectionOpenInit, ConnectionEnd, ConnectionId, Version,
};

use crate::{
    component::{
        client::StateReadExt as _,
        connection::{StateReadExt as _, StateWriteExt as _},
        connection_counter::SUPPORTED_VERSIONS,
        MsgHandler,
    },
    event,
};

use ibc_types2::core::connection::State as ConnectionState;
use penumbra_storage::StateWrite;

#[async_trait]
impl MsgHandler for MsgConnectionOpenInit {
    async fn check_stateless(&self) -> Result<()> {
        version_is_supported(self)?;

        Ok(())
    }

    async fn try_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        tracing::debug!(msg = ?self);

        // check that the client with the specified ID exists
        state.get_client_state(&self.client_id_on_a).await?;
        state.get_client_type(&self.client_id_on_a).await?;

        let connection_id = ConnectionId::new(state.get_connection_counter().await.unwrap().0);

        let compatible_versions = vec![Version::default()];

        let new_connection_end = ConnectionEnd {
            state: ConnectionState::Init,
            client_id: self.client_id_on_a.clone(),
            counterparty: self.counterparty.clone(),
            versions: compatible_versions,
            delay_period: self.delay_period,
        };

        // commit the connection, this also increments the connection counter
        state
            .put_new_connection(&connection_id, new_connection_end)
            .await
            .unwrap();

        state.record(event::connection_open_init(
            &connection_id,
            &self.client_id_on_a,
            &self.counterparty,
        ));

        Ok(())
    }
}

fn version_is_supported(msg: &MsgConnectionOpenInit) -> anyhow::Result<()> {
    // check if the version is supported (we use the same versions as the cosmos SDK)
    // TODO: should we be storing the compatible versions in our state instead?

    // NOTE: version can be nil in MsgConnectionOpenInit
    if msg.version.is_none() {
        return Ok(());
    }

    if !SUPPORTED_VERSIONS.contains(
        msg.version
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("invalid version"))?,
    ) {
        Err(anyhow::anyhow!(
            "unsupported version: in ConnectionOpenInit",
        ))
    } else {
        Ok(())
    }
}
