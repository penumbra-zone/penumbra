use async_trait::async_trait;
use cnidarium_component::ChainStateReadExt;
use ibc_proto::ibc::core::channel::v1::query_server::Query as ConsensusQuery;
use ibc_proto::ibc::core::channel::v1::{
    PacketState, QueryChannelClientStateRequest, QueryChannelClientStateResponse,
    QueryChannelConsensusStateRequest, QueryChannelConsensusStateResponse, QueryChannelRequest,
    QueryChannelResponse, QueryChannelsRequest, QueryChannelsResponse,
    QueryConnectionChannelsRequest, QueryConnectionChannelsResponse,
    QueryNextSequenceReceiveRequest, QueryNextSequenceReceiveResponse,
    QueryNextSequenceSendRequest, QueryNextSequenceSendResponse, QueryPacketAcknowledgementRequest,
    QueryPacketAcknowledgementResponse, QueryPacketAcknowledgementsRequest,
    QueryPacketAcknowledgementsResponse, QueryPacketCommitmentRequest,
    QueryPacketCommitmentResponse, QueryPacketCommitmentsRequest, QueryPacketCommitmentsResponse,
    QueryPacketReceiptRequest, QueryPacketReceiptResponse, QueryUnreceivedAcksRequest,
    QueryUnreceivedAcksResponse, QueryUnreceivedPacketsRequest, QueryUnreceivedPacketsResponse,
};
use ibc_proto::ibc::core::client::v1::Height;
use ibc_types::core::channel::{ChannelId, IdentifiedChannelEnd, PortId};
use ibc_types::core::connection::ConnectionId;

use std::str::FromStr;

use crate::component::rpc::{Snapshot, Storage};
use crate::component::ChannelStateReadExt;

use super::IbcQuery;

