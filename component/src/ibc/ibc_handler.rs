use std::collections::BTreeMap;

/// IBC "app handlers" for the IBC component.
///
/// An app handler defines an interface for any IBC application to consume verified IBC events from
/// the core IBC component. IBC applications listen to channel events that occur on the Port ID
/// that they have subscribed to, and apply application-specific state transition logic.
///
/// The primary IBC application is the Ics20 transfer application, which allows for interchain
/// token transfers.
use crate::Context;
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
use ibc::core::ics24_host::identifier::PortId;

/// AppHandlerCheck defines the interface for an IBC application to consume IBC channel and packet
/// events, and apply their validation logic. This validation logic is used for stateful validation
/// only.
#[async_trait]
pub trait AppHandlerCheck: Send + Sync {
    async fn chan_open_init_check(&self, msg: &MsgChannelOpenInit) -> Result<()>;
    async fn chan_open_try_check(&self, msg: &MsgChannelOpenTry) -> Result<()>;
    async fn chan_open_ack_check(&self, msg: &MsgChannelOpenAck) -> Result<()>;
    async fn chan_open_confirm_check(&self, msg: &MsgChannelOpenConfirm) -> Result<()>;
    async fn chan_close_confirm_check(&self, msg: &MsgChannelCloseConfirm) -> Result<()>;
    async fn chan_close_init_check(&self, msg: &MsgChannelCloseInit) -> Result<()>;

    async fn recv_packet_check(&self, msg: &MsgRecvPacket) -> Result<()>;
    async fn timeout_packet_check(&self, msg: &MsgTimeout) -> Result<()>;
    async fn acknowledge_packet_check(&self, msg: &MsgAcknowledgement) -> Result<()>;
}

// AppHandlerExecute defines the interface for an IBC application to consume IBC channel and packet
// events and apply their state transition logic. The IBC component will only call these methods
// once the transaction has been validated using the AppHandlerCheck interface.
#[async_trait]
pub trait AppHandlerExecute: Send + Sync {
    async fn chan_open_init_execute(&mut self, msg: &MsgChannelOpenInit);
    async fn chan_open_try_execute(&mut self, msg: &MsgChannelOpenTry);
    async fn chan_open_ack_execute(&mut self, msg: &MsgChannelOpenAck);
    async fn chan_open_confirm_execute(&mut self, msg: &MsgChannelOpenConfirm);
    async fn chan_close_confirm_execute(&mut self, msg: &MsgChannelCloseConfirm);
    async fn chan_close_init_execute(&mut self, msg: &MsgChannelCloseInit);

    async fn recv_packet_execute(&mut self, msg: &MsgRecvPacket);
    async fn timeout_packet_execute(&mut self, msg: &MsgTimeout);
    async fn acknowledge_packet_execute(&mut self, msg: &MsgAcknowledgement);
}

pub trait AppHandler: AppHandlerCheck + AppHandlerExecute {}

/// an AppRouter is an implementation of AppHandler that is the root router for all IBC
/// applications. Applications can register themselves on an IBC port by calling AppRouter.bind().
pub struct AppRouter {
    handlers: BTreeMap<PortId, Box<dyn AppHandler>>,
}

impl AppRouter {
    pub fn new() -> Self {
        AppRouter {
            handlers: BTreeMap::new(),
        }
    }

    /// Bind an IBC application, given by `handler`, to the given port ID. This will panic if there
    /// is already an application bound to the given port ID.
    pub fn bind(&mut self, port_id: PortId, handler: Box<dyn AppHandler>) {
        if self.handlers.contains_key(&port_id) {
            panic!("AppRouter: handler already bound for port {}", port_id);
        }
        self.handlers.insert(port_id, handler);
    }
}

