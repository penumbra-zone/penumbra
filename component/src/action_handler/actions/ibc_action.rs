use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::IbcAction, Transaction};

use crate::action_handler::ActionHandler;

mod msg;

#[async_trait]
impl ActionHandler for IbcAction {
    async fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        match self {
            IbcAction::CreateClient(msg) => msg.check_stateless(context).await?,
            IbcAction::UpdateClient(msg) => msg.check_stateless(context).await?,
            IbcAction::ConnectionOpenInit(msg) => msg.check_stateless(context).await?,
            IbcAction::ConnectionOpenTry(msg) => msg.check_stateless(context).await?,
            IbcAction::ConnectionOpenAck(msg) => msg.check_stateless(context).await?,
            IbcAction::ConnectionOpenConfirm(msg) => msg.check_stateless(context).await?,
            IbcAction::ChannelOpenInit(msg) => msg.check_stateless(context).await?,
            IbcAction::ChannelOpenTry(msg) => msg.check_stateless(context).await?,
            IbcAction::ChannelOpenAck(msg) => msg.check_stateless(context).await?,
            IbcAction::ChannelOpenConfirm(msg) => msg.check_stateless(context).await?,
            IbcAction::ChannelCloseInit(msg) => msg.check_stateless(context).await?,
            IbcAction::ChannelCloseConfirm(msg) => msg.check_stateless(context).await?,
            IbcAction::RecvPacket(msg) => msg.check_stateless(context).await?,
            IbcAction::Acknowledgement(msg) => msg.check_stateless(context).await?,
            IbcAction::Timeout(msg) => msg.check_stateless(context).await?,
            IbcAction::Unknown(msg) => {
                return Err(anyhow::anyhow!(
                    "unknown IBC message type: {}",
                    msg.type_url
                ))
            }
        }

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        match self {
            IbcAction::CreateClient(msg) => msg.check_stateful(state).await?,
            IbcAction::UpdateClient(msg) => msg.check_stateful(state).await?,
            IbcAction::ConnectionOpenInit(msg) => msg.check_stateful(state).await?,
            IbcAction::ConnectionOpenTry(msg) => msg.check_stateful(state).await?,
            IbcAction::ConnectionOpenAck(msg) => msg.check_stateful(state).await?,
            IbcAction::ConnectionOpenConfirm(msg) => msg.check_stateful(state).await?,
            IbcAction::ChannelOpenInit(msg) => msg.check_stateful(state).await?,
            IbcAction::ChannelOpenTry(msg) => msg.check_stateful(state).await?,
            IbcAction::ChannelOpenAck(msg) => msg.check_stateful(state).await?,
            IbcAction::ChannelOpenConfirm(msg) => msg.check_stateful(state).await?,
            IbcAction::ChannelCloseInit(msg) => msg.check_stateful(state).await?,
            IbcAction::ChannelCloseConfirm(msg) => msg.check_stateful(state).await?,
            IbcAction::RecvPacket(msg) => msg.check_stateful(state).await?,
            IbcAction::Acknowledgement(msg) => msg.check_stateful(state).await?,
            IbcAction::Timeout(msg) => msg.check_stateful(state).await?,
            IbcAction::Unknown(msg) => {
                return Err(anyhow::anyhow!(
                    "unknown IBC message type: {}",
                    msg.type_url
                ))
            }
        }

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
