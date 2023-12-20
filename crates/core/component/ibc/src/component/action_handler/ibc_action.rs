use std::sync::Arc;

use anyhow::{Context, Result};
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::ChainStateReadExt;

use crate::{
    component::{app_handler::AppHandler, MsgHandler as _},
    IbcActionWithHandler, IbcRelay,
};

impl<H: AppHandler> IbcActionWithHandler<H> {
    pub async fn check_stateless(&self, _context: ()) -> Result<()> {
        let action = self.action();
        match action {
            IbcRelay::CreateClient(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::UpdateClient(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::UpgradeClient(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::SubmitMisbehavior(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::ConnectionOpenInit(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::ConnectionOpenTry(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::ConnectionOpenAck(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::ConnectionOpenConfirm(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::ChannelOpenInit(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::ChannelOpenTry(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::ChannelOpenAck(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::ChannelOpenConfirm(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::ChannelCloseInit(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::ChannelCloseConfirm(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::RecvPacket(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::Acknowledgement(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::Timeout(msg) => msg.check_stateless::<H>().await?,
            IbcRelay::Unknown(msg) => {
                anyhow::bail!("unknown IBC message type: {}", msg.type_url)
            }
        }

        Ok(())
    }

    pub async fn check_stateful<_S: StateRead + 'static>(&self, _state: Arc<_S>) -> Result<()> {
        // No-op: IBC actions merge check_stateful and execute.
        Ok(())
    }

    pub async fn execute<S: StateWrite + ChainStateReadExt>(&self, state: S) -> Result<()> {
        let action = self.action();
        match action {
            IbcRelay::CreateClient(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgCreateClient")?,
            IbcRelay::UpdateClient(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgUpdateClient")?,
            IbcRelay::UpgradeClient(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgUpgradeClient")?,
            IbcRelay::SubmitMisbehavior(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgSubmitMisbehaviour")?,
            IbcRelay::ConnectionOpenInit(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgConnectionOpenInit")?,
            IbcRelay::ConnectionOpenTry(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgConnectionOpenTry")?,
            IbcRelay::ConnectionOpenAck(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgConnectionOpenAck")?,
            IbcRelay::ConnectionOpenConfirm(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgConnectionOpenConfirm")?,
            IbcRelay::ChannelOpenInit(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgChannelOpenInit")?,
            IbcRelay::ChannelOpenTry(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgChannelOpenTry")?,
            IbcRelay::ChannelOpenAck(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgChannelOpenAck")?,
            IbcRelay::ChannelOpenConfirm(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgChannelOpenConfirm")?,
            IbcRelay::ChannelCloseInit(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgChannelCloseInit")?,
            IbcRelay::ChannelCloseConfirm(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgChannelCloseConfirm")?,
            IbcRelay::RecvPacket(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgRecvPacket")?,
            IbcRelay::Acknowledgement(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgAcknowledgement")?,
            IbcRelay::Timeout(msg) => msg
                .try_execute::<S, H>(state)
                .await
                .context("failed to execute MsgTimeout")?,
            IbcRelay::Unknown(msg) => {
                anyhow::bail!("unknown IBC message type: {}", msg.type_url)
            }
        }

        Ok(())
    }
}
