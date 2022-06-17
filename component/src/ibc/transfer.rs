use crate::ibc::ibc_handler::{AppHandler, AppHandlerCheck, AppHandlerExecute};
use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
use ibc::core::ics04_channel::msgs::chan_close_confirm::MsgChannelCloseConfirm;
use ibc::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
use ibc::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
use ibc::core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm;
use ibc::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use ibc::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
use ibc::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
use ibc::core::ics04_channel::msgs::timeout::MsgTimeout;
use penumbra_storage::State;
use tracing::instrument;

pub struct ICS20Transfer {
    state: State,
}

impl ICS20Transfer {
    #[instrument(name = "ics20_transfer", skip(state))]
    pub fn new(state: State) -> Self {
        Self { state }
    }
}

// TODO: ICS20 implementation.
// see: https://github.com/cosmos/ibc/tree/master/spec/app/ics-020-fungible-token-transfer
#[async_trait]
impl AppHandlerCheck for ICS20Transfer {
    async fn chan_open_init_check(&self, _msg: &MsgChannelOpenInit) -> Result<()> {
        Ok(())
    }
    async fn chan_open_try_check(&self, _msg: &MsgChannelOpenTry) -> Result<()> {
        Ok(())
    }
    async fn chan_open_ack_check(&self, _msg: &MsgChannelOpenAck) -> Result<()> {
        Ok(())
    }
    async fn chan_open_confirm_check(&self, _msg: &MsgChannelOpenConfirm) -> Result<()> {
        Ok(())
    }
    async fn chan_close_confirm_check(&self, _msg: &MsgChannelCloseConfirm) -> Result<()> {
        Ok(())
    }
    async fn chan_close_init_check(&self, _msg: &MsgChannelCloseInit) -> Result<()> {
        Ok(())
    }
    async fn recv_packet_check(&self, _msg: &MsgRecvPacket) -> Result<()> {
        Ok(())
    }
    async fn timeout_packet_check(&self, _msg: &MsgTimeout) -> Result<()> {
        Ok(())
    }
    async fn acknowledge_packet_check(&self, _msg: &MsgAcknowledgement) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl AppHandlerExecute for ICS20Transfer {
    async fn chan_open_init_execute(&mut self, _msg: &MsgChannelOpenInit) {}
    async fn chan_open_try_execute(&mut self, _msg: &MsgChannelOpenTry) {}
    async fn chan_open_ack_execute(&mut self, _msg: &MsgChannelOpenAck) {}
    async fn chan_open_confirm_execute(&mut self, _msg: &MsgChannelOpenConfirm) {}
    async fn chan_close_confirm_execute(&mut self, _msg: &MsgChannelCloseConfirm) {}
    async fn chan_close_init_execute(&mut self, _msg: &MsgChannelCloseInit) {}
    async fn recv_packet_execute(&mut self, _msg: &MsgRecvPacket) {}
    async fn timeout_packet_execute(&mut self, _msg: &MsgTimeout) {}
    async fn acknowledge_packet_execute(&mut self, _msg: &MsgAcknowledgement) {}
}

impl AppHandler for ICS20Transfer {}
