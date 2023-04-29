use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc_types::core::ics03_connection::connection::State as ConnectionState;
use ibc_types::core::ics04_channel::channel::Order as ChannelOrder;
use ibc_types::core::ics04_channel::channel::State as ChannelState;
use ibc_types::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
use ibc_types::core::ics24_host::identifier::PortId;
use penumbra_storage::{StateRead, StateWrite};

use crate::action_handler::ActionHandler;
use crate::ibc::component::channel::stateful::proof_verification::{
    commit_packet, PacketProofVerifier,
};
use crate::ibc::component::channel::StateReadExt as _;
use crate::ibc::component::channel::StateWriteExt as _;
use crate::ibc::component::connection::StateReadExt as _;
use crate::ibc::event;
use crate::ibc::ibc_handler::{AppHandlerCheck, AppHandlerExecute};
use crate::ibc::transfer::Ics20Transfer;

#[async_trait]
impl ActionHandler for MsgAcknowledgement {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // NOTE: no additional stateless validation is possible

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        // No-op: IBC actions merge check_stateful and execute.
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
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
