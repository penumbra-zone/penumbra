use ibc_types::{
    core::{
        ics04_channel::packet::Packet,
        ics24_host::identifier::{ChannelId, ClientId, PortId},
    },
    Height,
};

use penumbra_crypto::asset;

use std::string::String;

pub fn client_type(client_id: &ClientId) -> String {
    format!("clients/{client_id}/clientType")
}

pub fn client_state(client_id: &ClientId) -> String {
    format!("clients/{client_id}/clientState")
}

pub fn verified_client_consensus_state(client_id: &ClientId, height: &Height) -> String {
    format!("clients/{client_id}/consensusStates/{height}")
}

pub fn client_processed_heights(client_id: &ClientId, height: &Height) -> String {
    format!("clients/{client_id}/processedHeights/{height}")
}
pub fn client_processed_times(client_id: &ClientId, height: &Height) -> String {
    format!("clients/{client_id}/processedTimes/{height}")
}

pub mod connections {
    use ibc_types::core::ics24_host::identifier::{ClientId, ConnectionId};

    use std::string::String;

    // This is part of the ICS-3 spec but not exposed yet:
    // https://github.com/cosmos/ibc/tree/main/spec/core/ics-003-connection-semantics
    #[allow(dead_code)]
    pub fn by_client_id_list(client_id: &ClientId) -> String {
        format!("clients/{client_id}/connections/")
    }

    pub fn by_client_id(client_id: &ClientId, connection_id: &ConnectionId) -> String {
        format!(
            "clients/{}/connections/{}",
            client_id,
            connection_id.as_str()
        )
    }

    pub fn by_connection_id(connection_id: &ConnectionId) -> String {
        format!("connections/{}", connection_id.as_str())
    }

    pub fn counter() -> &'static str {
        "ibc/ics03-connection/connection_counter"
    }
}

pub fn channel(channel_id: &ChannelId, port_id: &PortId) -> String {
    format!("channelEnds/ports/{port_id}/channels/{channel_id}")
}

pub fn seq_recv(channel_id: &ChannelId, port_id: &PortId) -> String {
    format!("seqRecvs/ports/{port_id}/channels/{channel_id}/nextSequenceRecv")
}

pub fn seq_ack(channel_id: &ChannelId, port_id: &PortId) -> String {
    format!("seqAcks/ports/{port_id}/channels/{channel_id}/nextSequenceAck")
}

pub fn seq_send(channel_id: &ChannelId, port_id: &PortId) -> String {
    format!("seqSends/ports/{port_id}/channels/{channel_id}/nextSequenceSend")
}

pub fn packet_receipt(packet: &Packet) -> String {
    format!(
        "receipts/ports/{}/channels/{}/receipts/{}",
        packet.port_on_b, packet.chan_on_b, packet.sequence
    )
}

pub fn receipt_by_channel(port_id: &PortId, channel_id: &ChannelId, sequence: u64) -> String {
    format!(
        "receipts/ports/{port_id}/channels/{channel_id}/receipts/{sequence}",
        port_id = port_id,
        channel_id = channel_id,
        sequence = sequence
    )
}

pub fn packet_commitment(packet: &Packet) -> String {
    format!(
        "commitments/ports/{}/channels/{}/packets/{}",
        packet.port_on_a, packet.chan_on_a, packet.sequence
    )
}

pub fn packet_acknowledgement(port_id: &PortId, channel_id: &ChannelId, sequence: u64) -> String {
    format!(
        "acks/ports/{}/channels/{}/sequences/{}",
        port_id, channel_id, sequence
    )
}

pub fn packet_commitment_by_port(
    port_id: &PortId,
    channel_id: &ChannelId,
    sequence: u64,
) -> String {
    format!("commitments/ports/{port_id}/channels/{channel_id}/packets/{sequence}")
}

pub fn ics20_value_balance(channel_id: &ChannelId, asset_id: &asset::Id) -> String {
    format!("ics20-value-balance/{channel_id}/{asset_id}")
}
