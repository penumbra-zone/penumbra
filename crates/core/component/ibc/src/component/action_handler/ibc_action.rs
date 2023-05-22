use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_component::ActionHandler;
use penumbra_storage::{StateRead, StateWrite};

use crate::{component::MsgHandler as LocalActionHandler, IbcAction};

#[async_trait]
impl ActionHandler for IbcAction {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        match self {
            IbcAction::CreateClient(msg) => msg.check_stateless().await?,
            IbcAction::UpdateClient(msg) => msg.check_stateless().await?,
            IbcAction::ConnectionOpenInit(msg) => msg.check_stateless().await?,
            IbcAction::ConnectionOpenTry(msg) => msg.check_stateless().await?,
            IbcAction::ConnectionOpenAck(msg) => msg.check_stateless().await?,
            IbcAction::ConnectionOpenConfirm(msg) => msg.check_stateless().await?,
            IbcAction::ChannelOpenInit(msg) => msg.check_stateless().await?,
            IbcAction::ChannelOpenTry(msg) => msg.check_stateless().await?,
            IbcAction::ChannelOpenAck(msg) => msg.check_stateless().await?,
            IbcAction::ChannelOpenConfirm(msg) => msg.check_stateless().await?,
            IbcAction::ChannelCloseInit(msg) => msg.check_stateless().await?,
            IbcAction::ChannelCloseConfirm(msg) => msg.check_stateless().await?,
            IbcAction::RecvPacket(msg) => msg.check_stateless().await?,
            IbcAction::Acknowledgement(msg) => msg.check_stateless().await?,
            IbcAction::Timeout(msg) => msg.check_stateless().await?,
            IbcAction::Unknown(msg) => {
                return Err(anyhow::anyhow!(
                    "unknown IBC message type: {}",
                    msg.type_url
                ))
            }
        }

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        // No-op: IBC actions merge check_stateful and execute.
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, state: S) -> Result<()> {
        match self {
            IbcAction::CreateClient(msg) => msg.try_execute(state).await?,
            IbcAction::UpdateClient(msg) => msg.try_execute(state).await?,
            IbcAction::ConnectionOpenInit(msg) => msg.try_execute(state).await?,
            IbcAction::ConnectionOpenTry(msg) => msg.try_execute(state).await?,
            IbcAction::ConnectionOpenAck(msg) => msg.try_execute(state).await?,
            IbcAction::ConnectionOpenConfirm(msg) => msg.try_execute(state).await?,
            IbcAction::ChannelOpenInit(msg) => msg.try_execute(state).await?,
            IbcAction::ChannelOpenTry(msg) => msg.try_execute(state).await?,
            IbcAction::ChannelOpenAck(msg) => msg.try_execute(state).await?,
            IbcAction::ChannelOpenConfirm(msg) => msg.try_execute(state).await?,
            IbcAction::ChannelCloseInit(msg) => msg.try_execute(state).await?,
            IbcAction::ChannelCloseConfirm(msg) => msg.try_execute(state).await?,
            IbcAction::RecvPacket(msg) => msg.try_execute(state).await?,
            IbcAction::Acknowledgement(msg) => msg.try_execute(state).await?,
            IbcAction::Timeout(msg) => msg.try_execute(state).await?,
            IbcAction::Unknown(msg) => {
                return Err(anyhow::anyhow!(
                    "unknown IBC message type: {}",
                    msg.type_url
                ))
            }
        }

        Ok(())
    }
}
