use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use ibc_types::core::{
    channel::{channel::State as ChannelState, events, ChannelId, Packet, PortId},
    client::Height,
};
use tendermint::Time;

use crate::component::{
    channel::{StateReadExt as _, StateWriteExt as _},
    client::StateReadExt as _,
    connection::StateReadExt as _,
};

pub trait CheckStatus: private::Sealed {}

#[derive(Debug, Clone)]
pub enum Checked {}
impl CheckStatus for Checked {}

#[derive(Debug, Clone)]
pub enum Unchecked {}
impl CheckStatus for Unchecked {}

mod private {
    use super::*;

    pub trait Sealed {}
    impl Sealed for Checked {}
    impl Sealed for Unchecked {}
}

pub struct IBCPacket<S: CheckStatus> {
    pub(crate) source_port: PortId,
    pub(crate) source_channel: ChannelId,
    pub(crate) timeout_height: Height,
    pub(crate) timeout_timestamp: u64,
    pub(crate) data: Vec<u8>,

    m: std::marker::PhantomData<S>,
}

impl IBCPacket<Unchecked> {
    pub fn new(
        source_port: PortId,
        source_channel: ChannelId,
        timeout_height: Height,
        timeout_timestamp: u64,
        data: Vec<u8>,
    ) -> Self {
        Self {
            source_port,
            source_channel,
            timeout_height,
            timeout_timestamp,
            data,
            m: std::marker::PhantomData,
        }
    }

    pub fn assume_checked(self) -> IBCPacket<Checked> {
        IBCPacket {
            source_port: self.source_port,
            source_channel: self.source_channel,
            timeout_height: self.timeout_height,
            timeout_timestamp: self.timeout_timestamp,
            data: self.data,
            m: std::marker::PhantomData,
        }
    }
}

impl<S: CheckStatus> IBCPacket<S> {
    pub fn source_port(&self) -> &PortId {
        &self.source_port
    }

    pub fn source_channel(&self) -> &ChannelId {
        &self.source_channel
    }

    pub fn timeout_height(&self) -> &Height {
        &self.timeout_height
    }

    pub fn timeout_timestamp(&self) -> u64 {
        self.timeout_timestamp
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[async_trait]
pub trait SendPacketRead: StateRead {
    /// send_packet_check verifies that a packet can be sent using the provided parameters.
    async fn send_packet_check(
        &self,
        packet: IBCPacket<Unchecked>,
        current_block_time: Time,
    ) -> Result<IBCPacket<Checked>> {
        let channel = self
            .get_channel(&packet.source_channel, &packet.source_port)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "channel {} on port {} does not exist",
                    packet.source_channel,
                    packet.source_port
                )
            })?;

        if channel.state_matches(&ChannelState::Closed) {
            anyhow::bail!(
                "channel {} on port {} is closed",
                packet.source_channel,
                packet.source_port
            );
        }

        // TODO: should we check dest port & channel here?
        let connection = self
            .get_connection(&channel.connection_hops[0])
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("connection {} does not exist", channel.connection_hops[0])
            })?;

        // check that the client state is active so we don't do accidental sends on frozen clients.
        let client_state = self.get_client_state(&connection.client_id).await?;
        if client_state.is_frozen() {
            anyhow::bail!("client {} is frozen", &connection.client_id);
        }

        let latest_consensus_state = self
            .get_verified_consensus_state(&client_state.latest_height(), &connection.client_id)
            .await?;

        let time_elapsed = current_block_time.duration_since(latest_consensus_state.timestamp)?;

        if client_state.expired(time_elapsed) {
            anyhow::bail!("client {} is expired", &connection.client_id);
        }

        let latest_height = client_state.latest_height();

        // check that time timeout height hasn't already passed in the local client tracking the
        // receiving chain
        if packet.timeout_height <= latest_height {
            anyhow::bail!(
                "timeout height {} is less than the latest height on the counterparty {}",
                packet.timeout_height,
                latest_height,
            );
        }

        // check that the timeout timestamp hasn't already passed in the local client tracking
        // the receiving chain
        let chain_ts = latest_consensus_state.timestamp.unix_timestamp_nanos() as u64;
        if packet.timeout_timestamp <= chain_ts {
            anyhow::bail!(
                "timeout timestamp {} is less than the latest timestamp on the counterparty {}",
                packet.timeout_timestamp,
                chain_ts,
            );
        }

        Ok(IBCPacket::<Checked> {
            source_port: packet.source_port.clone(),
            source_channel: packet.source_channel,
            timeout_height: packet.timeout_height,
            timeout_timestamp: packet.timeout_timestamp,
            data: packet.data,

            m: std::marker::PhantomData,
        })
    }
}