#[async_trait]
impl<C: ChainStateReadExt + Snapshot + 'static, S: Storage<C>> ConsensusQuery for IbcQuery<C, S> {
    /// Channel queries an IBC Channel.
    async fn channel(
        &self,
        _request: tonic::Request<QueryChannelRequest>,
    ) -> std::result::Result<tonic::Response<QueryChannelResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// Channels queries all the IBC channels of a chain.
    async fn channels(
        &self,
        _request: tonic::Request<QueryChannelsRequest>,
    ) -> std::result::Result<tonic::Response<QueryChannelsResponse>, tonic::Status> {
        let snapshot = self.0.latest_snapshot();
        let height = Height {
            revision_number: 0,
            revision_height: snapshot.version(),
        };

        let channel_counter = snapshot
            .get_channel_counter()
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get channel counter: {e}")))?;

        let mut channels = vec![];
        for chan_idx in 0..channel_counter {
            let chan_id = ChannelId(format!("channel-{}", chan_idx));
            let channel = snapshot
                .get_channel(&chan_id, &PortId::transfer())
                .await
                .map_err(|e| {
                    tonic::Status::aborted(format!("couldn't get channel {chan_id}: {e}"))
                })?
                .ok_or("unable to get channel")
                .map_err(|e| {
                    tonic::Status::aborted(format!("couldn't get channel {chan_id}: {e}"))
                })?;

            let id_chan = IdentifiedChannelEnd {
                channel_id: chan_id,
                port_id: PortId::transfer(),
                channel_end: channel,
            };
            channels.push(id_chan.into());
        }

        let res = QueryChannelsResponse {
            channels,
            pagination: None,
            height: Some(height),
        };

        Ok(tonic::Response::new(res))
    }
    /// ConnectionChannels queries all the channels associated with a connection
    /// end.
    async fn connection_channels(
        &self,
        request: tonic::Request<QueryConnectionChannelsRequest>,
    ) -> std::result::Result<tonic::Response<QueryConnectionChannelsResponse>, tonic::Status> {
        let snapshot = self.0.latest_snapshot();
        let height = Height {
            revision_number: 0,
            revision_height: snapshot.version(),
        };
        let request = request.get_ref();

        let connection_id: ConnectionId = ConnectionId::from_str(&request.connection)
            .map_err(|e| tonic::Status::aborted(format!("invalid connection id: {e}")))?;

        // look up all of the channels for this connection
        let channel_counter = snapshot
            .get_channel_counter()
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get channel counter: {e}")))?;

        let mut channels = vec![];
        for chan_idx in 0..channel_counter {
            let chan_id = ChannelId(format!("channel-{}", chan_idx));
            let channel = snapshot
                .get_channel(&chan_id, &PortId::transfer())
                .await
                .map_err(|e| {
                    tonic::Status::aborted(format!("couldn't get channel {chan_id}: {e}"))
                })?
                .ok_or("unable to get channel")
                .map_err(|e| {
                    tonic::Status::aborted(format!("couldn't get channel {chan_id}: {e}"))
                })?;
            if channel.connection_hops.contains(&connection_id) {
                let id_chan = IdentifiedChannelEnd {
                    channel_id: chan_id,
                    port_id: PortId::transfer(),
                    channel_end: channel,
                };
                channels.push(id_chan.into());
            }
        }

        let res = QueryConnectionChannelsResponse {
            channels,
            pagination: None,
            height: Some(height),
        };

        Ok(tonic::Response::new(res))
    }
    /// ChannelClientState queries for the client state for the channel associated
    /// with the provided channel identifiers.
    async fn channel_client_state(
        &self,
        _request: tonic::Request<QueryChannelClientStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryChannelClientStateResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// ChannelConsensusState queries for the consensus state for the channel
    /// associated with the provided channel identifiers.
    async fn channel_consensus_state(
        &self,
        _request: tonic::Request<QueryChannelConsensusStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryChannelConsensusStateResponse>, tonic::Status>
    {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// PacketCommitment queries a stored packet commitment hash.
    async fn packet_commitment(
        &self,
        _request: tonic::Request<QueryPacketCommitmentRequest>,
    ) -> std::result::Result<tonic::Response<QueryPacketCommitmentResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// PacketCommitments returns all the packet commitments hashes associated
    /// with a channel.
    async fn packet_commitments(
        &self,
        request: tonic::Request<QueryPacketCommitmentsRequest>,
    ) -> std::result::Result<tonic::Response<QueryPacketCommitmentsResponse>, tonic::Status> {
        let snapshot = self.0.latest_snapshot();
        let height = snapshot.version();
        let request = request.get_ref();

        let chan_id: ChannelId = ChannelId::from_str(&request.channel_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid channel id: {e}")))?;
        let port_id: PortId = PortId::from_str(&request.port_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid port id: {e}")))?;

        let mut commitment_states = vec![];
        let commitment_counter = snapshot
            .get_send_sequence(&chan_id, &port_id)
            .await
            .map_err(|e| {
                tonic::Status::aborted(format!(
                    "couldn't get send sequence for channel {chan_id} and port {port_id}: {e}"
                ))
            })?;

        // this starts at 1; the first commitment index is 1 (from ibc spec)
        for commitment_idx in 1..commitment_counter {
            let commitment = snapshot
                .get_packet_commitment_by_id(&chan_id, &port_id, commitment_idx)
                .await.map_err(|e| {
                    tonic::Status::aborted(format!(
                        "couldn't get packet commitment for channel {chan_id} and port {port_id} at index {commitment_idx}: {e}"
                    ))
                })?;
            if commitment.is_none() {
                continue;
            }
            let commitment = commitment.expect("commitment existence was checked earlier");

            let commitment_state = PacketState {
                port_id: request.port_id.clone(),
                channel_id: request.channel_id.clone(),
                sequence: commitment_idx,
                data: commitment.clone(),
            };

            commitment_states.push(commitment_state);
        }

        let height = Height {
            revision_number: 0,
            revision_height: height,
        };

        let res = QueryPacketCommitmentsResponse {
            commitments: commitment_states,
            pagination: None,
            height: Some(height),
        };

        Ok(tonic::Response::new(res))
    }
    /// PacketReceipt queries if a given packet sequence has been received on the
    /// queried chain
    async fn packet_receipt(
        &self,
        _request: tonic::Request<QueryPacketReceiptRequest>,
    ) -> std::result::Result<tonic::Response<QueryPacketReceiptResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// PacketAcknowledgement queries a stored packet acknowledgement hash.
    async fn packet_acknowledgement(
        &self,
        _request: tonic::Request<QueryPacketAcknowledgementRequest>,
    ) -> std::result::Result<tonic::Response<QueryPacketAcknowledgementResponse>, tonic::Status>
    {
        Err(tonic::Status::unimplemented("not implemented"))
    }
    /// PacketAcknowledgements returns all the packet acknowledgements associated
    /// with a channel.
    async fn packet_acknowledgements(
        &self,
        request: tonic::Request<QueryPacketAcknowledgementsRequest>,
    ) -> std::result::Result<tonic::Response<QueryPacketAcknowledgementsResponse>, tonic::Status>
    {
        let snapshot = self.0.latest_snapshot();
        let height = Height {
            revision_number: 0,
            revision_height: snapshot.version(),
        };
        let request = request.get_ref();

        let chan_id: ChannelId = ChannelId::from_str(&request.channel_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid channel id: {e}")))?;
        let port_id: PortId = PortId::from_str(&request.port_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid port id: {e}")))?;

        let ack_counter = snapshot
            .get_ack_sequence(&chan_id, &port_id)
            .await
            .map_err(|e| {
                tonic::Status::aborted(format!(
                    "couldn't get ack sequence for channel {chan_id} and port {port_id}: {e}"
                ))
            })?;

        let mut acks = vec![];
        for ack_idx in 0..ack_counter {
            let maybe_ack = snapshot
                .get_packet_acknowledgement(&port_id, &chan_id, ack_idx)
                .await.map_err(|e| {
                    tonic::Status::aborted(format!(
                        "couldn't get packet acknowledgement for channel {chan_id} and port {port_id} at index {ack_idx}: {e}"
                    ))
                })?;

            // Only include the ack if it was found; otherwise, signal lack of
            // by omitting it from the response.
            if let Some(ack) = maybe_ack {
                let ack_state = PacketState {
                    port_id: request.port_id.clone(),
                    channel_id: request.channel_id.clone(),
                    sequence: ack_idx,
                    data: ack.clone(),
                };

                acks.push(ack_state);
            }
        }

        let res = QueryPacketAcknowledgementsResponse {
            acknowledgements: acks,
            pagination: None,
            height: Some(height),
        };

        Ok(tonic::Response::new(res))
    }
    /// UnreceivedPackets returns all the unreceived IBC packets associated with a
    /// channel and sequences.
    async fn unreceived_packets(
        &self,
        request: tonic::Request<QueryUnreceivedPacketsRequest>,
    ) -> std::result::Result<tonic::Response<QueryUnreceivedPacketsResponse>, tonic::Status> {
        let snapshot = self.0.latest_snapshot();
        let height = snapshot.version();
        let request = request.get_ref();

        let chan_id: ChannelId = ChannelId::from_str(&request.channel_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid channel id: {e}")))?;
        let port_id: PortId = PortId::from_str(&request.port_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid port id: {e}")))?;

        let mut unreceived_seqs = vec![];

        for seq in request.packet_commitment_sequences.clone() {
            if seq == 0 {
                return Err(tonic::Status::aborted(format!(
                    "packet sequence {} cannot be 0",
                    seq
                )));
            }

            if !snapshot
                .seen_packet_by_channel(&chan_id, &port_id, seq)
                .await.map_err(|e| {
                    tonic::Status::aborted(format!(
                        "couldn't get packet commitment for channel {chan_id} and port {port_id} at index {seq}: {e}"
                    ))
                })?
            {
                unreceived_seqs.push(seq);
            }
        }

        let height = Height {
            revision_number: 0,
            revision_height: height,
        };

        let res = QueryUnreceivedPacketsResponse {
            sequences: unreceived_seqs,
            height: Some(height),
        };

        Ok(tonic::Response::new(res))
    }
    /// UnreceivedAcks returns all the unreceived IBC acknowledgements associated
    /// with a channel and sequences.
    async fn unreceived_acks(
        &self,
        request: tonic::Request<QueryUnreceivedAcksRequest>,
    ) -> std::result::Result<tonic::Response<QueryUnreceivedAcksResponse>, tonic::Status> {
        let snapshot = self.0.latest_snapshot();
        let height = Height {
            revision_number: 0,
            revision_height: snapshot.version(),
        };
        let request = request.get_ref();

        let chan_id: ChannelId = ChannelId::from_str(&request.channel_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid channel id: {e}")))?;
        let port_id: PortId = PortId::from_str(&request.port_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid port id: {e}")))?;

        let mut unreceived_seqs = vec![];

        for seq in request.packet_ack_sequences.clone() {
            if seq == 0 {
                return Err(tonic::Status::aborted(format!(
                    "packet sequence {} cannot be 0",
                    seq
                )));
            }

            if snapshot
                .get_packet_commitment_by_id(&chan_id, &port_id, seq)
                .await.map_err(|e| {
                    tonic::Status::aborted(format!(
                        "couldn't get packet commitment for channel {chan_id} and port {port_id} at index {seq}: {e}"
                    ))
                })?
                .is_some()
            {
                unreceived_seqs.push(seq);
            }
        }

        let res = QueryUnreceivedAcksResponse {
            sequences: unreceived_seqs,
            height: Some(height),
        };

        Ok(tonic::Response::new(res))
    }

    /// NextSequenceReceive returns the next receive sequence for a given channel.
    async fn next_sequence_receive(
        &self,
        _request: tonic::Request<QueryNextSequenceReceiveRequest>,
    ) -> std::result::Result<tonic::Response<QueryNextSequenceReceiveResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    /// NextSequenceSend returns the next send sequence for a given channel.
    async fn next_sequence_send(
        &self,
        _request: tonic::Request<QueryNextSequenceSendRequest>,
    ) -> std::result::Result<tonic::Response<QueryNextSequenceSendResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
}
