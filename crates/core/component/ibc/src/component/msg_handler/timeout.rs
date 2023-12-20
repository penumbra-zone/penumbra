use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ChainStateReadExt;
use ibc_types::core::channel::{
    channel::{Order as ChannelOrder, State as ChannelState},
    events,
    msgs::MsgTimeout,
    PortId,
};

use crate::component::{
    app_handler::{AppHandlerCheck, AppHandlerExecute},
    channel::{StateReadExt as _, StateWriteExt},
    client::StateReadExt,
    connection::StateReadExt as _,
    proof_verification::{commit_packet, PacketProofVerifier},
    MsgHandler,
};

#[async_trait]
impl MsgHandler for MsgTimeout {
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
        let mut channel = state
            .get_channel(&self.packet.chan_on_a, &self.packet.port_on_a)
            .await?
            .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
        if !channel.state_matches(&ChannelState::Open) {
            anyhow::bail!("channel is not open");
        }

        // TODO: capability authentication?
        if self.packet.chan_on_b.ne(channel
            .counterparty()
            .channel_id()
            .ok_or_else(|| anyhow::anyhow!("missing channel id"))?)
        {
            anyhow::bail!("packet destination channel does not match channel");
        }
        if self.packet.port_on_b != channel.counterparty().port_id {
            anyhow::bail!("packet destination port does not match channel");
        }

        let connection = state
            .get_connection(&channel.connection_hops[0])
            .await?
            .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;

        let chain_ts = state
            .get_client_update_time(&connection.client_id, &self.proof_height_on_b)
            .await?;
        let chain_height = self.proof_height_on_b;

        // check that timeout height or timeout timestamp has passed on the other end
        if !self.packet.timed_out(&chain_ts, chain_height) {
            anyhow::bail!("packet has not timed out on the counterparty chain");
        }

        // verify that we actually sent this packet
        let commitment = state
            .get_packet_commitment(&self.packet)
            .await?
            .ok_or_else(|| anyhow::anyhow!("packet commitment not found"))?;
        if commitment != commit_packet(&self.packet) {
            anyhow::bail!("packet commitment does not match");
        }

        if channel.ordering == ChannelOrder::Ordered {
            // ordered channel: check that packet has not been received
            if self.next_seq_recv_on_b != self.packet.sequence {
                anyhow::bail!("packet sequence number does not match");
            }

            // in the case of a timed-out ordered packet, the counterparty should have
            // committed the next sequence number to their state
            state.verify_packet_timeout_proof(&connection, self).await?;
        } else {
            // in the case of a timed-out unordered packet, the counterparty should not have
            // committed a receipt to the state.
            state
                .verify_packet_timeout_absence_proof(&connection, self)
                .await?;
        }

        let transfer = PortId::transfer();
        if self.packet.port_on_b == transfer {
            H::timeout_packet_check(&mut state, self).await?;
        } else {
            anyhow::bail!("invalid port id");
        }

        state.delete_packet_commitment(
            &self.packet.chan_on_a,
            &self.packet.port_on_a,
            self.packet.sequence.into(),
        );

        if channel.ordering == ChannelOrder::Ordered {
            // if the channel is ordered and we get a timeout packet, close the channel
            channel.set_state(ChannelState::Closed);
            state.put_channel(
                &self.packet.chan_on_a,
                &self.packet.port_on_a,
                channel.clone(),
            );
        }

        state.record(
            events::packet::TimeoutPacket {
                timeout_height: self.packet.timeout_height_on_b,
                timeout_timestamp: self.packet.timeout_timestamp_on_b,
                sequence: self.packet.sequence,
                src_port_id: self.packet.port_on_a.clone(),
                src_channel_id: self.packet.chan_on_a.clone(),
                dst_port_id: self.packet.port_on_b.clone(),
                dst_channel_id: self.packet.chan_on_b.clone(),
                channel_ordering: channel.ordering,
            }
            .into(),
        );

        let transfer = PortId::transfer();
        if self.packet.port_on_b == transfer {
            H::timeout_packet_execute(state, self).await;
        } else {
            anyhow::bail!("invalid port id");
        }

        Ok(())
    }
}