impl<T: StateRead + ?Sized> SendPacketRead for T {}

/// This trait, an extension of the Channel, Connection, and Client views, allows a component to
/// send a packet.
#[async_trait]
pub trait SendPacketWrite: StateWrite {
    /// Send a packet on a channel. This assumes that send_packet_check has already been called on
    /// the provided packet.
    async fn send_packet_execute(&mut self, packet: IBCPacket<Checked>) {
        // increment the send sequence counter
        let sequence = self
            .get_send_sequence(&packet.source_channel, &packet.source_port)
            .await
            .expect("able to get send sequence while executing send packet");
        self.put_send_sequence(&packet.source_channel, &packet.source_port, sequence + 1);

        let channel = self
            .get_channel(&packet.source_channel, &packet.source_port)
            .await
            .expect("should be able to get channel")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "channel {} on port {} does not exist",
                    packet.source_channel,
                    packet.source_port
                )
            })
            .expect("should be able to get channel");

        // store commitment to the packet data & packet timeout
        let packet = Packet {
            chan_on_a: packet.source_channel,
            port_on_a: packet.source_port.clone(),
            sequence: sequence.into(),

            chan_on_b: channel
                .counterparty()
                .channel_id
                .clone()
                .expect("should have counterparty channel"),
            port_on_b: channel.counterparty().port_id.clone(),

            timeout_height_on_b: packet.timeout_height.into(),
            timeout_timestamp_on_b: ibc_types::timestamp::Timestamp::from_nanoseconds(
                packet.timeout_timestamp,
            )
            .expect("able to parse timeout timestamp from nanoseconds"),

            data: packet.data,
        };

        self.put_packet_commitment(&packet);

        self.record(
            events::packet::SendPacket {
                packet_data: packet.data.clone(),
                timeout_height: packet.timeout_height_on_b,
                timeout_timestamp: packet.timeout_timestamp_on_b,
                sequence: packet.sequence,
                src_port_id: packet.port_on_a.clone(),
                src_channel_id: packet.chan_on_a.clone(),
                dst_port_id: packet.port_on_b.clone(),
                dst_channel_id: packet.chan_on_b,
                channel_ordering: channel.ordering,
                src_connection_id: channel.connection_hops[0].clone(),
            }
            .into(),
        );
    }
}

impl<T: StateWrite + ?Sized> SendPacketWrite for T {}

#[async_trait]
pub trait WriteAcknowledgement: StateWrite {
    // see: https://github.com/cosmos/ibc/blob/8326e26e7e1188b95c32481ff00348a705b23700/spec/core/ics-004-channel-and-packet-semantics/README.md?plain=1#L779
    async fn write_acknowledgement(&mut self, packet: &Packet, ack_bytes: &[u8]) -> Result<()> {
        if ack_bytes.is_empty() {
            anyhow::bail!("acknowledgement cannot be empty");
        }

        let exists_prev_ack = self
            .get_packet_acknowledgement(
                &packet.port_on_b,
                &packet.chan_on_b,
                packet.sequence.into(),
            )
            .await?
            .is_some();
        if exists_prev_ack {
            anyhow::bail!("acknowledgement already exists");
        }

        let channel = self
            .get_channel(&packet.chan_on_b, &packet.port_on_b)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "channel {} on port {} does not exist",
                    packet.chan_on_b,
                    packet.port_on_b
                )
            })?;

        self.put_packet_acknowledgement(
            &packet.port_on_b,
            &packet.chan_on_b,
            packet.sequence.into(),
            ack_bytes,
        );

        self.record(
            events::packet::WriteAcknowledgement {
                packet_data: packet.data.clone(),
                timeout_height: packet.timeout_height_on_b,
                timeout_timestamp: packet.timeout_timestamp_on_b,
                sequence: packet.sequence,
                src_port_id: packet.port_on_a.clone(),
                src_channel_id: packet.chan_on_a.clone(),
                dst_port_id: packet.port_on_b.clone(),
                dst_channel_id: packet.chan_on_b.clone(),
                acknowledgement: ack_bytes.to_vec(),
                dst_connection_id: channel.connection_hops[0].clone(),
            }
            .into(),
        );

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> WriteAcknowledgement for T {}
