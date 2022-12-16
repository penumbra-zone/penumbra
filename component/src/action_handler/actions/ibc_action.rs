use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::msgs::create_client::{MsgCreateClient, TYPE_URL as CREATE_CLIENT};
use ibc::core::ics02_client::msgs::update_client::{MsgUpdateClient, TYPE_URL as UPDATE_CLIENT};
use ibc::core::ics03_connection::msgs::conn_open_ack::{
    MsgConnectionOpenAck, TYPE_URL as CONNECTION_OPEN_ACK,
};
use ibc::core::ics03_connection::msgs::conn_open_confirm::{
    MsgConnectionOpenConfirm, TYPE_URL as CONNECTION_OPEN_CONFIRM,
};
use ibc::core::ics03_connection::msgs::conn_open_init::{
    MsgConnectionOpenInit, TYPE_URL as CONNECTION_OPEN_INIT,
};
use ibc::core::ics03_connection::msgs::conn_open_try::{
    MsgConnectionOpenTry, TYPE_URL as CONNECTION_OPEN_TRY,
};
use ibc::core::ics04_channel::msgs::acknowledgement::{
    MsgAcknowledgement, TYPE_URL as ACKNOWLEDGEMENT,
};
use ibc::core::ics04_channel::msgs::chan_close_confirm::{
    MsgChannelCloseConfirm, TYPE_URL as CHANNEL_CLOSE_CONFIRM,
};
use ibc::core::ics04_channel::msgs::chan_close_init::{
    MsgChannelCloseInit, TYPE_URL as CHANNEL_CLOSE_INIT,
};
use ibc::core::ics04_channel::msgs::chan_open_ack::{
    MsgChannelOpenAck, TYPE_URL as CHANNEL_OPEN_ACK,
};
use ibc::core::ics04_channel::msgs::chan_open_confirm::{
    MsgChannelOpenConfirm, TYPE_URL as CHANNEL_OPEN_CONFIRM,
};
use ibc::core::ics04_channel::msgs::chan_open_init::{
    MsgChannelOpenInit, TYPE_URL as CHANNEL_OPEN_INIT,
};
use ibc::core::ics04_channel::msgs::chan_open_try::{
    MsgChannelOpenTry, TYPE_URL as CHANNEL_OPEN_TRY,
};
use ibc::core::ics04_channel::msgs::recv_packet::{MsgRecvPacket, TYPE_URL as RECV_PACKET};
use ibc::core::ics04_channel::msgs::timeout::{MsgTimeout, TYPE_URL as TIMEOUT};
use ibc_proto::protobuf::Protobuf;
use penumbra_proto::core::ibc::v1alpha1::IbcAction;
use penumbra_storage::{State, StateTransaction};
use penumbra_transaction::Transaction;
use tracing::instrument;

use crate::action_handler::ActionHandler;

mod msg;

