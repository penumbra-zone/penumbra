use crate::ibc::ibc_handler::AppHandler;
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
impl AppHandler for ICS20Transfer {
    async fn chan_open_init(&mut self, _msg: &MsgChannelOpenInit) {}
    async fn chan_open_try(&mut self, _msg: &MsgChannelOpenTry) {}
    async fn chan_open_ack(&mut self, _msg: &MsgChannelOpenAck) {}
    async fn chan_open_confirm(&mut self, _msg: &MsgChannelOpenConfirm) {}
    async fn chan_close_confirm(&mut self, _msg: &MsgChannelCloseConfirm) {}
    async fn chan_close_init(&mut self, _msg: &MsgChannelCloseInit) {}
    async fn recv_packet(&mut self, _msg: &MsgRecvPacket) {}
    async fn timeout_packet(&mut self, _msg: &MsgTimeout) {}
    async fn acknowledge_packet(&mut self, _msg: &MsgAcknowledgement) {}
}
