use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_component::ActionHandler;
use penumbra_storage::{StateRead, StateWrite};

use crate::{
    component::{app_handler::AppHandler, MsgHandler as _},
    IbcAction, IbcActionWithHandler,
};

#[async_trait]
impl<H: AppHandler> ActionHandler for IbcActionWithHandler<H> {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        let action = self.action();
        match action {
            IbcAction::CreateClient(msg) => msg.check_stateless::<H>().await?,
            IbcAction::UpdateClient(msg) => msg.check_stateless::<H>().await?,
            IbcAction::UpgradeClient(msg) => msg.check_stateless::<H>().await?,
            IbcAction::SubmitMisbehavior(msg) => msg.check_stateless::<H>().await?,
            IbcAction::ConnectionOpenInit(msg) => msg.check_stateless::<H>().await?,
            IbcAction::ConnectionOpenTry(msg) => msg.check_stateless::<H>().await?,
            IbcAction::ConnectionOpenAck(msg) => msg.check_stateless::<H>().await?,
            IbcAction::ConnectionOpenConfirm(msg) => msg.check_stateless::<H>().await?,
            IbcAction::ChannelOpenInit(msg) => msg.check_stateless::<H>().await?,
            IbcAction::ChannelOpenTry(msg) => msg.check_stateless::<H>().await?,
            IbcAction::ChannelOpenAck(msg) => msg.check_stateless::<H>().await?,
            IbcAction::ChannelOpenConfirm(msg) => msg.check_stateless::<H>().await?,
            IbcAction::ChannelCloseInit(msg) => msg.check_stateless::<H>().await?,
            IbcAction::ChannelCloseConfirm(msg) => msg.check_stateless::<H>().await?,
            IbcAction::RecvPacket(msg) => msg.check_stateless::<H>().await?,
            IbcAction::Acknowledgement(msg) => msg.check_stateless::<H>().await?,
            IbcAction::Timeout(msg) => msg.check_stateless::<H>().await?,
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
        let action = self.action();
        match action {
            IbcAction::CreateClient(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgCreateClient")?,
            IbcAction::UpdateClient(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgUpdateClient")?,
            IbcAction::UpgradeClient(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgUpgradeClient")?,
            IbcAction::SubmitMisbehavior(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgSubmitMisbehaviour")?,
            IbcAction::ConnectionOpenInit(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgConnectionOpenInit")?,
            IbcAction::ConnectionOpenTry(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgConnectionOpenTry")?,
            IbcAction::ConnectionOpenAck(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgConnectionOpenAck")?,
            IbcAction::ConnectionOpenConfirm(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgConnectionOpenConfirm")?,
            IbcAction::ChannelOpenInit(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgChannelOpenInit")?,
            IbcAction::ChannelOpenTry(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgChannelOpenTry")?,
            IbcAction::ChannelOpenAck(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgChannelOpenAck")?,
            IbcAction::ChannelOpenConfirm(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgChannelOpenConfirm")?,
            IbcAction::ChannelCloseInit(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgChannelCloseInit")?,
            IbcAction::ChannelCloseConfirm(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgChannelCloseConfirm")?,
            IbcAction::RecvPacket(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgRecvPacket")?,
            IbcAction::Acknowledgement(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgAcknowledgement")?,
            IbcAction::Timeout(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgTimeout")?,
            IbcAction::Unknown(msg) => {
                anyhow::bail!("unknown IBC message type: {}", msg.type_url)
            }
        }

        Ok(())
    }
}
