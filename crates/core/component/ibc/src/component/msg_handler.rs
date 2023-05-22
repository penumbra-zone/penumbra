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
mod recv_packet;
mod timeout;
mod update_client;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::StateWrite;

/// Variant of ActionHandler defined locally (so it can be implemented for IBC
/// message types) and tweaked (removing the separate check_stateless step).
#[async_trait]
pub(crate) trait MsgHandler {
    async fn check_stateless(&self) -> Result<()>;
    async fn try_execute<S: StateWrite>(&self, state: S) -> Result<()>;
}
