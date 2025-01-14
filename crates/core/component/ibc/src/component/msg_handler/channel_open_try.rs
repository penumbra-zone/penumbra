use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use ibc_types::core::{
    channel::{
        channel::State as ChannelState, events, msgs::MsgChannelOpenTry, ChannelEnd, Counterparty,
        PortId,
    },
    connection::{ConnectionEnd, State as ConnectionState},
};

use crate::component::{
    app_handler::{AppHandlerCheck, AppHandlerExecute},
    channel::StateWriteExt,
    connection::StateReadExt,
    proof_verification::ChannelProofVerifier,
    HostInterface, MsgHandler,
};

#[async_trait]
impl MsgHandler for MsgChannelOpenTry {
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
        let connection_on_b = verify_connections_open(&state, self).await?;

        // TODO: do we want to do capability authentication?
        // TODO: version intersection

        let expected_channel_on_a = ChannelEnd {
            state: ChannelState::Init,
            ordering: self.ordering,
            remote: Counterparty::new(self.port_id_on_b.clone(), None),
            connection_hops: vec![connection_on_b
                .counterparty
                .connection_id
                .clone()
                .ok_or_else(|| anyhow::anyhow!("no counterparty connection id provided"))?],
            version: self.version_supported_on_a.clone(),
            ..ChannelEnd::default()
        };

        tracing::debug!(?self, ?expected_channel_on_a);

        state
            .verify_channel_proof(
                &connection_on_b,
                &self.proof_chan_end_on_a,
                &self.proof_height_on_a,
                &self.chan_id_on_a,
                &self.port_id_on_a,
                &expected_channel_on_a,
            )
            .await?;

        let transfer = PortId::transfer();
        if self.port_id_on_b == transfer {
            AH::chan_open_try_check(&mut state, self).await?;
        } else {
            anyhow::bail!("invalid port id");
        }

        let channel_id = state
            .next_channel_id()
            .await
            .context("unable to retrieve next channel id")?;
        let new_channel = ChannelEnd {
            state: ChannelState::TryOpen,
            ordering: self.ordering,
            remote: Counterparty::new(self.port_id_on_a.clone(), Some(self.chan_id_on_a.clone())),
            connection_hops: self.connection_hops_on_b.clone(),
            version: self.version_supported_on_a.clone(),
            ..ChannelEnd::default()
        };

        state.put_channel(&channel_id, &self.port_id_on_b, new_channel.clone());
        state.put_send_sequence(&channel_id, &self.port_id_on_b, 1);
        state.put_recv_sequence(&channel_id, &self.port_id_on_b, 1);
        state.put_ack_sequence(&channel_id, &self.port_id_on_b, 1);

        state.record(
            events::channel::OpenTry {
                port_id: self.port_id_on_b.clone(),
                channel_id: channel_id.clone(),
                counterparty_port_id: new_channel.counterparty().port_id().clone(),
                counterparty_channel_id: new_channel
                    .counterparty()
                    .channel_id
                    .clone()
                    .unwrap_or_default(),
                connection_id: new_channel.connection_hops[0].clone(),
                version: new_channel.version.clone(),
            }
            .into(),
        );

        let transfer = PortId::transfer();
        if self.port_id_on_b == transfer {
            AH::chan_open_try_execute(state, self).await;
        } else {
            anyhow::bail!("invalid port id");
        }

        Ok(())
    }
}

pub fn connection_hops_eq_1(msg: &MsgChannelOpenTry) -> anyhow::Result<()> {
    if msg.connection_hops_on_b.len() != 1 {
        anyhow::bail!("currently only channels with one connection hop are supported");
    }
    Ok(())
}

async fn verify_connections_open<S: StateRead>(
    state: S,
    msg: &MsgChannelOpenTry,
) -> anyhow::Result<ConnectionEnd> {
    let connection = state
        .get_connection(&msg.connection_hops_on_b[0])
        .await?
        .ok_or_else(|| anyhow::anyhow!("connection not found"))?;

    if connection.state != ConnectionState::Open {
        Err(anyhow::anyhow!("connection for channel is not open"))
    } else {
        Ok(connection)
    }
}
