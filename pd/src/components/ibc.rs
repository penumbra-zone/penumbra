use anyhow::Result;
use async_trait::async_trait;
use penumbra_ibc::{ClientCounter, ClientData, ConsensusState, IBCAction};
use penumbra_transaction::{Action, Transaction};
use std::convert::TryFrom;
use tendermint::abci;
use tracing::instrument;

use ibc::core::{
    ics02_client::{client_consensus::AnyConsensusState, msgs::create_client::MsgCreateAnyClient},
    ics24_host::identifier::ClientId,
};
use penumbra_proto::ibc::ibc_action::Action::CreateClient;

use super::{app::View as _, Component, Overlay};
use crate::{genesis, WriteOverlayExt};

pub struct IBCComponent {
    overlay: Overlay,
}

#[async_trait]
impl Component for IBCComponent {
    async fn new(overlay: Overlay) -> Result<Self> {
        Ok(Self { overlay })
    }

    async fn init_chain(&mut self, _app_state: &genesis::AppState) -> Result<()> {
        // set the initial client count
        self.overlay.put_client_counter(ClientCounter(0)).await;

        Ok(())
    }

    async fn begin_block(&mut self, _begin_block: &abci::request::BeginBlock) -> Result<()> {
        Ok(())
    }

    fn check_tx_stateless(_tx: &Transaction) -> Result<()> {
        Ok(())
    }

    async fn check_tx_stateful(&self, _tx: &Transaction) -> Result<()> {
        Ok(())
    }

    async fn execute_tx(&mut self, tx: &Transaction) -> Result<()> {
        // handle client transactions
        for ibc_action in tx
            .transaction_body
            .actions
            .iter()
            .filter_map(|action| match action {
                Action::IBCAction(ibc_action) => Some(ibc_action),
                _ => None,
            })
        {
            self.handle_ibc_action(ibc_action).await?;
        }

        Ok(())
    }

    async fn end_block(&mut self, _end_block: &abci::request::EndBlock) -> Result<()> {
        Ok(())
    }
}

impl IBCComponent {
    async fn handle_ibc_action(&mut self, ibc_action: &IBCAction) -> Result<()> {
        match &ibc_action.action {
            // Handle IBC CreateClient. Here we need to validate the following:
            // - client type is one of the supported types (currently, only Tendermint light clients)
            // - consensus state is valid (is of the same type as the client type, also currently only Tendermint consensus states are permitted)
            //
            // Then, we compute the client's ID (a concatenation of a monotonically increasing
            // integer, the number of clients on Penumbra, and the client type) and commit the
            // following to our state:
            // - client type
            // - consensus state
            // - processed time and height
            CreateClient(raw_msg_create_client) => {
                // NOTE: MsgCreateAnyClient::try_from will validate that client_state and
                // consensus_state are Tendermint client and consensus states only, as these
                // are the only currently supported client types.
                let msg_create_client =
                    MsgCreateAnyClient::try_from(raw_msg_create_client.clone())?;

                // get the current client counter
                let id_counter = self.overlay.client_counter().await?;
                let client_id =
                    ClientId::new(msg_create_client.client_state.client_type(), id_counter.0)?;

                tracing::info!("creating client {:?}", client_id);

                self.store_new_client(client_id, msg_create_client).await?;
            }
            _ => return Ok(()),
        }

        Ok(())
    }
    async fn store_new_client(
        &mut self,
        client_id: ClientId,
        msg: MsgCreateAnyClient,
    ) -> Result<()> {
        let height = self.overlay.get_block_height().await?;
        let timestamp = self.overlay.get_block_timestamp().await?;

        let data = ClientData::new(
            client_id.clone(),
            msg.client_state,
            timestamp.to_rfc3339(),
            height,
        );

        // store the client data
        self.overlay.put_client_data(data).await;

        // store the genesis consensus state
        self.overlay
            .put_consensus_state(height, client_id, ConsensusState(msg.consensus_state))
            .await;

        // increment client counter
        let counter = self
            .overlay
            .client_counter()
            .await
            .unwrap_or(ClientCounter(0));
        self.overlay
            .put_client_counter(ClientCounter(counter.0 + 1))
            .await;

        Ok(())
    }
}

#[async_trait]
pub trait View: WriteOverlayExt + Send + Sync {
    async fn put_client_counter(&mut self, counter: ClientCounter) {
        self.put_domain("ibc/ics02-client/client_counter".into(), counter)
            .await;
    }
    async fn client_counter(&self) -> Result<ClientCounter> {
        self.get_domain("ibc/ics02-client/client_counter".into())
            .await
            .map(|counter| counter.unwrap_or(ClientCounter(0)))
    }
    async fn put_client_data(&mut self, data: ClientData) {
        self.put_domain(
            format!(
                "ibc/ics02-client/clients/{}",
                hex::encode(data.client_id.as_bytes())
            )
            .into(),
            data,
        )
        .await;
    }
    async fn put_consensus_state(
        &mut self,
        height: u64,
        client_id: ClientId,
        consensus_state: ConsensusState,
    ) {
        self.put_domain(
            format!(
                "ibc/ics02-client/clients/{}/consensus_state/{}",
                hex::encode(client_id.as_bytes()),
                height
            )
            .into(),
            consensus_state,
        )
        .await;
    }
}

impl<T: WriteOverlayExt + Send + Sync> View for T {}
