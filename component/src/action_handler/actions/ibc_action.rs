use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use ibc::core::ics02_client::msgs::create_client::MsgCreateAnyClient;
use ibc::core::ics02_client::msgs::update_client::MsgUpdateAnyClient;
use penumbra_proto::core::ibc::v1alpha1::ibc_action::Action::{
    Acknowledgement, ChannelCloseConfirm, ChannelCloseInit, ChannelOpenAck, ChannelOpenConfirm,
    ChannelOpenInit, ChannelOpenTry, CreateClient, RecvPacket, Timeout, UpdateClient,
};
use penumbra_proto::core::ibc::v1alpha1::IbcAction;
use penumbra_transaction::Transaction;
use tracing::instrument;

use crate::action_handler::ActionHandler;

#[async_trait]
impl ActionHandler for IbcAction {
    #[instrument(name = "ibc_action", skip(self, context))]
    fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        // Each stateless check is a distinct function in an appropriate submodule,
        // so that we can easily add new stateless checks and see a birds' eye view
        // of all of the checks we're performing.

        match self.action {
            Some(CreateClient(msg)) => {
                use stateless::create_client::*;
                let msg = MsgCreateAnyClient::try_from(msg.clone())?;

                client_state_is_tendermint(&msg)?;
                consensus_state_is_tendermint(&msg)?;
            }
            Some(UpdateClient(msg)) => {
                use stateless::update_client::*;
                let msg = MsgUpdateAnyClient::try_from(msg.clone())?;

                header_is_tendermint(&msg)?;
            }
            // Other IBC messages are not handled by this component.
            _ => {}
        }

        Ok(())
    }

    #[instrument(name = "ibc_action", skip(self, state, context))]
    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        match self.action {
            Some(CreateClient(msg)) => {
                use stateful::create_client::CreateClientCheck;
                let msg = MsgCreateAnyClient::try_from(msg.clone())?;
                state.validate(&msg).await?;
            }
            Some(UpdateClient(msg)) => {
                use stateful::update_client::UpdateClientCheck;
                let msg = MsgUpdateAnyClient::try_from(msg.clone())?;
                state.validate(&msg).await?;
            }
            // Other IBC messages are not handled by this component.
            _ => {}
        }
        Ok(())
    }

    #[instrument(name = "ibc_action", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        // Handle any IBC actions found in the transaction.
        match self.action {
            Some(CreateClient(raw_msg_create_client)) => {
                let msg_create_client =
                    MsgCreateAnyClient::try_from(raw_msg_create_client.clone()).unwrap();

                state.execute_create_client(msg_create_client).await;
            }
            Some(UpdateClient(raw_msg_update_client)) => {
                let msg_update_client =
                    MsgUpdateAnyClient::try_from(raw_msg_update_client.clone()).unwrap();

                state.execute_update_client(msg_update_client).await;
            }
            _ => {}
        }

        Ok(())
    }
}
