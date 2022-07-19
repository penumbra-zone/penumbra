use ibc::{
    core::{
        ics04_channel::packet::Packet,
        ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
    },
    Height,
};

use jmt::KeyHash;

pub fn client_type(client_id: &ClientId) -> KeyHash {
    format!("clients/{}/clientType", client_id).into()
}

pub fn client_state(client_id: &ClientId) -> KeyHash {
    format!("clients/{}/clientState", client_id).into()
}

pub fn verified_client_consensus_state(client_id: &ClientId, height: &Height) -> KeyHash {
    format!("clients/{}/consensusStates/{}", client_id, height).into()
}

pub fn client_processed_heights(client_id: &ClientId, height: &Height) -> KeyHash {
    format!("clients/{}/processedHeights/{}", client_id, height).into()
}
pub fn client_processed_times(client_id: &ClientId, height: &Height) -> KeyHash {
    format!("clients/{}/processedTimes/{}", client_id, height).into()
}

pub fn client_connections(client_id: &ClientId) -> KeyHash {
    format!("clients/{}/connections", client_id).into()
}

pub fn connection(connection_id: &ConnectionId) -> KeyHash {
    format!("connections/{}", connection_id.as_str()).into()
}

pub fn connection_counter() -> KeyHash {
    "ibc/ics03-connection/connection_counter".into()
}

pub fn channel(channel_id: &ChannelId, port_id: &PortId) -> KeyHash {
    format!("channelEnds/ports/{}/channels/{}", port_id, channel_id).into()
}

pub fn seq_recv(channel_id: &ChannelId, port_id: &PortId) -> KeyHash {
    format!(
        "seqRecvs/ports/{}/channels/{}/nextSequenceRecv",
        port_id, channel_id
    )
    .into()
}

pub fn seq_ack(channel_id: &ChannelId, port_id: &PortId) -> KeyHash {
    format!(
        "seqAcks/ports/{}/channels/{}/nextSequenceAck",
        port_id, channel_id
    )
    .into()
}

pub fn seq_send(channel_id: &ChannelId, port_id: &PortId) -> KeyHash {
    format!(
        "seqSends/ports/{}/channels/{}/nextSequenceSend",
        port_id, channel_id
    )
    .into()
}

pub fn packet_receipt(packet: &Packet) -> KeyHash {
    format!(
        "receipts/ports/{}/channels/{}/receipts/{}",
        packet.destination_port, packet.destination_channel, packet.sequence
    )
    .into()
}

pub fn packet_commitment(packet: &Packet) -> KeyHash {
    format!(
        "commitments/ports/{}/channels/{}/packets/{}",
        packet.source_port, packet.source_channel, packet.sequence
    )
    .into()
}

pub fn packet_commitment_by_port(
    port_id: &PortId,
    channel_id: &ChannelId,
    sequence: u64,
) -> KeyHash {
    format!(
        "commitments/ports/{}/channels/{}/packets/{}",
        port_id, channel_id, sequence
    )
    .into()
}
