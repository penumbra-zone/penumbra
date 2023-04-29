use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc_types::core::ics02_client::height::Height as IBCHeight;
use ibc_types::core::ics03_connection::connection::State as ConnectionState;
use ibc_types::core::ics04_channel::channel::Order as ChannelOrder;
use ibc_types::core::ics04_channel::channel::State as ChannelState;
use ibc_types::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
use ibc_types::core::ics24_host::identifier::PortId;
use penumbra_chain::StateReadExt;
use penumbra_storage::{StateRead, StateWrite};

use crate::action_handler::ActionHandler;
use crate::ibc::component::channel::stateful::proof_verification::PacketProofVerifier;
use crate::ibc::component::channel::StateReadExt as _;
use crate::ibc::component::channel::StateWriteExt;
use crate::ibc::component::connection::StateReadExt as _;
use crate::ibc::event;
use crate::ibc::ibc_handler::{AppHandlerCheck, AppHandlerExecute};
use crate::ibc::transfer::Ics20Transfer;

#[async_trait]
impl ActionHandler for MsgRecvPacket {
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
            .get_channel(&self.packet.chan_on_b, &self.packet.port_on_b)
            .await?
            .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
        if !channel.state_matches(&ChannelState::Open) {
            return Err(anyhow::anyhow!("channel is not open"));
        }

        // TODO: capability authentication?

        if self.packet.port_on_a != channel.counterparty().port_id {
            return Err(anyhow::anyhow!("packet source port does not match channel"));
        }
        let counterparty_channel = channel
            .counterparty()
            .channel_id()
            .ok_or_else(|| anyhow::anyhow!("missing channel id"))?;

        if self.packet.chan_on_a.ne(counterparty_channel) {
            return Err(anyhow::anyhow!(
                "packet source channel does not match channel"
            ));
        }

        let connection = state
            .get_connection(&channel.connection_hops[0])
            .await?
            .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;

        if !connection.state_matches(&ConnectionState::Open) {
            return Err(anyhow::anyhow!("connection for channel is not open"));
        }

        let block_height = state.get_block_height().await?;
        let height = IBCHeight::new(0, block_height)?;

        if self.packet.timeout_height_on_b.has_expired(height) {
            return Err(anyhow::anyhow!("packet has timed out"));
        }

        let packet_timeout = self
            .packet
            .timeout_timestamp_on_b
            .into_tm_time()
            .ok_or_else(|| anyhow::anyhow!("invalid timestamp"))?;

        if state.get_block_timestamp().await? >= packet_timeout {
            return Err(anyhow::anyhow!("packet has timed out"));
        }

        state.verify_packet_recv_proof(&connection, self).await?;

        if channel.ordering == ChannelOrder::Ordered {
            let next_sequence_recv = state
                .get_recv_sequence(&self.packet.chan_on_b, &self.packet.port_on_b)
                .await?;

            if self.packet.sequence != next_sequence_recv.into() {
                return Err(anyhow::anyhow!("packet sequence number does not match"));
            }
        } else if state.seen_packet(&self.packet).await? {
            return Err(anyhow::anyhow!("packet has already been processed"));
        }

        let transfer = PortId::transfer();
        if self.packet.port_on_b == transfer {
            Ics20Transfer::recv_packet_check(&mut state, self).await?;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
        }

        if channel.ordering == ChannelOrder::Ordered {
            let mut next_sequence_recv = state
                .get_recv_sequence(&self.packet.chan_on_b, &self.packet.port_on_b)
                .await?;

            next_sequence_recv += 1;
            state.put_recv_sequence(
                &self.packet.chan_on_b,
                &self.packet.port_on_b,
                next_sequence_recv,
            );
        } else {
            // for unordered channels we must set the receipt so it can be verified on the other side
            // this receipt does not contain any data, since the packet has not yet been processed
            // it's just a single store key set to an empty string to indicate that the packet has been received
            state.put_packet_receipt(&self.packet);
        }

        state.record(event::receive_packet(&self.packet, &channel));

        let transfer = PortId::transfer();
        if self.packet.port_on_b == transfer {
            Ics20Transfer::recv_packet_execute(state, self).await;
        } else {
            return Err(anyhow::anyhow!("invalid port id"));
        }

        Ok(())
    }
}
