use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc_types::core::ics04_channel::msgs::timeout::MsgTimeout;
use ibc_types::core::ics24_host::identifier::PortId;
use penumbra_storage::{StateRead, StateWrite};

use crate::action_handler::ActionHandler;
use crate::ibc::component::channel::stateful::proof_verification::{
    commit_packet, PacketProofVerifier,
};
use crate::ibc::component::channel::{StateReadExt as _, StateWriteExt};
use crate::ibc::component::client::StateReadExt;
use crate::ibc::component::connection::StateReadExt as _;
use crate::ibc::event;
use crate::ibc::ibc_handler::{AppHandlerCheck, AppHandlerExecute};
use crate::ibc::transfer::Ics20Transfer;
use ibc_types::core::ics04_channel::channel::Order as ChannelOrder;
use ibc_types::core::ics04_channel::channel::State as ChannelState;

#[async_trait]
impl ActionHandler for MsgTimeout {
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
        let mut channel = state
            .get_channel(&self.packet.chan_on_a, &self.packet.port_on_a)
            .await?
            .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
        if !channel.state_matches(&ChannelState::Open) {
            return Err(anyhow::anyhow!("channel is not open"));
        }

        // TODO: capability authentication?
        if self.packet.chan_on_b.ne(channel
            .counterparty()
            .channel_id()
            .ok_or_else(|| anyhow::anyhow!("missing channel id"))?)
        {
            return Err(anyhow::anyhow!(
                "packet destination channel does not match channel"
            ));
        }
        if self.packet.port_on_b != channel.counterparty().port_id {
            return Err(anyhow::anyhow!(
                "packet destination port does not match channel"
            ));
        }

        let connection = state
            .get_connection(&channel.connection_hops[0])
            .await?
            .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;

        let chain_ts = state
            .get_client_update_time(connection.client_id(), &self.proof_height_on_b)
            .await?;
        let chain_height = self.proof_height_on_b;

        // check that timeout height or timeout timestamp has passed on the other end
        if !self.packet.timed_out(&chain_ts, chain_height) {
            return Err(anyhow::anyhow!(
                "packet has not timed out on the counterparty chain"
            ));
        }

        // verify that we actually sent this packet
        let commitment = state
            .get_packet_commitment(&self.packet)
            .await?
            .ok_or_else(|| anyhow::anyhow!("packet commitment not found"))?;
        if commitment != commit_packet(&self.packet) {
            return Err(anyhow::anyhow!("packet commitment does not match"));
        }

        if channel.ordering == ChannelOrder::Ordered {
            // ordered channel: check that packet has not been received
            if self.next_seq_recv_on_b != self.packet.sequence {
                return Err(anyhow::anyhow!("packet sequence number does not match"));
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
            Ics20Transfer::timeout_packet_check(&mut state, self).await?;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
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

        state.record(event::timeout_packet(&self.packet, &channel));

        let transfer = PortId::transfer();
        if self.packet.port_on_b == transfer {
            Ics20Transfer::timeout_packet_execute(state, self).await;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
        }

        Ok(())
    }
}