#[async_trait]
impl ActionHandler for IbcAction {
    #[instrument(name = "ibc_action", skip(self, context))]
    async fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        // Each stateless check is a distinct function in an appropriate submodule,
        // so that we can easily add new stateless checks and see a birds' eye view
        // of all of the checks we're performing.
        let raw_action = self
            .raw_action
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("empty IBC transaction is not allowed"))?;

        let action_type = raw_action.type_url.as_str();
        let raw_action_bytes = &raw_action.value;

        match action_type {
            CREATE_CLIENT => {
                let msg = MsgCreateClient::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            UPDATE_CLIENT => {
                let msg = MsgUpdateClient::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            CONNECTION_OPEN_INIT => {
                let msg = MsgConnectionOpenInit::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            CONNECTION_OPEN_TRY => {
                let msg = MsgConnectionOpenTry::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            CONNECTION_OPEN_ACK => {
                let msg = MsgConnectionOpenAck::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            CONNECTION_OPEN_CONFIRM => {
                let msg = MsgConnectionOpenConfirm::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            ACKNOWLEDGEMENT => {
                let msg = MsgAcknowledgement::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            CHANNEL_OPEN_INIT => {
                let msg = MsgChannelOpenInit::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            CHANNEL_OPEN_TRY => {
                let msg = MsgChannelOpenTry::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            CHANNEL_OPEN_ACK => {
                let msg = MsgChannelOpenAck::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            CHANNEL_OPEN_CONFIRM => {
                let msg = MsgChannelOpenConfirm::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            CHANNEL_CLOSE_INIT => {
                let msg = MsgChannelCloseInit::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            CHANNEL_CLOSE_CONFIRM => {
                let msg = MsgChannelCloseConfirm::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            RECV_PACKET => {
                let msg = MsgRecvPacket::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            TIMEOUT => {
                let msg = MsgTimeout::decode(raw_action_bytes.as_slice())?;

                msg.check_stateless(context).await?;
            }
            _ => {
                return Err(anyhow::anyhow!("unknown IBC action type: {}", action_type));
            }
        }

        Ok(())
    }

    #[instrument(name = "ibc_action", skip(self, state))]
    async fn check_stateful(&self, state: Arc<State>) -> Result<()> {
        let raw_action = self
            .raw_action
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("empty IBC transaction is not allowed"))?;

        let action_type = raw_action.type_url.as_str();
        let raw_action_bytes = &raw_action.value;

        match action_type {
            CREATE_CLIENT => {
                let msg = MsgCreateClient::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            UPDATE_CLIENT => {
                let msg = MsgUpdateClient::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            CONNECTION_OPEN_INIT => {
                let msg = MsgConnectionOpenInit::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            CONNECTION_OPEN_TRY => {
                let msg = MsgConnectionOpenTry::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            CONNECTION_OPEN_ACK => {
                let msg = MsgConnectionOpenAck::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            CONNECTION_OPEN_CONFIRM => {
                let msg = MsgConnectionOpenConfirm::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            ACKNOWLEDGEMENT => {
                let msg = MsgAcknowledgement::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            CHANNEL_OPEN_INIT => {
                let msg = MsgChannelOpenInit::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            CHANNEL_OPEN_TRY => {
                let msg = MsgChannelOpenTry::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            CHANNEL_OPEN_ACK => {
                let msg = MsgChannelOpenAck::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            CHANNEL_OPEN_CONFIRM => {
                let msg = MsgChannelOpenConfirm::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            CHANNEL_CLOSE_INIT => {
                let msg = MsgChannelCloseInit::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            CHANNEL_CLOSE_CONFIRM => {
                let msg = MsgChannelCloseConfirm::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            RECV_PACKET => {
                let msg = MsgRecvPacket::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            TIMEOUT => {
                let msg = MsgTimeout::decode(raw_action_bytes.as_slice())?;

                msg.check_stateful(state).await?;
            }
            _ => {
                return Err(anyhow::anyhow!("unknown IBC action type: {}", action_type));
            }
        }

        Ok(())
    }

    #[instrument(name = "ibc_action", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        let raw_action = self
            .raw_action
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("empty IBC transaction is not allowed"))?;

        let action_type = raw_action.type_url.as_str();
        let raw_action_bytes = &raw_action.value;

        // Handle the message type of this IBC action.
        match action_type {
            CREATE_CLIENT => {
                let msg = MsgCreateClient::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            UPDATE_CLIENT => {
                let msg = MsgUpdateClient::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            CONNECTION_OPEN_INIT => {
                let msg = MsgConnectionOpenInit::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            CONNECTION_OPEN_TRY => {
                let msg = MsgConnectionOpenTry::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            CONNECTION_OPEN_ACK => {
                let msg = MsgConnectionOpenAck::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            CONNECTION_OPEN_CONFIRM => {
                let msg = MsgConnectionOpenConfirm::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            ACKNOWLEDGEMENT => {
                let msg = MsgAcknowledgement::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            CHANNEL_OPEN_INIT => {
                let msg = MsgChannelOpenInit::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            CHANNEL_OPEN_TRY => {
                let msg = MsgChannelOpenTry::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            CHANNEL_OPEN_ACK => {
                let msg = MsgChannelOpenAck::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            CHANNEL_OPEN_CONFIRM => {
                let msg = MsgChannelOpenConfirm::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            CHANNEL_CLOSE_INIT => {
                let msg = MsgChannelCloseInit::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            CHANNEL_CLOSE_CONFIRM => {
                let msg = MsgChannelCloseConfirm::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            RECV_PACKET => {
                let msg = MsgRecvPacket::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            TIMEOUT => {
                let msg = MsgTimeout::decode(raw_action_bytes.as_slice())?;

                msg.execute(state).await?;
            }
            _ => {
                return Err(anyhow::anyhow!("unknown IBC action type: {}", action_type));
            }
        }

        Ok(())
    }
}
