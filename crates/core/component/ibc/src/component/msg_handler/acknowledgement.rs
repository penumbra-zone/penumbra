use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use ibc_types::core::{
    channel::channel::Order as ChannelOrder, channel::channel::State as ChannelState,
    channel::events, channel::msgs::MsgAcknowledgement, channel::PortId,
    connection::State as ConnectionState,
};

use crate::component::{
    app_handler::{AppHandlerCheck, AppHandlerExecute},
    channel::{StateReadExt as _, StateWriteExt as _},
    connection::StateReadExt as _,
    proof_verification::{commit_packet, PacketProofVerifier},
    MsgHandler,
};
use cnidarium_component::ChainStateReadExt;

#[async_trait]
impl MsgHandler for MsgAcknowledgement {
    async fn check_stateless<H: AppHandlerCheck>(&self) -> Result<()> {
        // NOTE: no additional stateless validation is possible

        Ok(())
    }

    async fn try_execute<
        S: StateWrite + ChainStateReadExt,
        H: AppHandlerCheck + AppHandlerExecute,
    >(
        &self,
        mut state: S,
    ) -> Result<()> {
        tracing::debug!(msg = ?self);
        let channel = state
            .get_channel(&self.packet.chan_on_a, &self.packet.port_on_a)
            .await?
            .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
        if !channel.state_matches(&ChannelState::Open) {
            anyhow::bail!("channel is not open");
        }

        // TODO: capability authentication?

        if channel.counterparty().port_id().ne(&self.packet.port_on_b) {
            anyhow::bail!("packet destination port does not match channel");
        }

        let connection = state
            .get_connection(&channel.connection_hops[0])
            .await?
            .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;
        if !connection.state_matches(&ConnectionState::Open) {
            anyhow::bail!("connection for channel is not open");
        }

        // verify we sent this packet and haven't cleared it yet
        let commitment = state
            .get_packet_commitment(&self.packet)
            .await?
            .ok_or_else(|| anyhow::anyhow!("packet commitment not found"))?;
        if commitment != commit_packet(&self.packet) {
            anyhow::bail!("packet commitment does not match");
        }

        state
            .verify_packet_ack_proof(&connection, self)
            .await
            .with_context(|| "packet ack proof verification failed")?;

        if channel.ordering == ChannelOrder::Ordered {
            let next_sequence_ack = state
                .get_ack_sequence(&self.packet.chan_on_a, &self.packet.port_on_a)
                .await?;
            if self.packet.sequence != next_sequence_ack.into() {
                anyhow::bail!("packet sequence number does not match");
            }
        }

        let transfer = PortId::transfer();
        if self.packet.port_on_b == transfer {
            H::acknowledge_packet_check(&mut state, self).await?;
        } else {
            anyhow::bail!("invalid port id");
        }
        if channel.ordering == ChannelOrder::Ordered {
            let mut next_sequence_ack = state
                .get_ack_sequence(&self.packet.chan_on_a, &self.packet.port_on_a)
                .await?;
            next_sequence_ack += 1;
            state.put_ack_sequence(
                &self.packet.chan_on_a,
                &self.packet.port_on_a,
                next_sequence_ack,
            );
        }

        // delete our commitment so we can't ack it again
        state.delete_packet_commitment(
            &self.packet.chan_on_a,
            &self.packet.port_on_a,
            self.packet.sequence.into(),
        );

        state.record(
            events::packet::AcknowledgePacket {
                timeout_height: self.packet.timeout_height_on_b,
                timeout_timestamp: self.packet.timeout_timestamp_on_b,
                sequence: self.packet.sequence,
                src_port_id: self.packet.port_on_a.clone(),
                src_channel_id: self.packet.chan_on_a.clone(),
                dst_port_id: self.packet.port_on_b.clone(),
                dst_channel_id: self.packet.chan_on_b.clone(),
                channel_ordering: channel.ordering,
                src_connection_id: channel.connection_hops[0].clone(),
            }
            .into(),
        );

        let transfer = PortId::transfer();
        if self.packet.port_on_b == transfer {
            H::acknowledge_packet_execute(state, self).await;
        } else {
            anyhow::bail!("invalid port id");
        }

        Ok(())
    }
}
