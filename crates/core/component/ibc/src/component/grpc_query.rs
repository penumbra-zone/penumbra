use async_trait::async_trait;
use ibc_proto::ibc::core::channel::v1::query_server::Query as ConsensusQuery;
use ibc_proto::ibc::core::channel::v1::{
    QueryChannelClientStateRequest, QueryChannelClientStateResponse,
    QueryChannelConsensusStateRequest, QueryChannelConsensusStateResponse, QueryChannelRequest,
    QueryChannelResponse, QueryChannelsRequest, QueryChannelsResponse,
    QueryConnectionChannelsRequest, QueryConnectionChannelsResponse,
    QueryNextSequenceReceiveRequest, QueryNextSequenceReceiveResponse,
    QueryPacketAcknowledgementRequest, QueryPacketAcknowledgementResponse,
    QueryPacketAcknowledgementsRequest, QueryPacketAcknowledgementsResponse,
    QueryPacketCommitmentRequest, QueryPacketCommitmentResponse, QueryPacketCommitmentsRequest,
    QueryPacketCommitmentsResponse, QueryPacketReceiptRequest, QueryPacketReceiptResponse,
    QueryUnreceivedAcksRequest, QueryUnreceivedAcksResponse, QueryUnreceivedPacketsRequest,
    QueryUnreceivedPacketsResponse,
};
use ibc_proto::ibc::core::client::v1::query_server::Query as ClientQuery;
use ibc_proto::ibc::core::client::v1::{
    QueryClientParamsRequest, QueryClientParamsResponse, QueryClientStateRequest,
    QueryClientStateResponse, QueryClientStatesRequest, QueryClientStatesResponse,
    QueryClientStatusRequest, QueryClientStatusResponse, QueryConsensusStateHeightsRequest,
    QueryConsensusStateHeightsResponse, QueryConsensusStateRequest, QueryConsensusStateResponse,
    QueryConsensusStatesRequest, QueryConsensusStatesResponse, QueryUpgradedClientStateRequest,
    QueryUpgradedClientStateResponse, QueryUpgradedConsensusStateRequest,
    QueryUpgradedConsensusStateResponse,
};
use ibc_proto::ibc::core::connection::v1::query_server::Query as ConnectionQuery;
use ibc_proto::ibc::core::connection::v1::{
    QueryClientConnectionsRequest, QueryClientConnectionsResponse,
    QueryConnectionClientStateRequest, QueryConnectionClientStateResponse,
    QueryConnectionConsensusStateRequest, QueryConnectionConsensusStateResponse,
    QueryConnectionRequest, QueryConnectionResponse, QueryConnectionsRequest,
    QueryConnectionsResponse,
};
use tonic::{Response, Status};
use tower::ServiceExt;

#[derive(Clone)]
pub struct IbcQuery();

