use crate::prefix::MerklePrefixExt;
use crate::IBC_COMMITMENT_PREFIX;
use async_trait::async_trait;
use cnidarium_component::ChainStateReadExt;
use ibc_proto::ibc::core::channel::v1::query_server::Query as ConsensusQuery;
use ibc_proto::ibc::core::channel::v1::{
    Channel, PacketState, QueryChannelClientStateRequest, QueryChannelClientStateResponse,
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
use ibc_proto::ibc::core::client::v1::{Height, IdentifiedClientState};
use ibc_types::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath,
    ReceiptPath, SeqRecvPath, SeqSendPath,
};
use ibc_types::DomainType;

use ibc_types::core::channel::{ChannelId, IdentifiedChannelEnd, PortId};
use ibc_types::core::connection::ConnectionId;
use prost::Message;

use std::str::FromStr;

use crate::component::rpc::{Snapshot, Storage};
use crate::component::{ChannelStateReadExt, ConnectionStateReadExt};

use super::IbcQuery;

#[async_trait]
impl<C: ChainStateReadExt + Snapshot + 'static, S: Storage<C>> ConsensusQuery for IbcQuery<C, S> {
    /// Channel queries an IBC Channel.
    async fn channel(
        &self,
        request: tonic::Request<QueryChannelRequest>,
    ) -> std::result::Result<tonic::Response<QueryChannelResponse>, tonic::Status> {
        let snapshot = self.0.latest_snapshot();
        let channel_id = ChannelId::from_str(request.get_ref().channel_id.as_str())
            .map_err(|e| tonic::Status::aborted(format!("invalid channel id: {e}")))?;
        let port_id = PortId::from_str(request.get_ref().port_id.as_str())
            .map_err(|e| tonic::Status::aborted(format!("invalid port id: {e}")))?;

        let (channel, proof) = snapshot
            .get_with_proof(
                IBC_COMMITMENT_PREFIX
                    .apply_string(ChannelEndPath::new(&port_id, &channel_id).to_string())
                    .as_bytes()
                    .to_vec(),
            )
            .await
            .map(|res| {
                let channel = res
                    .0
                    .map(|chan_bytes| Channel::decode(chan_bytes.as_ref()))
                    .transpose();

                (channel, res.1)
            })
            .map_err(|e| tonic::Status::aborted(format!("couldn't get channel: {e}")))?;

        let channel =
            channel.map_err(|e| tonic::Status::aborted(format!("couldn't decode channel: {e}")))?;

        let res = QueryChannelResponse {
            channel,
            proof: proof.encode_to_vec(),
            proof_height: Some(Height {
                revision_number: 0,
                revision_height: snapshot.version(),
            }),
        };

        Ok(tonic::Response::new(res))
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
        request: tonic::Request<QueryChannelClientStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryChannelClientStateResponse>, tonic::Status> {
        let snapshot = self.0.latest_snapshot();

        // 1. get the channel
        let channel_id = ChannelId::from_str(request.get_ref().channel_id.as_str())
            .map_err(|e| tonic::Status::aborted(format!("invalid channel id: {e}")))?;
        let port_id = PortId::from_str(request.get_ref().port_id.as_str())
            .map_err(|e| tonic::Status::aborted(format!("invalid port id: {e}")))?;

        let channel = snapshot
            .get_channel(&channel_id, &port_id)
            .await
            .map_err(|e| {
                tonic::Status::aborted(format!(
                    "couldn't get channel {channel_id} for port {port_id}: {e}"
                ))
            })?
            .ok_or("unable to get channel")
            .map_err(|e| {
                tonic::Status::aborted(format!(
                    "couldn't get channel {channel_id} for port {port_id}: {e}"
                ))
            })?;

        // 2. get the connection for the channel
        let connection_id = channel
            .connection_hops
            .first()
            .ok_or("channel has no connection hops")
            .map_err(|e| {
                tonic::Status::aborted(format!(
                    "couldn't get connection for channel {channel_id} for port {port_id}: {e}"
                ))
            })?;

        let connection = snapshot.get_connection(&connection_id).await.map_err(|e| {
            tonic::Status::aborted(format!(
                "couldn't get connection {connection_id} for channel {channel_id} for port {port_id}: {e}"
            ))
        })?.ok_or("unable to get connection").map_err(|e| {
            tonic::Status::aborted(format!(
                "couldn't get connection {connection_id} for channel {channel_id} for port {port_id}: {e}"
            ))
        })?;

        // 3. get the client state for the connection
        let (client_state, proof) = snapshot
            .get_with_proof(
                IBC_COMMITMENT_PREFIX
                    .apply_string(ClientStatePath::new(&connection.client_id).to_string())
                    .as_bytes()
                    .to_vec(),
            )
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get client state: {e}")))?;

        let client_state_any = client_state
            .map(|cs_bytes| ibc_proto::google::protobuf::Any::decode(cs_bytes.as_ref()))
            .transpose()
            .map_err(|e| tonic::Status::aborted(format!("couldn't decode client state: {e}")))?;

        let identified_client_state = IdentifiedClientState {
            client_id: connection.client_id.clone().to_string(),
            client_state: client_state_any,
        };

        Ok(tonic::Response::new(QueryChannelClientStateResponse {
            identified_client_state: Some(identified_client_state),
            proof: proof.encode_to_vec(),
            proof_height: Some(Height {
                revision_number: 0,
                revision_height: snapshot.version(),
            }),
        }))
    }
    /// ChannelConsensusState queries for the consensus state for the channel
    /// associated with the provided channel identifiers.
    async fn channel_consensus_state(
        &self,
        request: tonic::Request<QueryChannelConsensusStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryChannelConsensusStateResponse>, tonic::Status>
    {
        let snapshot = self.0.latest_snapshot();
        let consensus_state_height = ibc_types::core::client::Height {
            revision_number: request.get_ref().revision_number,
            revision_height: request.get_ref().revision_height,
        };

        // 1. get the channel
        let channel_id = ChannelId::from_str(request.get_ref().channel_id.as_str())
            .map_err(|e| tonic::Status::aborted(format!("invalid channel id: {e}")))?;
        let port_id = PortId::from_str(request.get_ref().port_id.as_str())
            .map_err(|e| tonic::Status::aborted(format!("invalid port id: {e}")))?;

        let channel = snapshot
            .get_channel(&channel_id, &port_id)
            .await
            .map_err(|e| {
                tonic::Status::aborted(format!(
                    "couldn't get channel {channel_id} for port {port_id}: {e}"
                ))
            })?
            .ok_or("unable to get channel")
            .map_err(|e| {
                tonic::Status::aborted(format!(
                    "couldn't get channel {channel_id} for port {port_id}: {e}"
                ))
            })?;

        // 2. get the connection for the channel
        let connection_id = channel
            .connection_hops
            .first()
            .ok_or("channel has no connection hops")
            .map_err(|e| {
                tonic::Status::aborted(format!(
                    "couldn't get connection for channel {channel_id} for port {port_id}: {e}"
                ))
            })?;

        let connection = snapshot.get_connection(&connection_id).await.map_err(|e| {
            tonic::Status::aborted(format!(
                "couldn't get connection {connection_id} for channel {channel_id} for port {port_id}: {e}"
            ))
        })?.ok_or("unable to get connection").map_err(|e| {
            tonic::Status::aborted(format!(
                "couldn't get connection {connection_id} for channel {channel_id} for port {port_id}: {e}"
            ))
        })?;

        // 3. get the consensus state for the connection
        let (consensus_state, proof) = snapshot
            .get_with_proof(
                IBC_COMMITMENT_PREFIX
                    .apply_string(
                        ClientConsensusStatePath::new(
                            &connection.client_id,
                            &consensus_state_height,
                        )
                        .to_string(),
                    )
                    .as_bytes()
                    .to_vec(),
            )
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get client state: {e}")))?;

        let consensus_state_any = consensus_state
            .map(|cs_bytes| ibc_proto::google::protobuf::Any::decode(cs_bytes.as_ref()))
            .transpose()
            .map_err(|e| tonic::Status::aborted(format!("couldn't decode client state: {e}")))?;

        Ok(tonic::Response::new(QueryChannelConsensusStateResponse {
            consensus_state: consensus_state_any,
            client_id: connection.client_id.clone().to_string(),
            proof: proof.encode_to_vec(),
            proof_height: Some(Height {
                revision_number: 0,
                revision_height: snapshot.version(),
            }),
        }))
    }
    /// PacketCommitment queries a stored packet commitment hash.
    async fn packet_commitment(
        &self,
        request: tonic::Request<QueryPacketCommitmentRequest>,
    ) -> std::result::Result<tonic::Response<QueryPacketCommitmentResponse>, tonic::Status> {
        let snapshot = self.0.latest_snapshot();

        let port_id = PortId::from_str(&request.get_ref().port_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid port id: {e}")))?;
        let channel_id = ChannelId::from_str(&request.get_ref().channel_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid channel id: {e}")))?;

        let (commitment, proof) = snapshot
            .get_with_proof(
                IBC_COMMITMENT_PREFIX
                    .apply_string(
                        CommitmentPath::new(
                            &port_id,
                            &channel_id,
                            request.get_ref().sequence.into(),
                        )
                        .to_string(),
                    )
                    .as_bytes()
                    .to_vec(),
            )
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get packet commitment: {e}")))?;

        let commitment =
            commitment.ok_or_else(|| tonic::Status::aborted("commitment not found"))?;

        Ok(tonic::Response::new(QueryPacketCommitmentResponse {
            commitment,
            proof: proof.encode_to_vec(),
            proof_height: Some(Height {
                revision_number: 0,
                revision_height: snapshot.version(),
            }),
        }))
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
        request: tonic::Request<QueryPacketReceiptRequest>,
    ) -> std::result::Result<tonic::Response<QueryPacketReceiptResponse>, tonic::Status> {
        let snapshot = self.0.latest_snapshot();

        let port_id = PortId::from_str(&request.get_ref().port_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid port id: {e}")))?;
        let channel_id = ChannelId::from_str(&request.get_ref().channel_id)
            .map_err(|e| tonic::Status::aborted(format!("invalid channel id: {e}")))?;

        let (receipt, proof) = snapshot
            .get_with_proof(
                IBC_COMMITMENT_PREFIX
                    .apply_string(
                        ReceiptPath::new(&port_id, &channel_id, request.get_ref().sequence.into())
                            .to_string(),
                    )
                    .as_bytes()
                    .to_vec(),
            )
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get packet commitment: {e}")))?;

        Ok(tonic::Response::new(QueryPacketReceiptResponse {
            received: receipt.is_some(),
            proof: proof.encode_to_vec(),
            proof_height: Some(Height {
                revision_number: 0,
                revision_height: snapshot.version(),
            }),
        }))
    }
    /// PacketAcknowledgement queries a stored packet acknowledgement hash.
    async fn packet_acknowledgement(
        &self,
        request: tonic::Request<QueryPacketAcknowledgementRequest>,
    ) -> std::result::Result<tonic::Response<QueryPacketAcknowledgementResponse>, tonic::Status>
    {
        let snapshot = self.0.latest_snapshot();
        let channel_id = ChannelId::from_str(request.get_ref().channel_id.as_str())
            .map_err(|e| tonic::Status::aborted(format!("invalid channel id: {e}")))?;
        let port_id = PortId::from_str(request.get_ref().port_id.as_str())
            .map_err(|e| tonic::Status::aborted(format!("invalid port id: {e}")))?;

        let (acknowledgement, proof) = snapshot
            .get_with_proof(
                IBC_COMMITMENT_PREFIX
                    .apply_string(
                        AckPath::new(&port_id, &channel_id, request.get_ref().sequence.into())
                            .to_string(),
                    )
                    .as_bytes()
                    .to_vec(),
            )
            .await
            .map_err(|e| {
                tonic::Status::aborted(format!("couldn't get packet acknowledgement: {e}"))
            })?;

        let acknowledgement =
            acknowledgement.ok_or_else(|| tonic::Status::aborted("acknowledgement not found"))?;

        Ok(tonic::Response::new(QueryPacketAcknowledgementResponse {
            acknowledgement,
            proof: proof.encode_to_vec(),
            proof_height: Some(Height {
                revision_number: 0,
                revision_height: snapshot.version(),
            }),
        }))
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
        request: tonic::Request<QueryNextSequenceReceiveRequest>,
    ) -> std::result::Result<tonic::Response<QueryNextSequenceReceiveResponse>, tonic::Status> {
        let snapshot = self.0.latest_snapshot();

        let channel_id = ChannelId::from_str(request.get_ref().channel_id.as_str())
            .map_err(|e| tonic::Status::aborted(format!("invalid channel id: {e}")))?;
        let port_id = PortId::from_str(request.get_ref().port_id.as_str())
            .map_err(|e| tonic::Status::aborted(format!("invalid port id: {e}")))?;

        let (next_recv_sequence, proof) = snapshot
            .get_with_proof(
                IBC_COMMITMENT_PREFIX
                    .apply_string(SeqRecvPath::new(&port_id, &channel_id).to_string())
                    .as_bytes()
                    .to_vec(),
            )
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get channel: {e}")))?;

        let next_recv_sequence = next_recv_sequence
            .map(|seq_bytes| u64::from_be_bytes(seq_bytes.try_into().expect("invalid sequence")))
            .ok_or_else(|| tonic::Status::aborted("next receive sequence not found"))?;

        Ok(tonic::Response::new(QueryNextSequenceReceiveResponse {
            next_sequence_receive: next_recv_sequence,
            proof: proof.encode_to_vec(),
            proof_height: Some(Height {
                revision_number: 0,
                revision_height: snapshot.version(),
            }),
        }))
    }

    /// NextSequenceSend returns the next send sequence for a given channel.
    async fn next_sequence_send(
        &self,
        request: tonic::Request<QueryNextSequenceSendRequest>,
    ) -> std::result::Result<tonic::Response<QueryNextSequenceSendResponse>, tonic::Status> {
        let snapshot = self.0.latest_snapshot();

        let channel_id = ChannelId::from_str(request.get_ref().channel_id.as_str())
            .map_err(|e| tonic::Status::aborted(format!("invalid channel id: {e}")))?;
        let port_id = PortId::from_str(request.get_ref().port_id.as_str())
            .map_err(|e| tonic::Status::aborted(format!("invalid port id: {e}")))?;

        let (next_send_sequence, proof) = snapshot
            .get_with_proof(
                IBC_COMMITMENT_PREFIX
                    .apply_string(SeqSendPath::new(&port_id, &channel_id).to_string())
                    .as_bytes()
                    .to_vec(),
            )
            .await
            .map_err(|e| tonic::Status::aborted(format!("couldn't get channel: {e}")))?;

        let next_send_sequence = next_send_sequence
            .map(|seq_bytes| u64::from_be_bytes(seq_bytes.try_into().expect("invalid sequence")))
            .ok_or_else(|| tonic::Status::aborted("next receive sequence not found"))?;

        Ok(tonic::Response::new(QueryNextSequenceSendResponse {
            next_sequence_send: next_send_sequence,
            proof: proof.encode_to_vec(),
            proof_height: Some(Height {
                revision_number: 0,
                revision_height: snapshot.version(),
            }),
        }))
    }
}
