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
use ibc_types::core::channel::msgs::{
    MsgAcknowledgement, MsgChannelCloseConfirm, MsgChannelCloseInit, MsgChannelOpenAck,
    MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry, MsgRecvPacket, MsgTimeout,
};
use penumbra_storage::{StateRead, StateWrite};

/// AppHandlerCheck defines the interface for an IBC application to consume IBC channel and packet
/// events, and apply their validation logic. This validation logic is used for stateful validation
/// only.
#[async_trait]
pub trait AppHandlerCheck: Send + Sync {
    async fn chan_open_init_check<S: StateRead>(state: S, msg: &MsgChannelOpenInit) -> Result<()>;
    async fn chan_open_try_check<S: StateRead>(state: S, msg: &MsgChannelOpenTry) -> Result<()>;
    async fn chan_open_ack_check<S: StateRead>(state: S, msg: &MsgChannelOpenAck) -> Result<()>;
    async fn chan_open_confirm_check<S: StateRead>(
        state: S,
        msg: &MsgChannelOpenConfirm,
    ) -> Result<()>;
    async fn chan_close_confirm_check<S: StateRead>(
        state: S,
        msg: &MsgChannelCloseConfirm,
    ) -> Result<()>;
    async fn chan_close_init_check<S: StateRead>(state: S, msg: &MsgChannelCloseInit)
        -> Result<()>;

    async fn recv_packet_check<S: StateRead>(state: S, msg: &MsgRecvPacket) -> Result<()>;
    async fn timeout_packet_check<S: StateRead>(state: S, msg: &MsgTimeout) -> Result<()>;
    async fn acknowledge_packet_check<S: StateRead>(
        state: S,
        msg: &MsgAcknowledgement,
    ) -> Result<()>;
}

// AppHandlerExecute defines the interface for an IBC application to consume IBC channel and packet
// events and apply their state transition logic. The IBC component will only call these methods
// once the transaction has been validated using the AppHandlerCheck interface.
#[async_trait]
pub trait AppHandlerExecute: Send + Sync {
    async fn chan_open_init_execute<S: StateWrite>(state: S, msg: &MsgChannelOpenInit);
    async fn chan_open_try_execute<S: StateWrite>(state: S, msg: &MsgChannelOpenTry);
    async fn chan_open_ack_execute<S: StateWrite>(state: S, msg: &MsgChannelOpenAck);
    async fn chan_open_confirm_execute<S: StateWrite>(state: S, msg: &MsgChannelOpenConfirm);
    async fn chan_close_confirm_execute<S: StateWrite>(state: S, msg: &MsgChannelCloseConfirm);
    async fn chan_close_init_execute<S: StateWrite>(state: S, msg: &MsgChannelCloseInit);

    async fn recv_packet_execute<S: StateWrite>(state: S, msg: &MsgRecvPacket);
    async fn timeout_packet_execute<S: StateWrite>(state: S, msg: &MsgTimeout);
    async fn acknowledge_packet_execute<S: StateWrite>(state: S, msg: &MsgAcknowledgement);
}

pub trait AppHandler: AppHandlerCheck + AppHandlerExecute {}
