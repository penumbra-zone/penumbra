use std::sync::Arc;

/// IBC "app handlers" for the IBC component.
///
/// An app handler defines an interface for any IBC application to consume verified IBC events from
/// the core IBC component. IBC applications listen to channel events that occur on the Port ID
/// that they have subscribed to, and apply application-specific state transition logic.
///
/// The primary IBC application is the Ics20 transfer application, which allows for interchain
/// token transfers.
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
use penumbra_storage2::{State, StateTransaction};

/// AppHandlerCheck defines the interface for an IBC application to consume IBC channel and packet
/// events, and apply their validation logic. This validation logic is used for stateful validation
/// only.
#[async_trait]
pub trait AppHandlerCheck: Send + Sync {
    async fn chan_open_init_check(state: Arc<State>, msg: &MsgChannelOpenInit) -> Result<()>;
    async fn chan_open_try_check(state: Arc<State>, msg: &MsgChannelOpenTry) -> Result<()>;
    async fn chan_open_ack_check(state: Arc<State>, msg: &MsgChannelOpenAck) -> Result<()>;
    async fn chan_open_confirm_check(state: Arc<State>, msg: &MsgChannelOpenConfirm) -> Result<()>;
    async fn chan_close_confirm_check(
        state: Arc<State>,
        msg: &MsgChannelCloseConfirm,
    ) -> Result<()>;
    async fn chan_close_init_check(state: Arc<State>, msg: &MsgChannelCloseInit) -> Result<()>;

    async fn recv_packet_check(state: Arc<State>, msg: &MsgRecvPacket) -> Result<()>;
    async fn timeout_packet_check(state: Arc<State>, msg: &MsgTimeout) -> Result<()>;
    async fn acknowledge_packet_check(state: Arc<State>, msg: &MsgAcknowledgement) -> Result<()>;
}

// AppHandlerExecute defines the interface for an IBC application to consume IBC channel and packet
// events and apply their state transition logic. The IBC component will only call these methods
// once the transaction has been validated using the AppHandlerCheck interface.
#[async_trait]
pub trait AppHandlerExecute: Send + Sync {
    async fn chan_open_init_execute(state: &mut StateTransaction, msg: &MsgChannelOpenInit);
    async fn chan_open_try_execute(state: &mut StateTransaction, msg: &MsgChannelOpenTry);
    async fn chan_open_ack_execute(state: &mut StateTransaction, msg: &MsgChannelOpenAck);
    async fn chan_open_confirm_execute(state: &mut StateTransaction, msg: &MsgChannelOpenConfirm);
    async fn chan_close_confirm_execute(state: &mut StateTransaction, msg: &MsgChannelCloseConfirm);
    async fn chan_close_init_execute(state: &mut StateTransaction, msg: &MsgChannelCloseInit);

    async fn recv_packet_execute(state: &mut StateTransaction, msg: &MsgRecvPacket);
    async fn timeout_packet_execute(state: &mut StateTransaction, msg: &MsgTimeout);
    async fn acknowledge_packet_execute(state: &mut StateTransaction, msg: &MsgAcknowledgement);
}

pub trait AppHandler: AppHandlerCheck + AppHandlerExecute {}
