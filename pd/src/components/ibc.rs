use anyhow::Result;
use async_trait::async_trait;
use penumbra_transaction::{Action, Transaction};
use std::convert::TryFrom;
use tendermint::abci;
use tracing::instrument;

use ibc::core::{
    ics02_client::msgs::create_client::MsgCreateAnyClient, ics24_host::identifier::ClientId,
};
use penumbra_proto::ibc::ibc_action::Action::CreateClient;

use super::{Component, Overlay};
use crate::genesis;

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
        self.overlay
            .lock()
            .await
            .put(b"ibc/ics02-client/client_counter".into(), 0u64.to_le_bytes().to_vec());

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
                _ => todo!(),
            }
        }

        todo!()
    }

    async fn end_block(&mut self, _end_block: &abci::request::EndBlock) -> Result<()> {
        Ok(())
    }
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
    async fn store_new_client(&mut self, client_id: ClientId, msg: MsgCreateAnyClient) -> Result<()> {
        let client_state = msg.client_state;
        let consensus_state = msg.consensus_state;

        // store the client state
        self.overlay
            .lock()
            .await
            .put(format!("ibc/ics02-client/client_states/{}", hex::encode(client_id.as_bytes())).into(), serde_json::to_string(&client_state)?.into());

        // increment the client counter
        let next_counter = self.client_counter().await? + 1;
        self
            .overlay
            .lock()
            .await
            .put(b"ibc/ics02-client/client_counter".into(), next_counter.to_le_bytes().to_vec());
        
        // store the consensus state
        // TODO: should we store block height here as well?
        self.overlay
            .lock()
            .await
            .put(format!("ibc/ics02-client/consensus_states/{}", hex::encode(client_id.as_bytes())).into(), serde_json::to_string(&consensus_state)?.into());
        
        // TODO: store update time and update height
        Ok(())
    }
}

/*   // Construct this client's identifier
let id_counter = ctx.client_counter()?;
let client_id = ClientId::new(msg.client_state.client_type(), id_counter).map_err(|e| {
    Error::client_identifier_constructor(msg.client_state.client_type(), id_counter, e)
})?;

output.log(format!(
    "success: generated new client identifier: {}",
    client_id
));

let result = ClientResult::Create(Result {
    client_id: client_id.clone(),
    client_type: msg.client_state.client_type(),
    client_state: msg.client_state.clone(),
    consensus_state: msg.consensus_state,
    processed_time: ctx.host_timestamp(),
    processed_height: ctx.host_height(),
});

let event_attributes = Attributes {
    client_id,
    ..Default::default()
};
output.emit(IbcEvent::CreateClient(event_attributes.into()));

Ok(output.with_result(result)) */
