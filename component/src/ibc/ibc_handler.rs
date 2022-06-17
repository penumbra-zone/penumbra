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
use ibc::core::ics24_host::identifier::PortId;
use std::collections::HashMap;

// defines an asynchronous handler for core IBC events, to be implemented by IBC apps (such as an
// ICS20 transfer implementation, or interchain accounts implementation).
#[async_trait]
pub trait AppHandler: Send + Sync {
    async fn chan_open_init(&mut self, msg: &MsgChannelOpenInit);
    async fn chan_open_try(&mut self, msg: &MsgChannelOpenTry);
    async fn chan_open_ack(&mut self, msg: &MsgChannelOpenAck);
    async fn chan_open_confirm(&mut self, msg: &MsgChannelOpenConfirm);
    async fn chan_close_confirm(&mut self, msg: &MsgChannelCloseConfirm);
    async fn chan_close_init(&mut self, msg: &MsgChannelCloseInit);

    async fn recv_packet(&mut self, msg: &MsgRecvPacket);
    async fn timeout_packet(&mut self, msg: &MsgTimeout);
    async fn acknowledge_packet(&mut self, msg: &MsgAcknowledgement);
}

pub struct AppRouter {
    handlers: HashMap<PortId, Box<dyn AppHandler>>,
}

impl AppRouter {
    pub fn new() -> Self {
        AppRouter {
            handlers: HashMap::new(),
        }
    }
    pub fn bind(&mut self, port_id: PortId, handler: Box<dyn AppHandler>) {
        if self.handlers.contains_key(&port_id) {
            panic!("AppRouter: handler already bound for port {}", port_id);
        }
        self.handlers.insert(port_id, handler);
    }
}

#[async_trait]
impl AppHandler for AppRouter {
    async fn chan_open_init(&mut self, msg: &MsgChannelOpenInit) {
        if let Some(handler) = self.handlers.get_mut(&msg.port_id) {
            handler.chan_open_init(msg).await;
        }
    }
    async fn chan_open_try(&mut self, msg: &MsgChannelOpenTry) {
        if let Some(handler) = self.handlers.get_mut(&msg.port_id) {
            handler.chan_open_try(msg).await;
        }
    }
    async fn chan_open_ack(&mut self, msg: &MsgChannelOpenAck) {
        if let Some(handler) = self.handlers.get_mut(&msg.port_id) {
            handler.chan_open_ack(msg).await;
        }
    }
    async fn chan_open_confirm(&mut self, msg: &MsgChannelOpenConfirm) {
        if let Some(handler) = self.handlers.get_mut(&msg.port_id) {
            handler.chan_open_confirm(msg).await;
        }
    }
    async fn chan_close_confirm(&mut self, msg: &MsgChannelCloseConfirm) {
        if let Some(handler) = self.handlers.get_mut(&msg.port_id) {
            handler.chan_close_confirm(msg).await;
        }
    }
    async fn chan_close_init(&mut self, msg: &MsgChannelCloseInit) {
        if let Some(handler) = self.handlers.get_mut(&msg.port_id) {
            handler.chan_close_init(msg).await;
        }
    }
    async fn recv_packet(&mut self, msg: &MsgRecvPacket) {
        if let Some(handler) = self.handlers.get_mut(&msg.packet.destination_port) {
            handler.recv_packet(msg).await;
        }
    }
    async fn timeout_packet(&mut self, msg: &MsgTimeout) {
        if let Some(handler) = self.handlers.get_mut(&msg.packet.destination_port) {
            handler.timeout_packet(msg).await;
        }
    }
    async fn acknowledge_packet(&mut self, msg: &MsgAcknowledgement) {
        if let Some(handler) = self.handlers.get_mut(&msg.packet.destination_port) {
            handler.acknowledge_packet(msg).await;
        }
    }
}
