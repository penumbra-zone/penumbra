use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics03_connection::connection::{ConnectionEnd, State};
use ibc::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
use ibc::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
use ibc::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use ibc::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use ibc::core::ics03_connection::version::Version;
use ibc::core::ics24_host::identifier::ConnectionId;
use penumbra_ibc::{Connection, ConnectionCounter, IBCAction};
use penumbra_proto::ibc::ibc_action::Action::{
    ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry,
};
use penumbra_storage::{Overlay, OverlayExt};
use penumbra_transaction::Transaction;
use tendermint::abci;
use tracing::instrument;

use crate::{components::ibc::client::View as _, components::Component, genesis};

pub struct ConnectionComponent {
    overlay: Overlay,
}

#[async_trait]
impl Component for ConnectionComponent {
    #[instrument(name = "ibc_connection", skip(overlay))]
    async fn new(overlay: Overlay) -> Self {
        Self { overlay }
    }

    #[instrument(name = "ibc_connection", skip(self, _app_state))]
    async fn init_chain(&mut self, _app_state: &genesis::AppState) {}

    #[instrument(name = "ibc_connection", skip(self, _begin_block))]
    async fn begin_block(&mut self, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "ibc_connection", skip(tx))]
    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
        for ibc_action in tx.ibc_actions() {
            validate_ibc_action_stateless(ibc_action)?;
        }

        Ok(())
    }

    #[instrument(name = "ibc_connection", skip(self, tx))]
    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()> {
        for ibc_action in tx.ibc_actions() {
            self.validate_ibc_action_stateful(ibc_action).await?;
        }
        Ok(())
    }

    #[instrument(name = "ibc_connection", skip(self, tx))]
    async fn execute_tx(&mut self, tx: &Transaction) {
        for ibc_action in tx.ibc_actions() {
            self.execute_ibc_action(ibc_action).await;
        }
    }

    #[instrument(name = "ibc_connection", skip(self, _end_block))]
    async fn end_block(&mut self, _end_block: &abci::request::EndBlock) {}
}

fn validate_ibc_action_stateless(ibc_action: &IBCAction) -> Result<(), anyhow::Error> {
    match &ibc_action.action {
        ConnectionOpenInit(msg) => {
            let msg_connection_open_init = MsgConnectionOpenInit::try_from(msg.clone())?;

            // check if the version is supported (we use the same versions as the cosmos SDK)
            // TODO: should we be storing the compatible versions in our state instead?
            let compatible_versions = vec![Version::default()];

            if !compatible_versions.contains(&msg_connection_open_init.version) {
                return Err(anyhow::anyhow!(
                    "unsupported version: in ConnectionOpenInit",
                ));
            }

            return Ok(());
        }

        ConnectionOpenTry(msg) => {
            let _msg_connection_open_try = MsgConnectionOpenTry::try_from(msg.clone())?;
        }

        ConnectionOpenAck(msg) => {
            let _msg_connection_open_ack = MsgConnectionOpenAck::try_from(msg.clone())?;
        }

        ConnectionOpenConfirm(msg) => {
            let _msg_connection_open_confirm = MsgConnectionOpenConfirm::try_from(msg.clone())?;
        }

        _ => {}
    }

    Ok(())
}

impl ConnectionComponent {
    async fn execute_ibc_action(&mut self, ibc_action: &IBCAction) {
        match &ibc_action.action {
            ConnectionOpenInit(msg) => {
                let msg_connection_open_init =
                    MsgConnectionOpenInit::try_from(msg.clone()).unwrap();
                self.execute_connection_open_init(&msg_connection_open_init)
                    .await;
            }

            ConnectionOpenTry(msg) => {
                let _msg_connection_open_try = MsgConnectionOpenTry::try_from(msg.clone()).unwrap();
            }

            ConnectionOpenAck(msg) => {
                let _msg_connection_open_ack = MsgConnectionOpenAck::try_from(msg.clone()).unwrap();
            }

            ConnectionOpenConfirm(msg) => {
                let _msg_connection_open_confirm =
                    MsgConnectionOpenConfirm::try_from(msg.clone()).unwrap();
            }

            _ => {}
        }
    }

    async fn execute_connection_open_init(&mut self, msg: &MsgConnectionOpenInit) {
        let connection_id =
            ConnectionId::new(self.overlay.get_connection_counter().await.unwrap().0);

        let compatible_versions = vec![Version::default()];

        let new_connection_end = ConnectionEnd::new(
            State::Init,
            msg.client_id.clone(),
            msg.counterparty.clone(),
            compatible_versions,
            msg.delay_period,
        );

        // commit the connection, this also increments the connection counter
        self.overlay
            .put_new_connection(&connection_id, new_connection_end.into())
            .await
            .unwrap();
    }

    async fn validate_ibc_action_stateful(&self, ibc_action: &IBCAction) -> Result<()> {
        match &ibc_action.action {
            ConnectionOpenInit(raw_msg) => {
                // check that the client id exists
                let msg = MsgConnectionOpenInit::try_from(raw_msg.clone())?;
                self.overlay.get_client_data(&msg.client_id).await?;

                return Ok(());
            }

            ConnectionOpenTry(msg) => {
                let _msg_connection_open_try = MsgConnectionOpenTry::try_from(msg.clone())?;
            }

            ConnectionOpenAck(msg) => {
                let _msg_connection_open_ack = MsgConnectionOpenAck::try_from(msg.clone())?;
            }

            ConnectionOpenConfirm(msg) => {
                let _msg_connection_open_confirm = MsgConnectionOpenConfirm::try_from(msg.clone())?;
            }

            _ => {}
        }

        Ok(())
    }
}

#[async_trait]
pub trait View: OverlayExt + Send + Sync {
    async fn get_connection_counter(&self) -> Result<ConnectionCounter> {
        self.get_domain("ibc/ics03-connection/connection_counter".into())
            .await
            .map(|counter| counter.unwrap_or(ConnectionCounter(0)))
    }

    async fn put_connection_counter(&self, counter: ConnectionCounter) {
        self.put_domain("ibc/ics03-connection/connection_counter".into(), counter)
            .await;
    }

    // puts a new connection into the state, updating the connections associated with the client,
    // and incrementing the client counter.
    async fn put_new_connection(
        &mut self,
        connection_id: &ConnectionId,
        connection: Connection,
    ) -> Result<()> {
        self.put_domain(
            format!(
                "ibc/ics03-connection/connections/{}",
                connection_id.as_str()
            )
            .into(),
            connection.clone(),
        )
        .await;
        let counter = self
            .get_connection_counter()
            .await
            .unwrap_or(ConnectionCounter(0));
        self.put_connection_counter(ConnectionCounter(counter.0 + 1))
            .await;

        self.add_connection_to_client(connection.0.client_id(), connection_id)
            .await?;

        return Ok(());
    }
}

impl<T: OverlayExt + Send + Sync> View for T {}
