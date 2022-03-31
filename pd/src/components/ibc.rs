use anyhow::Result;
use async_trait::async_trait;
use penumbra_transaction::{Action, Transaction};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use tendermint::abci;
use tracing::instrument;

use ibc::core::{
    ics02_client::{
        client_consensus::AnyConsensusState, client_state::AnyClientState,
        msgs::create_client::MsgCreateAnyClient,
    },
    ics24_host::identifier::ClientId,
};
use penumbra_proto::ibc::ibc_action::Action::CreateClient;

use super::{Component, Overlay};
use crate::{genesis, PenumbraStore};

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
        self.overlay.lock().await.put(
            b"ibc/ics02-client/client_counter".into(),
            0u64.to_le_bytes().to_vec(),
        );

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
        for action in tx.transaction_body.actions.iter() {
            let ibc_action = match action {
                Action::IBCAction(ibc_action) => Some(ibc_action),
                _ => None,
            };
            if ibc_action.is_none() {
                continue;
            }

            match &ibc_action.unwrap().action {
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
                    let id_counter = self.client_counter().await?;
                    let client_id =
                        ClientId::new(msg_create_client.client_state.client_type(), id_counter)?;

                    tracing::info!("creating client {:?}", client_id);

                    self.store_new_client(client_id, msg_create_client).await?;
                }
                _ => continue,
            }
        }

        todo!()
    }

    async fn end_block(&mut self, _end_block: &abci::request::EndBlock) -> Result<()> {
        Ok(())
    }
}

#[derive(Serialize)]
struct ClientData {
    pub client_id: ClientId,
    pub client_state: AnyClientState,
    pub consensus_state: AnyConsensusState,
    pub processed_time: String,
    pub processed_height: u64,
}

impl IBCComponent {
    async fn client_counter(&mut self) -> Result<u64> {
        let count_bytes = self
            .overlay
            .lock()
            .await
            .get(b"ibc/client_counter".into())
            .await?
            .ok_or(anyhow::anyhow!("no client count"))?;

        Ok(u64::from_le_bytes(count_bytes.try_into().unwrap()))
    }
    async fn store_new_client(
        &mut self,
        client_id: ClientId,
        msg: MsgCreateAnyClient,
    ) -> Result<()> {
        let height = self.overlay.get_block_height().await?;
        let timestamp = self.overlay.get_block_timestamp().await?;
        let data = ClientData {
            client_id: client_id.clone(),
            client_state: msg.client_state,
            consensus_state: msg.consensus_state,
            processed_time: timestamp.to_rfc3339(),
            processed_height: height,
        };

        // store the client data
        self.overlay.lock().await.put(
            format!("ibc/clients/{}", hex::encode(client_id.as_bytes())).into(),
            serde_json::to_vec(&data).unwrap(),
        );

        Ok(())
    }
}