#[async_trait]
impl ConnectionQuery for IbcQuery {
    /// Connection queries an IBC connection end.
    async fn connection(
        &self,
        request: tonic::Request<QueryConnectionRequest>,
    ) -> std::result::Result<tonic::Response<QueryConnectionResponse>, tonic::Status> {
        todo!()
    }
    /// Connections queries all the IBC connections of a chain.
    async fn connections(
        &self,
        request: tonic::Request<QueryConnectionsRequest>,
    ) -> std::result::Result<tonic::Response<QueryConnectionsResponse>, tonic::Status> {
        todo!()
    }
    /// ClientConnections queries the connection paths associated with a client
    /// state.
    async fn client_connections(
        &self,
        request: tonic::Request<QueryClientConnectionsRequest>,
    ) -> std::result::Result<tonic::Response<QueryClientConnectionsResponse>, tonic::Status> {
        todo!()
    }
    /// ConnectionClientState queries the client state associated with the
    /// connection.
    async fn connection_client_state(
        &self,
        request: tonic::Request<QueryConnectionClientStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryConnectionClientStateResponse>, tonic::Status>
    {
        todo!()
    }
    /// ConnectionConsensusState queries the consensus state associated with the
    /// connection.
    async fn connection_consensus_state(
        &self,
        request: tonic::Request<QueryConnectionConsensusStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryConnectionConsensusStateResponse>, tonic::Status>
    {
        todo!()
    }
}

#[async_trait]
impl ConsensusQuery for IbcQuery {
    /// Channel queries an IBC Channel.
    async fn channel(
        &self,
        request: tonic::Request<QueryChannelRequest>,
    ) -> std::result::Result<tonic::Response<QueryChannelResponse>, tonic::Status> {
        todo!()
    }
    /// Channels queries all the IBC channels of a chain.
    async fn channels(
        &self,
        request: tonic::Request<QueryChannelsRequest>,
    ) -> std::result::Result<tonic::Response<QueryChannelsResponse>, tonic::Status> {
        todo!()
    }
    /// ConnectionChannels queries all the channels associated with a connection
    /// end.
    async fn connection_channels(
        &self,
        request: tonic::Request<QueryConnectionChannelsRequest>,
    ) -> std::result::Result<tonic::Response<QueryConnectionChannelsResponse>, tonic::Status> {
        todo!()
    }
    /// ChannelClientState queries for the client state for the channel associated
    /// with the provided channel identifiers.
    async fn channel_client_state(
        &self,
        request: tonic::Request<QueryChannelClientStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryChannelClientStateResponse>, tonic::Status> {
        todo!()
    }
    /// ChannelConsensusState queries for the consensus state for the channel
    /// associated with the provided channel identifiers.
    async fn channel_consensus_state(
        &self,
        request: tonic::Request<QueryChannelConsensusStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryChannelConsensusStateResponse>, tonic::Status>
    {
        todo!()
    }
    /// PacketCommitment queries a stored packet commitment hash.
    async fn packet_commitment(
        &self,
        request: tonic::Request<QueryPacketCommitmentRequest>,
    ) -> std::result::Result<tonic::Response<QueryPacketCommitmentResponse>, tonic::Status> {
        todo!()
    }
    /// PacketCommitments returns all the packet commitments hashes associated
    /// with a channel.
    async fn packet_commitments(
        &self,
        request: tonic::Request<QueryPacketCommitmentsRequest>,
    ) -> std::result::Result<tonic::Response<QueryPacketCommitmentsResponse>, tonic::Status> {
        todo!()
    }
    /// PacketReceipt queries if a given packet sequence has been received on the
    /// queried chain
    async fn packet_receipt(
        &self,
        request: tonic::Request<QueryPacketReceiptRequest>,
    ) -> std::result::Result<tonic::Response<QueryPacketReceiptResponse>, tonic::Status> {
        todo!()
    }
    /// PacketAcknowledgement queries a stored packet acknowledgement hash.
    async fn packet_acknowledgement(
        &self,
        request: tonic::Request<QueryPacketAcknowledgementRequest>,
    ) -> std::result::Result<tonic::Response<QueryPacketAcknowledgementResponse>, tonic::Status>
    {
        todo!()
    }
    /// PacketAcknowledgements returns all the packet acknowledgements associated
    /// with a channel.
    async fn packet_acknowledgements(
        &self,
        request: tonic::Request<QueryPacketAcknowledgementsRequest>,
    ) -> std::result::Result<tonic::Response<QueryPacketAcknowledgementsResponse>, tonic::Status>
    {
        todo!()
    }
    /// UnreceivedPackets returns all the unreceived IBC packets associated with a
    /// channel and sequences.
    async fn unreceived_packets(
        &self,
        request: tonic::Request<QueryUnreceivedPacketsRequest>,
    ) -> std::result::Result<tonic::Response<QueryUnreceivedPacketsResponse>, tonic::Status> {
        todo!()
    }
    /// UnreceivedAcks returns all the unreceived IBC acknowledgements associated
    /// with a channel and sequences.
    async fn unreceived_acks(
        &self,
        request: tonic::Request<QueryUnreceivedAcksRequest>,
    ) -> std::result::Result<tonic::Response<QueryUnreceivedAcksResponse>, tonic::Status> {
        todo!()
    }
    /// NextSequenceReceive returns the next receive sequence for a given channel.
    async fn next_sequence_receive(
        &self,
        request: tonic::Request<QueryNextSequenceReceiveRequest>,
    ) -> std::result::Result<tonic::Response<QueryNextSequenceReceiveResponse>, tonic::Status> {
        todo!()
    }
}

#[async_trait]
impl ClientQuery for IbcQuery {
    async fn client_state(
        &self,
        request: tonic::Request<QueryClientStateRequest>,
    ) -> std::result::Result<Response<QueryClientStateResponse>, Status> {
        todo!()
    }
    /// ClientStates queries all the IBC light clients of a chain.
    async fn client_states(
        &self,
        request: tonic::Request<QueryClientStatesRequest>,
    ) -> std::result::Result<tonic::Response<QueryClientStatesResponse>, tonic::Status> {
        todo!()
    }
    /// ConsensusState queries a consensus state associated with a client state at
    /// a given height.
    async fn consensus_state(
        &self,
        request: tonic::Request<QueryConsensusStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryConsensusStateResponse>, tonic::Status> {
        todo!()
    }
    /// ConsensusStates queries all the consensus state associated with a given
    /// client.
    async fn consensus_states(
        &self,
        request: tonic::Request<QueryConsensusStatesRequest>,
    ) -> std::result::Result<tonic::Response<QueryConsensusStatesResponse>, tonic::Status> {
        todo!()
    }
    /// ConsensusStateHeights queries the height of every consensus states associated with a given client.
    async fn consensus_state_heights(
        &self,
        request: tonic::Request<QueryConsensusStateHeightsRequest>,
    ) -> std::result::Result<tonic::Response<QueryConsensusStateHeightsResponse>, tonic::Status>
    {
        todo!()
    }
    /// Status queries the status of an IBC client.
    async fn client_status(
        &self,
        request: tonic::Request<QueryClientStatusRequest>,
    ) -> std::result::Result<tonic::Response<QueryClientStatusResponse>, tonic::Status> {
        todo!()
    }
    /// ClientParams queries all parameters of the ibc client.
    async fn client_params(
        &self,
        request: tonic::Request<QueryClientParamsRequest>,
    ) -> std::result::Result<tonic::Response<QueryClientParamsResponse>, tonic::Status> {
        todo!()
    }
    /// UpgradedClientState queries an Upgraded IBC light client.
    async fn upgraded_client_state(
        &self,
        request: tonic::Request<QueryUpgradedClientStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryUpgradedClientStateResponse>, tonic::Status> {
        todo!()
    }
    /// UpgradedConsensusState queries an Upgraded IBC consensus state.
    async fn upgraded_consensus_state(
        &self,
        request: tonic::Request<QueryUpgradedConsensusStateRequest>,
    ) -> std::result::Result<tonic::Response<QueryUpgradedConsensusStateResponse>, tonic::Status>
    {
        todo!()
    }
}
