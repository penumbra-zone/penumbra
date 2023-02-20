use super::component::channel::StateReadExt as _;
use super::component::channel::StateWriteExt as _;
use super::component::client::StateReadExt as _;
use super::component::connection::StateReadExt as _;
use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics02_client::height::Height;
use ibc::core::ics04_channel::channel::State as ChannelState;
use ibc::core::ics04_channel::packet::Packet;
use ibc::core::ics24_host::identifier::ChannelId;
use ibc::core::ics24_host::identifier::PortId;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::action::Ics20Withdrawal;

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

impl From<Ics20Withdrawal> for IBCPacket<Unchecked> {
    fn from(withdrawal: Ics20Withdrawal) -> Self {
        Self {
            source_port: withdrawal.source_port.clone(),
            source_channel: withdrawal.source_channel.clone(),
            timeout_height: ibc::Height::new(0, withdrawal.timeout_height).unwrap(),
            timeout_timestamp: withdrawal.timeout_time,
            data: withdrawal.packet_data(),

            m: std::marker::PhantomData,
        }
    }
}

#[async_trait]
pub trait SendPacketRead: StateRead {
    /// send_packet_check verifies that a packet can be sent using the provided parameters.
    async fn send_packet_check(&self, packet: IBCPacket<Unchecked>) -> Result<IBCPacket<Checked>> {
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
            return Err(anyhow::anyhow!(
                "channel {} on port {} is closed",
                packet.source_channel,
                packet.source_port
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
        if packet.timeout_height <= latest_height {
            return Err(anyhow::anyhow!(
                "timeout height {} is less than the latest height {}",
                packet.timeout_height,
                latest_height.revision_height()
            ));
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
            .unwrap();
        self.put_send_sequence(&packet.source_channel, &packet.source_port, sequence + 1);

        // store commitment to the packet data & packet timeout
        let packet = Packet {
            chan_on_a: packet.source_channel,
            port_on_a: packet.source_port.clone(),
            sequence: sequence.into(),

            // NOTE: the packet commitment is solely a function of the source port and channel, so
            // these fields do not affect the commitment. Thus, we can set them to empty values.
            chan_on_b: ChannelId::default(),
            port_on_b: PortId::default(),

            timeout_height_on_b: packet.timeout_height.into(),
            timeout_timestamp_on_b: ibc::timestamp::Timestamp::from_nanoseconds(
                packet.timeout_timestamp,
            )
            .unwrap(),

            data: packet.data,
        };

        self.put_packet_commitment(&packet);
    }
}

impl<T: StateWrite + ?Sized> SendPacketWrite for T {}

#[async_trait]
pub trait WriteAcknowledgement: StateWrite {}