#[async_trait]
impl AppHandlerCheck for AppRouter {
    async fn chan_open_init_check(&self, msg: &MsgChannelOpenInit) -> Result<()> {
        if let Some(handler) = self.handlers.get(&msg.port_id) {
            handler.chan_open_init_check(ctx, msg).await?;
        }
        Ok(())
    }
    async fn chan_open_try_check(&self, msg: &MsgChannelOpenTry) -> Result<()> {
        if let Some(handler) = self.handlers.get(&msg.port_id) {
            handler.chan_open_try_check(ctx, msg).await?;
        }
        Ok(())
    }
    async fn chan_open_ack_check(&self, msg: &MsgChannelOpenAck) -> Result<()> {
        if let Some(handler) = self.handlers.get(&msg.port_id) {
            handler.chan_open_ack_check(ctx, msg).await?;
        }
        Ok(())
    }
    async fn chan_open_confirm_check(&self, msg: &MsgChannelOpenConfirm) -> Result<()> {
        if let Some(handler) = self.handlers.get(&msg.port_id) {
            handler.chan_open_confirm_check(ctx, msg).await?;
        }
        Ok(())
    }
    async fn chan_close_confirm_check(&self, msg: &MsgChannelCloseConfirm) -> Result<()> {
        if let Some(handler) = self.handlers.get(&msg.port_id) {
            handler.chan_close_confirm_check(ctx, msg).await?;
        }
        Ok(())
    }
    async fn chan_close_init_check(&self, msg: &MsgChannelCloseInit) -> Result<()> {
        if let Some(handler) = self.handlers.get(&msg.port_id) {
            handler.chan_close_init_check(ctx, msg).await?;
        }
        Ok(())
    }
    async fn recv_packet_check(&self, msg: &MsgRecvPacket) -> Result<()> {
        if let Some(handler) = self.handlers.get(&msg.packet.destination_port) {
            handler.recv_packet_check(ctx, msg).await?;
        }
        Ok(())
    }
    async fn timeout_packet_check(&self, msg: &MsgTimeout) -> Result<()> {
        if let Some(handler) = self.handlers.get(&msg.packet.destination_port) {
            handler.timeout_packet_check(ctx, msg).await?;
        }
        Ok(())
    }
    async fn acknowledge_packet_check(&self, msg: &MsgAcknowledgement) -> Result<()> {
        if let Some(handler) = self.handlers.get(&msg.packet.destination_port) {
            handler.acknowledge_packet_check(ctx, msg).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl AppHandlerExecute for AppRouter {
    async fn chan_open_init_execute(&mut self, msg: &MsgChannelOpenInit) {
        if let Some(handler) = self.handlers.get_mut(&msg.port_id) {
            handler.chan_open_init_execute(ctx, msg).await;
        }
    }
    async fn chan_open_try_execute(&mut self, msg: &MsgChannelOpenTry) {
        if let Some(handler) = self.handlers.get_mut(&msg.port_id) {
            handler.chan_open_try_execute(ctx, msg).await;
        }
    }
    async fn chan_open_ack_execute(&mut self, msg: &MsgChannelOpenAck) {
        if let Some(handler) = self.handlers.get_mut(&msg.port_id) {
            handler.chan_open_ack_execute(ctx, msg).await;
        }
    }
    async fn chan_open_confirm_execute(&mut self, msg: &MsgChannelOpenConfirm) {
        if let Some(handler) = self.handlers.get_mut(&msg.port_id) {
            handler.chan_open_confirm_execute(ctx, msg).await;
        }
    }
    async fn chan_close_confirm_execute(&mut self, msg: &MsgChannelCloseConfirm) {
        if let Some(handler) = self.handlers.get_mut(&msg.port_id) {
            handler.chan_close_confirm_execute(ctx, msg).await;
        }
    }
    async fn chan_close_init_execute(&mut self, msg: &MsgChannelCloseInit) {
        if let Some(handler) = self.handlers.get_mut(&msg.port_id) {
            handler.chan_close_init_execute(ctx, msg).await;
        }
    }
    async fn recv_packet_execute(&mut self, msg: &MsgRecvPacket) {
        if let Some(handler) = self.handlers.get_mut(&msg.packet.destination_port) {
            handler.recv_packet_execute(ctx, msg).await;
        }
    }
    async fn timeout_packet_execute(&mut self, msg: &MsgTimeout) {
        if let Some(handler) = self.handlers.get_mut(&msg.packet.destination_port) {
            handler.timeout_packet_execute(ctx, msg).await;
        }
    }
    async fn acknowledge_packet_execute(&mut self, msg: &MsgAcknowledgement) {
        if let Some(handler) = self.handlers.get_mut(&msg.packet.destination_port) {
            handler.acknowledge_packet_execute(ctx, msg).await;
        }
    }
}

impl AppHandler for AppRouter {}
