use crate::ibc::component::channel::View as ChannelView;
use crate::ibc::component::client::View as ClientView;
use crate::ibc::component::connection::View as ConnectionView;
use crate::Context;
use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics02_client::height::Height;
use ibc::core::ics04_channel::channel::State as ChannelState;
use ibc::core::ics04_channel::packet::Packet;
use ibc::core::ics24_host::identifier::ChannelId;
use ibc::core::ics24_host::identifier::PortId;
use ibc::timestamp::Timestamp;
use penumbra_storage::StateExt;

/// This trait, an extension of the Channel, Connection, and Client views, allows a component to
/// send a packet.
#[async_trait]
pub trait SendPacket: ChannelView + ConnectionView + ClientView {
    async fn send_packet_execute(
        &mut self,
        ctx: Context,
        source_port: &PortId,
        source_channel: &ChannelId,
        timeout_height: Height,
        timeout_timestamp: Timestamp,
    ) {
    }
    async fn send_packet_check(
        &mut self,
        ctx: Context,
        source_port: &PortId,
        source_channel: &ChannelId,
        timeout_height: Height,
        timeout_timestamp: Timestamp,
        data: Vec<u8>,
    ) -> Result<()> {
        let channel = self
            .get_channel(source_channel, source_port)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "channel {} on port {} does not exist",
                    source_channel,
                    source_port
                )
            })?;

        if channel.state_matches(&ChannelState::Closed) {
            return Err(anyhow::anyhow!(
                "channel {} on port {} is closed",
                source_channel,
                source_port
            ));
        }

        // TODO: should we check dest port & channel here?
        let connection = self
            .get_connection(&channel.connection_hops[0])
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("connection {} does not exist", channel.connection_hops[0])
            })?;

        // check that the client state is active so we don't do accidental sends on frozen clients.
        let client_state = self.get_client_state(connection.client_id()).await?;
        if client_state.is_frozen() {
            return Err(anyhow::anyhow!(
                "client {} is frozen",
                connection.client_id()
            ));
        }

        let latest_height = client_state.latest_height();

        // check that time timeout height hasn't already pased in the local client tracking the
        // receiving chain
        if timeout_height <= latest_height {
            return Err(anyhow::anyhow!(
                "timeout height {} is less than the latest height {}",
                timeout_height,
                latest_height.revision_height
            ));
        }

        // increment the send sequence counter
        let sequence = self.get_send_sequence(source_channel, source_port).await?;

        // increment send sequence counter
        self.put_send_sequence(source_channel, source_port, sequence + 1)
            .await;

        // store commitment to the packet data & packet timeout
        let packet = Packet {
            source_channel: source_channel.clone(),
            source_port: source_port.clone(),
            sequence: sequence.into(),

            // NOTE: the packet commitment is solely a function of the source port and channel, so
            // these fields do not affect the commitment. Thus, we can set them to empty values.
            destination_port: PortId::default(),
            destination_channel: ChannelId::default(),

            timeout_height,
            timeout_timestamp: timeout_timestamp.into(),

            data,
        };

        self.put_packet_commitment(&packet).await;

        Ok(())
    }
}

#[async_trait]
pub trait WriteAcknowledgement: StateExt {}
