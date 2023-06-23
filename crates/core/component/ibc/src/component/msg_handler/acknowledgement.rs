use anyhow::Result;
use async_trait::async_trait;
use ibc_types2::core::{
    channel::channel::Order as ChannelOrder, channel::channel::State as ChannelState,
    channel::msgs::MsgAcknowledgement, channel::PortId, connection::State as ConnectionState,
};
use penumbra_storage::StateWrite;

use crate::component::{
    app_handler::{AppHandlerCheck, AppHandlerExecute},
    channel::{StateReadExt as _, StateWriteExt as _},
    connection::StateReadExt as _,
    proof_verification::{commit_packet, PacketProofVerifier},
    transfer::Ics20Transfer,
    MsgHandler,
};

use crate::event;

#[async_trait]
impl MsgHandler for MsgAcknowledgement {
    async fn check_stateless(&self) -> Result<()> {
        // NOTE: no additional stateless validation is possible

        Ok(())
    }

    async fn try_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        tracing::debug!(msg = ?self);
        let channel = state
            .get_channel(&self.packet.chan_on_a, &self.packet.port_on_a)
            .await?
            .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
        if !channel.state_matches(&ChannelState::Open) {
            return Err(anyhow::anyhow!("channel is not open"));
        }

        // TODO: capability authentication?

        if channel.counterparty().port_id().ne(&self.packet.port_on_b) {
            return Err(anyhow::anyhow!(
                "packet destination port does not match channel"
            ));
        }

        let connection = state
            .get_connection(&channel.connection_hops[0])
            .await?
            .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;
        if !connection.state_matches(&ConnectionState::Open) {
            return Err(anyhow::anyhow!("connection for channel is not open"));
        }

        // verify we sent this packet and haven't cleared it yet
        let commitment = state
            .get_packet_commitment(&self.packet)
            .await?
            .ok_or_else(|| anyhow::anyhow!("packet commitment not found"))?;
        if commitment != commit_packet(&self.packet) {
            return Err(anyhow::anyhow!("packet commitment does not match"));
        }

        state.verify_packet_ack_proof(&connection, self).await?;

        if channel.ordering == ChannelOrder::Ordered {
            let next_sequence_ack = state
                .get_ack_sequence(&self.packet.chan_on_a, &self.packet.port_on_a)
                .await?;
            if self.packet.sequence != next_sequence_ack.into() {
                return Err(anyhow::anyhow!("packet sequence number does not match"));
            }
        }

        let transfer = PortId::transfer();
        if self.packet.port_on_b == transfer {
            Ics20Transfer::acknowledge_packet_check(&mut state, self).await?;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
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

        state.record(event::acknowledge_packet(&self.packet, &channel));

        let transfer = PortId::transfer();
        if self.packet.port_on_b == transfer {
            Ics20Transfer::acknowledge_packet_execute(state, self).await;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
        }

        Ok(())
    }
}
