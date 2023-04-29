use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_component::ActionHandler;
use penumbra_storage::{StateRead, StateWrite};

use crate::{component::ActionHandler as LocalActionHandler, IbcAction};

#[async_trait]
impl ActionHandler for IbcAction {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        match self {
            IbcAction::CreateClient(msg) => msg.check_stateless(()).await?,
            IbcAction::UpdateClient(msg) => msg.check_stateless(()).await?,
            IbcAction::ConnectionOpenInit(msg) => msg.check_stateless(()).await?,
            IbcAction::ConnectionOpenTry(msg) => msg.check_stateless(()).await?,
            IbcAction::ConnectionOpenAck(msg) => msg.check_stateless(()).await?,
            IbcAction::ConnectionOpenConfirm(msg) => msg.check_stateless(()).await?,
            IbcAction::ChannelOpenInit(msg) => msg.check_stateless(()).await?,
            IbcAction::ChannelOpenTry(msg) => msg.check_stateless(()).await?,
            IbcAction::ChannelOpenAck(msg) => msg.check_stateless(()).await?,
            IbcAction::ChannelOpenConfirm(msg) => msg.check_stateless(()).await?,
            IbcAction::ChannelCloseInit(msg) => msg.check_stateless(()).await?,
            IbcAction::ChannelCloseConfirm(msg) => msg.check_stateless(()).await?,
            IbcAction::RecvPacket(msg) => msg.check_stateless(()).await?,
            IbcAction::Acknowledgement(msg) => msg.check_stateless(()).await?,
            IbcAction::Timeout(msg) => msg.check_stateless(()).await?,
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
            IbcAction::CreateClient(msg) => msg.execute(state).await?,
            IbcAction::UpdateClient(msg) => msg.execute(state).await?,
            IbcAction::ConnectionOpenInit(msg) => msg.execute(state).await?,
            IbcAction::ConnectionOpenTry(msg) => msg.execute(state).await?,
            IbcAction::ConnectionOpenAck(msg) => msg.execute(state).await?,
            IbcAction::ConnectionOpenConfirm(msg) => msg.execute(state).await?,
            IbcAction::ChannelOpenInit(msg) => msg.execute(state).await?,
            IbcAction::ChannelOpenTry(msg) => msg.execute(state).await?,
            IbcAction::ChannelOpenAck(msg) => msg.execute(state).await?,
            IbcAction::ChannelOpenConfirm(msg) => msg.execute(state).await?,
            IbcAction::ChannelCloseInit(msg) => msg.execute(state).await?,
            IbcAction::ChannelCloseConfirm(msg) => msg.execute(state).await?,
            IbcAction::RecvPacket(msg) => msg.execute(state).await?,
            IbcAction::Acknowledgement(msg) => msg.execute(state).await?,
            IbcAction::Timeout(msg) => msg.execute(state).await?,
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
