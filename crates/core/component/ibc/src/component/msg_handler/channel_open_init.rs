use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use ibc_types::core::channel::msgs::MsgChannelOpenInit;
use ibc_types::core::channel::{
    channel::State, events, ChannelEnd, ChannelId, Counterparty, PortId,
};

use crate::component::HostInterface;
use crate::component::{
    app_handler::{AppHandlerCheck, AppHandlerExecute},
    channel::{StateReadExt as _, StateWriteExt as _},
    connection::StateReadExt as _,
    MsgHandler,
};

#[async_trait]
impl MsgHandler for MsgChannelOpenInit {
    async fn check_stateless<H: AppHandlerCheck>(&self) -> Result<()> {
        connection_hops_eq_1(self)?;

        Ok(())
    }

    async fn try_execute<
        S: StateWrite,
        AH: AppHandlerCheck + AppHandlerExecute,
        HI: HostInterface,
    >(
        &self,
        mut state: S,
    ) -> Result<()> {
        tracing::debug!(msg = ?self);
        let channel_id = get_channel_id(&state).await?;

        verify_channel_does_not_exist(&state, &channel_id, &self.port_id_on_a).await?;

        // NOTE: optimistic channel handshakes are allowed, so we don't check if the connection
        // is open here.
        verify_connections_exist(&state, self).await?;

        // TODO: do we want to do capability authentication?

        let transfer = PortId::transfer();
        if self.port_id_on_a == transfer {
            AH::chan_open_init_check(&mut state, self).await?;
        } else {
            anyhow::bail!("invalid port id");
        }
        let channel_id = state
            .next_channel_id()
            .await
            .context("unable to get next channel id")?;
        let new_channel = ChannelEnd {
            state: State::Init,
            ordering: self.ordering,
            remote: Counterparty::new(self.port_id_on_b.clone(), None),
            connection_hops: self.connection_hops_on_a.clone(),
            version: self.version_proposal.clone(),
            ..ChannelEnd::default()
        };

        state.put_channel(&channel_id, &self.port_id_on_a, new_channel.clone());
        state.put_send_sequence(&channel_id, &self.port_id_on_a, 1);
        state.put_recv_sequence(&channel_id, &self.port_id_on_a, 1);
        state.put_ack_sequence(&channel_id, &self.port_id_on_a, 1);

        state.record(
            events::channel::OpenInit {
                port_id: self.port_id_on_a.clone(),
                channel_id: channel_id.clone(),
                counterparty_port_id: new_channel.counterparty().port_id().clone(),
                connection_id: new_channel.connection_hops[0].clone(),
                version: new_channel.version.clone(),
            }
            .into(),
        );

        let transfer = PortId::transfer();
        if self.port_id_on_a == transfer {
            AH::chan_open_init_execute(state, self).await;
        } else {
            anyhow::bail!("invalid port id");
        }

        Ok(())
    }
}

fn connection_hops_eq_1(msg: &MsgChannelOpenInit) -> anyhow::Result<()> {
    if msg.connection_hops_on_a.len() != 1 {
        anyhow::bail!("currently only channels with one connection hop are supported");
    }
    Ok(())
}
async fn verify_connections_exist<S: StateRead>(
    state: S,
    msg: &MsgChannelOpenInit,
) -> anyhow::Result<()> {
    state
        .get_connection(&msg.connection_hops_on_a[0])
        .await?
        .ok_or_else(|| anyhow::anyhow!("connection not found"))
        .map(|_| ())
}

async fn get_channel_id<S: StateRead>(state: S) -> anyhow::Result<ChannelId> {
    let counter = state.get_channel_counter().await?;

    Ok(ChannelId::new(counter))
}

async fn verify_channel_does_not_exist<S: StateRead>(
    state: S,
    channel_id: &ChannelId,
    port_id: &PortId,
) -> anyhow::Result<()> {
    let channel = state.get_channel(channel_id, port_id).await?;
    if channel.is_some() {
        anyhow::bail!("channel already exists");
    }
    Ok(())
}
