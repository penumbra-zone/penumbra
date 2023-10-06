use std::sync::Arc;

use anyhow::{Context, Result};
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
            IbcAction::UpgradeClient(msg) => msg.check_stateless().await?,
            IbcAction::SubmitMisbehavior(msg) => msg.check_stateless().await?,
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
                anyhow::bail!("unknown IBC message type: {}", msg.type_url)
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
            IbcAction::CreateClient(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute CreateClient message")?,
            IbcAction::UpdateClient(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute UpdateClient message")?,
            IbcAction::UpgradeClient(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute UpgradeClient message")?,
            IbcAction::SubmitMisbehavior(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute SubmitMisbehavior message")?,
            IbcAction::ConnectionOpenInit(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute ConnectionOpenInit message")?,
            IbcAction::ConnectionOpenTry(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute ConnectionOpenTry message")?,
            IbcAction::ConnectionOpenAck(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute ConnectionOpenAck message")?,
            IbcAction::ConnectionOpenConfirm(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute ConnectionOpenConfirm message")?,
            IbcAction::ChannelOpenInit(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute ChannelOpenInit message")?,
            IbcAction::ChannelOpenTry(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute ChannelOpenTry message")?,
            IbcAction::ChannelOpenAck(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute ChannelOpenAck message")?,
            IbcAction::ChannelOpenConfirm(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute ChannelOpenConfirm message")?,
            IbcAction::ChannelCloseInit(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute ChannelCloseInit message")?,
            IbcAction::ChannelCloseConfirm(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute ChannelCloseConfirm message")?,
            IbcAction::RecvPacket(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute RecvPacket message")?,
            IbcAction::Acknowledgement(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute Acknowledgement message")?,
            IbcAction::Timeout(msg) => msg
                .try_execute(state)
                .await
                .with_context(|| "Failed to execute Timeout message")?,
            IbcAction::Unknown(msg) => {
                anyhow::bail!("unknown IBC message type: {}", msg.type_url)
            }
        }

        Ok(())
    }
}
