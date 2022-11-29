use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::msgs::create_client::MsgCreateAnyClient;
use ibc::core::ics02_client::msgs::update_client::MsgUpdateAnyClient;
use ibc::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
use ibc::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
use ibc::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use ibc::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use ibc::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
use ibc::core::ics04_channel::msgs::chan_close_confirm::MsgChannelCloseConfirm;
use ibc::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
use ibc::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
use ibc::core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm;
use ibc::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use ibc::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
use ibc::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
use ibc::core::ics04_channel::msgs::timeout::MsgTimeout;
use penumbra_proto::core::ibc::v1alpha1::ibc_action::Action::{
    Acknowledgement, ChannelCloseConfirm, ChannelCloseInit, ChannelOpenAck, ChannelOpenConfirm,
    ChannelOpenInit, ChannelOpenTry, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
    ConnectionOpenTry, CreateClient, RecvPacket, Timeout, UpdateClient,
};
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

        match &self.action {
            Some(CreateClient(msg)) => {
                let msg = MsgCreateAnyClient::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }
            Some(UpdateClient(msg)) => {
                let msg = MsgUpdateAnyClient::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }
            Some(ChannelOpenInit(msg)) => {
                let msg = MsgChannelOpenInit::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }
            Some(ChannelOpenTry(msg)) => {
                let msg = MsgChannelOpenTry::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }
            Some(ChannelOpenAck(msg)) => {
                let msg = MsgChannelOpenAck::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }
            Some(ChannelOpenConfirm(msg)) => {
                let msg = MsgChannelOpenConfirm::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }
            Some(ChannelCloseInit(msg)) => {
                let msg = MsgChannelCloseInit::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }
            Some(ChannelCloseConfirm(msg)) => {
                let msg = MsgChannelCloseConfirm::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }
            Some(RecvPacket(msg)) => {
                let msg = MsgRecvPacket::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }
            Some(Acknowledgement(msg)) => {
                let msg = MsgAcknowledgement::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }
            Some(Timeout(msg)) => {
                let msg = MsgTimeout::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }
            Some(ConnectionOpenInit(msg)) => {
                let msg = MsgConnectionOpenInit::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }
            Some(ConnectionOpenTry(msg)) => {
                let msg = MsgConnectionOpenTry::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }
            Some(ConnectionOpenAck(msg)) => {
                let msg = MsgConnectionOpenAck::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }

            Some(ConnectionOpenConfirm(msg)) => {
                let msg = MsgConnectionOpenConfirm::try_from(msg.clone())?;

                msg.check_stateless(context).await?;
            }
            _ => {}
        }

        Ok(())
    }

    #[instrument(name = "ibc_action", skip(self, state, context))]
    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        match &self.action {
            Some(CreateClient(msg)) => {
                let msg = MsgCreateAnyClient::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            Some(UpdateClient(msg)) => {
                let msg = MsgUpdateAnyClient::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            Some(ChannelOpenInit(msg)) => {
                let msg = MsgChannelOpenInit::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            Some(ChannelOpenTry(msg)) => {
                let msg = MsgChannelOpenTry::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            Some(ChannelOpenAck(msg)) => {
                let msg = MsgChannelOpenAck::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            Some(ChannelOpenConfirm(msg)) => {
                let msg = MsgChannelOpenConfirm::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            Some(ChannelCloseInit(msg)) => {
                let msg = MsgChannelCloseInit::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            Some(ChannelCloseConfirm(msg)) => {
                let msg = MsgChannelCloseConfirm::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            Some(RecvPacket(msg)) => {
                let msg = MsgRecvPacket::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            Some(Acknowledgement(msg)) => {
                let msg = MsgAcknowledgement::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            Some(Timeout(msg)) => {
                let msg = MsgTimeout::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            Some(ConnectionOpenInit(msg)) => {
                let msg = MsgConnectionOpenInit::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            Some(ConnectionOpenTry(msg)) => {
                let msg = MsgConnectionOpenTry::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            Some(ConnectionOpenAck(msg)) => {
                let msg = MsgConnectionOpenAck::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            Some(ConnectionOpenConfirm(msg)) => {
                let msg = MsgConnectionOpenConfirm::try_from(msg.clone())?;

                msg.check_stateful(state, context).await?;
            }
            _ => {}
        }
        Ok(())
    }

    #[instrument(name = "ibc_action", skip(self, state))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        // Handle the message type of this IBC action.
        match &self.action {
            Some(CreateClient(raw_msg_create_client)) => {
                let msg = MsgCreateAnyClient::try_from(raw_msg_create_client.clone()).unwrap();

                msg.execute(state).await?;
            }
            Some(UpdateClient(raw_msg_update_client)) => {
                let msg = MsgUpdateAnyClient::try_from(raw_msg_update_client.clone()).unwrap();

                msg.execute(state).await?;
            }
            Some(ChannelOpenInit(msg)) => {
                let msg = MsgChannelOpenInit::try_from(msg.clone()).unwrap();

                msg.execute(state).await?;
            }
            Some(ChannelOpenTry(msg)) => {
                let msg = MsgChannelOpenTry::try_from(msg.clone()).unwrap();

                msg.execute(state).await?;
            }
            Some(ChannelOpenAck(msg)) => {
                let msg = MsgChannelOpenAck::try_from(msg.clone()).unwrap();

                msg.execute(state).await?;
            }
            Some(ChannelOpenConfirm(msg)) => {
                let msg = MsgChannelOpenConfirm::try_from(msg.clone()).unwrap();

                msg.execute(state).await?;
            }
            Some(ChannelCloseInit(msg)) => {
                let msg = MsgChannelCloseInit::try_from(msg.clone()).unwrap();

                msg.execute(state).await?;
            }
            Some(ChannelCloseConfirm(msg)) => {
                let msg = MsgChannelCloseConfirm::try_from(msg.clone()).unwrap();

                msg.execute(state).await?;
            }
            Some(RecvPacket(msg)) => {
                let msg = MsgRecvPacket::try_from(msg.clone()).unwrap();

                msg.execute(state).await?;
            }
            Some(Acknowledgement(msg)) => {
                let msg = MsgAcknowledgement::try_from(msg.clone()).unwrap();

                msg.execute(state).await?;
            }
            Some(Timeout(msg)) => {
                let msg = MsgTimeout::try_from(msg.clone()).unwrap();

                msg.execute(state).await?;
            }
            Some(ConnectionOpenInit(msg)) => {
                let msg = MsgConnectionOpenInit::try_from(msg.clone()).unwrap();

                msg.execute(state).await?;
            }

            Some(ConnectionOpenTry(raw_msg)) => {
                let msg = MsgConnectionOpenTry::try_from(raw_msg.clone()).unwrap();

                msg.execute(state).await?;
            }

            Some(ConnectionOpenAck(raw_msg)) => {
                let msg = MsgConnectionOpenAck::try_from(raw_msg.clone()).unwrap();

                msg.execute(state).await?;
            }

            Some(ConnectionOpenConfirm(raw_msg)) => {
                let msg = MsgConnectionOpenConfirm::try_from(raw_msg.clone()).unwrap();

                msg.execute(state).await?;
            }
            _ => {}
        }

        Ok(())
    }
}
