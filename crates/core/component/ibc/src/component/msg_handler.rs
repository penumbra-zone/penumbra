mod acknowledgement;
mod channel_close_confirm;
mod channel_close_init;
mod channel_open_ack;
mod channel_open_confirm;
mod channel_open_init;
mod channel_open_try;
mod connection_open_ack;
mod connection_open_confirm;
mod connection_open_init;
mod connection_open_try;
mod create_client;
mod misbehavior;
mod recv_packet;
mod timeout;
mod update_client;
mod upgrade_client;

use crate::component::app_handler::{AppHandlerCheck, AppHandlerExecute};
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;

use super::HostInterface;

/// Variant of ActionHandler defined locally (so it can be implemented for IBC
/// message types) and tweaked (removing the separate check_stateless step).
#[async_trait]
pub(crate) trait MsgHandler {
    async fn check_stateless<AH: AppHandlerCheck>(&self) -> Result<()>;
    async fn try_execute<
        S: StateWrite,
        AH: AppHandlerCheck + AppHandlerExecute,
        HI: HostInterface,
    >(
        &self,
        state: S,
    ) -> Result<()>;
}
