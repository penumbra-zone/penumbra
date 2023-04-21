use self::stateful::proof_verification::commit_acknowledgement;

use super::state_key;
use anyhow::Result;
use async_trait::async_trait;

use ibc_types::core::ics02_client::consensus_state::ConsensusState;
use ibc_types::core::ics03_connection::connection::ConnectionEnd;
use ibc_types::core::ics04_channel::channel::ChannelEnd;
use ibc_types::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
use ibc_types::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
use ibc_types::core::ics04_channel::msgs::timeout::MsgTimeout;
use ibc_types::core::ics04_channel::packet::Packet;
use ibc_types::core::ics24_host::identifier::ChannelId;
use ibc_types::core::ics24_host::identifier::PortId;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::StateRead;
use penumbra_storage::StateWrite;

pub(crate) mod stateful;

use stateful::proof_verification::commit_packet;

#[async_trait]
pub trait StateWriteExt: StateWrite + StateReadExt {
    fn put_channel_counter(&mut self, counter: u64) {
        self.put_proto::<u64>("ibc_channel_counter".into(), counter);
    }

    async fn next_channel_id(&mut self) -> Result<ChannelId> {
        let ctr = self.get_channel_counter().await?;
        self.put_channel_counter(ctr + 1);

        Ok(ChannelId::new(ctr))
    }

    fn put_channel(&mut self, channel_id: &ChannelId, port_id: &PortId, channel: ChannelEnd) {
        self.put(state_key::channel(channel_id, port_id), channel);
    }

    fn put_ack_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(state_key::seq_ack(channel_id, port_id), sequence);
    }

    fn put_recv_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(state_key::seq_recv(channel_id, port_id), sequence);
    }

    fn put_send_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(state_key::seq_send(channel_id, port_id), sequence);
    }

    fn put_packet_receipt(&mut self, packet: &Packet) {
        self.put_proto::<String>(state_key::packet_receipt(packet), "1".to_string());
    }

    fn put_packet_commitment(&mut self, packet: &Packet) {
        let commitment_key = state_key::packet_commitment(packet);
        let packet_hash = commit_packet(packet);

        self.put_proto::<Vec<u8>>(commitment_key, packet_hash);
    }

    fn delete_packet_commitment(
        &mut self,
        channel_id: &ChannelId,
        port_id: &PortId,
        sequence: u64,
    ) {
        self.put_proto::<Vec<u8>>(
            state_key::packet_commitment_by_port(port_id, channel_id, sequence),
            vec![],
        );
    }

    fn put_packet_acknowledgement(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: u64,
        acknowledgement: &[u8],
    ) {
        self.put_proto::<Vec<u8>>(
            state_key::packet_acknowledgement(port_id, channel_id, sequence),
            commit_acknowledgement(acknowledgement),
        );
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}

#[async_trait]
pub trait StateReadExt: StateRead {
    async fn get_channel_counter(&self) -> Result<u64> {
        self.get_proto::<u64>("ibc_channel_counter")
            .await
            .map(|counter| counter.unwrap_or(0))
    }

    async fn get_channel(
        &self,
        channel_id: &ChannelId,
        port_id: &PortId,
    ) -> Result<Option<ChannelEnd>> {
        self.get(&state_key::channel(channel_id, port_id)).await
    }

    async fn get_recv_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        self.get_proto::<u64>(&state_key::seq_recv(channel_id, port_id))
            .await
            .map(|sequence| sequence.unwrap_or(0))
    }

    async fn get_ack_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        self.get_proto::<u64>(&state_key::seq_ack(channel_id, port_id))
            .await
            .map(|sequence| sequence.unwrap_or(0))
    }

    async fn get_send_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        self.get_proto::<u64>(&state_key::seq_send(channel_id, port_id))
            .await
            .map(|sequence| sequence.unwrap_or(0))
    }

    async fn seen_packet(&self, packet: &Packet) -> Result<bool> {
        self.get_proto::<String>(&state_key::packet_receipt(packet))
            .await
            .map(|res| res.is_some())
    }

    async fn get_packet_commitment(&self, packet: &Packet) -> Result<Option<Vec<u8>>> {
        let commitment = self
            .get_proto::<Vec<u8>>(&state_key::packet_commitment(packet))
            .await?;

        // this is for the special case where the commitment is empty, we consider this None.
        if let Some(commitment) = commitment.as_ref() {
            if commitment.is_empty() {
                return Ok(None);
            }
        }

        Ok(commitment)
    }

    async fn get_packet_acknowledgement(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: u64,
    ) -> Result<Option<Vec<u8>>> {
        self.get_proto::<Vec<u8>>(&state_key::packet_acknowledgement(
            port_id, channel_id, sequence,
        ))
        .await
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}
