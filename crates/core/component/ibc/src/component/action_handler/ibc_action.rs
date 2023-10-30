use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_component::ActionHandler;
use penumbra_storage::{StateRead, StateWrite};

use ibc_types::core::{channel::msgs::*, client::msgs::*, connection::msgs::*};

use crate::{
    component::{app_handler::AppHandler, MsgHandler as LocalActionHandler},
    IbcAction, IbcActionWithHandler,
};

#[async_trait]
impl<H: AppHandler> ActionHandler for IbcActionWithHandler<H> {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        let action = self.action();
        match action {
            IbcAction::CreateClient(msg) => {
                <MsgCreateClient as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::UpdateClient(msg) => {
                <MsgUpdateClient as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::UpgradeClient(msg) => {
                <MsgUpgradeClient as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::SubmitMisbehavior(msg) => {
                <MsgSubmitMisbehaviour as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::ConnectionOpenInit(msg) => {
                <MsgConnectionOpenInit as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::ConnectionOpenTry(msg) => {
                <MsgConnectionOpenTry as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::ConnectionOpenAck(msg) => {
                <MsgConnectionOpenAck as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::ConnectionOpenConfirm(msg) => {
                <MsgConnectionOpenConfirm as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::ChannelOpenInit(msg) => {
                <MsgChannelOpenInit as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::ChannelOpenTry(msg) => {
                <MsgChannelOpenTry as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::ChannelOpenAck(msg) => {
                <MsgChannelOpenAck as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::ChannelOpenConfirm(msg) => {
                <MsgChannelOpenConfirm as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::ChannelCloseInit(msg) => {
                <MsgChannelCloseInit as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::ChannelCloseConfirm(msg) => {
                <MsgChannelCloseConfirm as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::RecvPacket(msg) => {
                <MsgRecvPacket as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::Acknowledgement(msg) => {
                <MsgAcknowledgement as LocalActionHandler<H>>::check_stateless(msg).await?
            }
            IbcAction::Timeout(msg) => {
                <MsgTimeout as LocalActionHandler<H>>::check_stateless(msg).await?
            }
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
            IbcAction::CreateClient(msg) => {
                <MsgCreateClient as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgCreateClient")?
            }
            IbcAction::UpdateClient(msg) => {
                <MsgUpdateClient as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgUpdateClient")?
            }
            IbcAction::UpgradeClient(msg) => {
                <MsgUpgradeClient as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgUpgradeClient")?
            }
            IbcAction::SubmitMisbehavior(msg) => {
                <MsgSubmitMisbehaviour as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgSubmitMisbehaviour")?
            }
            IbcAction::ConnectionOpenInit(msg) => {
                <MsgConnectionOpenInit as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgConnectionOpenInit")?
            }
            IbcAction::ConnectionOpenTry(msg) => {
                <MsgConnectionOpenTry as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgConnectionOpenTry")?
            }
            IbcAction::ConnectionOpenAck(msg) => {
                <MsgConnectionOpenAck as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgConnectionOpenAck")?
            }
            IbcAction::ConnectionOpenConfirm(msg) => {
                <MsgConnectionOpenConfirm as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgConnectionOpenConfirm")?
            }
            IbcAction::ChannelOpenInit(msg) => {
                <MsgChannelOpenInit as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgChannelOpenInit")?
            }
            IbcAction::ChannelOpenTry(msg) => {
                <MsgChannelOpenTry as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgChannelOpenTry")?
            }
            IbcAction::ChannelOpenAck(msg) => {
                <MsgChannelOpenAck as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgChannelOpenAck")?
            }
            IbcAction::ChannelOpenConfirm(msg) => {
                <MsgChannelOpenConfirm as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgChannelOpenConfirm")?
            }
            IbcAction::ChannelCloseInit(msg) => {
                <MsgChannelCloseInit as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgChannelCloseInit")?
            }
            IbcAction::ChannelCloseConfirm(msg) => {
                <MsgChannelCloseConfirm as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgChannelCloseConfirm")?
            }
            IbcAction::RecvPacket(msg) => {
                <MsgRecvPacket as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgRecvPacket")?
            }
            IbcAction::Acknowledgement(msg) => {
                <MsgAcknowledgement as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgAcknowledgement")?
            }
            IbcAction::Timeout(msg) => {
                <MsgTimeout as LocalActionHandler<H>>::try_execute(msg, state)
                    .await
                    .context("failed to execute MsgTimeout")?
            }
            IbcAction::Unknown(msg) => {
                anyhow::bail!("unknown IBC message type: {}", msg.type_url)
            }
        }

        Ok(())
    }
}
