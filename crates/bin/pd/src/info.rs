//! Logic for enabling `pd` to interact with chain state.
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    vec,
};

use anyhow::Context as _;
use futures::FutureExt;
use ibc_proto::ibc::core::{
    channel::v1::{
        PacketState, QueryChannelsRequest, QueryChannelsResponse,
        QueryPacketAcknowledgementsRequest, QueryPacketAcknowledgementsResponse,
        QueryPacketCommitmentsRequest, QueryPacketCommitmentsResponse, QueryUnreceivedAcksRequest,
        QueryUnreceivedAcksResponse, QueryUnreceivedPacketsRequest, QueryUnreceivedPacketsResponse,
    },
    client::v1::{IdentifiedClientState, QueryClientStatesRequest, QueryClientStatesResponse},
};
use ibc_proto::ibc::core::{
    channel::v1::{QueryConnectionChannelsRequest, QueryConnectionChannelsResponse},
    connection::v1::{QueryConnectionsRequest, QueryConnectionsResponse},
};
use ibc_types2::core::channel::IdentifiedChannelEnd;
use ibc_types2::core::channel::{ChannelId, PortId};
use ibc_types2::core::client::ClientId;
use ibc_types2::core::connection::ConnectionId;
use ibc_types2::core::connection::IdentifiedConnectionEnd;
use penumbra_chain::component::AppHashRead;
use penumbra_ibc::component::ChannelStateReadExt as _;
use penumbra_ibc::component::ClientStateReadExt as _;
use penumbra_ibc::component::ConnectionStateReadExt as _;
use penumbra_storage::Storage;
use prost::Message;
use std::str::FromStr;
use tendermint::abci::{self, response::Echo, InfoRequest, InfoResponse};
use tower_abci::BoxError;
use tracing::Instrument;

use penumbra_tower_trace::RequestExt;

mod oblivious;
mod specific;

const ABCI_INFO_VERSION: &str = env!("VERGEN_GIT_SEMVER");
const APP_VERSION: u64 = 1;

/// Implements service traits for Tonic gRPC services.
///
/// The fields of this struct are the configuration and data
/// necessary to the gRPC services.
#[derive(Clone, Debug)]
pub struct Info {
    /// Storage interface for retrieving chain state.
    storage: Storage,
    // height_rx: watch::Receiver<block::Height>,
}

impl Info {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    async fn info(&self, info: abci::request::Info) -> Result<abci::response::Info, anyhow::Error> {
        let state = self.storage.latest_snapshot();
        tracing::info!(?info, version = ?state.version());

        let last_block_height = match state.version() {
            // When the state is uninitialized, state.version() will return -1 (u64::MAX),
            // which could confuse Tendermint, so special-case this value to 0.
            u64::MAX => 0,
            v => v,
        }
        .try_into()
        .unwrap();

        let last_block_app_hash = state.app_hash().await?.0.to_vec().try_into()?;

        Ok(abci::response::Info {
            data: "penumbra".to_string(),
            version: ABCI_INFO_VERSION.to_string(),
            app_version: APP_VERSION,
            last_block_height,
            last_block_app_hash,
        })
    }

    async fn get_snapshot_for_height(
        &self,
        height: u64,
    ) -> anyhow::Result<(penumbra_storage::Snapshot, u64)> {
        match height {
            // ABCI docs say:
            //
            // The default `0` returns data for the latest committed block. Note that
            // this is the height of the block containing the application's Merkle root
            // hash, which represents the state as it was after committing the block at
            // `height - 1`.
            //
            // Try to do this behavior by querying at height = latest-1.
            0 => {
                let height = self.storage.latest_snapshot().version() - 1;
                let snapshot = self
                    .storage
                    .snapshot(height)
                    .ok_or_else(|| anyhow::anyhow!("no snapshot of height {height}"))?;

                Ok((snapshot, height))
            }
            height => {
                let snapshot = self
                    .storage
                    .snapshot(height)
                    .ok_or_else(|| anyhow::anyhow!("no snapshot of height {height}"))?;

                Ok((snapshot, height))
            }
        }
    }

    async fn query(
        &self,
        query: abci::request::Query,
    ) -> Result<abci::response::Query, anyhow::Error> {
        tracing::info!(?query);

        match query.path.as_str() {
            "state/key" => {
                let (snapshot, height) = self
                    .get_snapshot_for_height(u64::from(query.height))
                    .await?;

                let key = hex::decode(&query.data).unwrap_or_else(|_| query.data.to_vec());

                let Some((value, proof_ops)) = snapshot.get_with_proof_to_apphash_tm(key).await? else {
                    anyhow::bail!("key not found")
                };

                Ok(abci::response::Query {
                    code: 0.into(),
                    key: query.data,
                    log: "".to_string(),
                    value: value.into(),
                    proof: Some(proof_ops),
                    height: height.try_into().unwrap(),
                    codespace: "".to_string(),
                    info: "".to_string(),
                    index: 0,
                })
            }
            "/ibc.core.connection.v1.Query/Connections" => {
                let (snapshot, height) = self
                    .get_snapshot_for_height(u64::from(query.height))
                    .await?;

                // TODO: handle request.pagination
                let _request = QueryConnectionsRequest::decode(query.data.clone())
                    .context("failed to decode QueryConnectionsRequest")?;

                let connection_counter = snapshot.get_connection_counter().await?;

                let mut connections = vec![];
                for conn_idx in 0..connection_counter.0 {
                    let conn_id = ConnectionId(format!("connection-{}", conn_idx));
                    let connection = snapshot.get_connection(&conn_id).await?.unwrap();
                    let id_conn = IdentifiedConnectionEnd {
                        connection_id: conn_id,
                        connection_end: connection,
                    };
                    connections.push(id_conn.into());
                }

                let res_value = QueryConnectionsResponse {
                    connections,
                    pagination: None,
                    height: None,
                }
                .encode_to_vec();

                Ok(abci::response::Query {
                    code: 0.into(),
                    key: query.data,
                    log: "".to_string(),
                    value: res_value.into(),
                    proof: None,
                    height: height.try_into().unwrap(),
                    codespace: "".to_string(),
                    info: "".to_string(),
                    index: 0,
                })
            }
            "/ibc.core.channel.v1.Query/Channels" => {
                let (snapshot, height) = self
                    .get_snapshot_for_height(u64::from(query.height))
                    .await?;

                // TODO: handle request.pagination
                let _request = QueryChannelsRequest::decode(query.data.clone())
                    .context("failed to decode QueryConnectionsRequest")?;

                let channel_counter = snapshot.get_channel_counter().await?;

                let mut channels = vec![];
                for chan_idx in 0..channel_counter {
                    let chan_id = ChannelId(format!("channel-{}", chan_idx));
                    let channel = snapshot
                        .get_channel(&chan_id, &PortId::transfer())
                        .await?
                        .unwrap();
                    let id_chan = IdentifiedChannelEnd {
                        channel_id: chan_id,
                        port_id: PortId::transfer(),
                        channel_end: channel,
                    };
                    channels.push(id_chan.into());
                }

                let res_value = QueryChannelsResponse {
                    channels,
                    pagination: None,
                    height: None,
                }
                .encode_to_vec();

                Ok(abci::response::Query {
                    code: 0.into(),
                    key: query.data,
                    log: "".to_string(),
                    value: res_value.into(),
                    proof: None,
                    height: height.try_into().unwrap(),
                    codespace: "".to_string(),
                    info: "".to_string(),
                    index: 0,
                })
            }
            "/ibc.core.channel.v1.Query/ConnectionChannels" => {
                let (snapshot, height) = self
                    .get_snapshot_for_height(u64::from(query.height))
                    .await?;

                let request = QueryConnectionChannelsRequest::decode(query.data.clone())
                    .context("failed to decode QueryConnectionChannelsRequest")?;

                let connection_id: ConnectionId = ConnectionId::from_str(&request.connection)
                    .context("couldn't decode connection id from request")?;

                // look up all of the channels for this connection
                let channel_counter = snapshot.get_channel_counter().await?;

                let mut channels = vec![];
                for chan_idx in 0..channel_counter {
                    let chan_id = ChannelId(format!("channel-{}", chan_idx));
                    let channel = snapshot
                        .get_channel(&chan_id, &PortId::transfer())
                        .await?
                        .unwrap();
                    if channel.connection_hops.contains(&connection_id) {
                        let id_chan = IdentifiedChannelEnd {
                            channel_id: chan_id,
                            port_id: PortId::transfer(),
                            channel_end: channel,
                        };
                        channels.push(id_chan.into());
                    }
                }

                let res_value = QueryConnectionChannelsResponse {
                    channels,
                    pagination: None,
                    height: None,
                }
                .encode_to_vec();

                Ok(abci::response::Query {
                    code: 0.into(),
                    key: query.data,
                    log: "".to_string(),
                    value: res_value.into(),
                    proof: None,
                    height: height.try_into().unwrap(),
                    codespace: "".to_string(),
                    info: "".to_string(),
                    index: 0,
                })
            }
            "/ibc.core.client.v1.Query/ClientStates" => {
                let (snapshot, height) = self
                    .get_snapshot_for_height(u64::from(query.height))
                    .await?;

                // TODO; handle request.pagination
                let _request = QueryClientStatesRequest::decode(query.data.clone())
                    .context("failed to decode QueryClientStatesRequest")?;

                let client_counter = snapshot.client_counter().await?.0;

                let mut client_states = vec![];
                for client_idx in 0..client_counter {
                    // NOTE: currently, we only look up tendermint clients, because we only support tendermint clients.
                    let client_id =
                        ClientId::from_str(format!("07-tendermint-{}", client_idx).as_str())?;
                    let client_state = snapshot.get_client_state(&client_id).await;
                    let id_client = IdentifiedClientState {
                        client_id: client_id.to_string(),
                        client_state: client_state.ok().map(|state| state.into()), // send None if we couldn't find the client state
                    };
                    client_states.push(id_client);
                }

                let res_value = QueryClientStatesResponse {
                    client_states,
                    pagination: None,
                }
                .encode_to_vec();

                Ok(abci::response::Query {
                    code: 0.into(),
                    key: query.data,
                    log: "".to_string(),
                    value: res_value.into(),
                    proof: None,
                    height: height.try_into().unwrap(),
                    codespace: "".to_string(),
                    info: "".to_string(),
                    index: 0,
                })
            }
            "/ibc.core.channel.v1.Query/PacketCommitments" => {
                let (snapshot, height) = self
                    .get_snapshot_for_height(u64::from(query.height))
                    .await?;

                let request = QueryPacketCommitmentsRequest::decode(query.data.clone())
                    .context("failed to decode QueryPacketCommitmentsRequest")?;

                let chan_id: ChannelId =
                    ChannelId::from_str(&request.channel_id).context("invalid channel id")?;
                let port_id: PortId =
                    PortId::from_str(&request.port_id).context("invalid port id")?;

                let mut commitment_states = vec![];
                let commitment_counter = snapshot.get_send_sequence(&chan_id, &port_id).await?;

                for commitment_idx in 0..commitment_counter {
                    let commitment = snapshot
                        .get_packet_commitment_by_id(&chan_id, &port_id, commitment_idx)
                        .await?
                        .ok_or(anyhow::anyhow!("couldnt find commitment"))?;

                    let commitment_state = PacketState {
                        port_id: request.port_id.clone(),
                        channel_id: request.channel_id.clone(),
                        sequence: commitment_idx,
                        data: commitment.clone(),
                    };

                    commitment_states.push(commitment_state);
                }

                let res_value = QueryPacketCommitmentsResponse {
                    commitments: commitment_states,
                    pagination: None,
                    height: None,
                }
                .encode_to_vec();

                Ok(abci::response::Query {
                    code: 0.into(),
                    key: query.data,
                    log: "".to_string(),
                    value: res_value.into(),
                    proof: None,
                    height: height.try_into().unwrap(),
                    codespace: "".to_string(),
                    info: "".to_string(),
                    index: 0,
                })
            }
            "/ibc.core.channel.v1.Query/PacketAcknowledgements" => {
                let (snapshot, height) = self
                    .get_snapshot_for_height(u64::from(query.height))
                    .await?;

                let request = QueryPacketAcknowledgementsRequest::decode(query.data.clone())
                    .context("failed to decode QueryPacketAcknowledgementsRequest")?;

                let chan_id: ChannelId =
                    ChannelId::from_str(&request.channel_id).context("invalid channel id")?;
                let port_id: PortId =
                    PortId::from_str(&request.port_id).context("invalid port id")?;

                let ack_counter = snapshot.get_ack_sequence(&chan_id, &port_id).await?;

                let mut acks = vec![];
                for ack_idx in 0..ack_counter {
                    let ack = snapshot
                        .get_packet_acknowledgement(&port_id, &chan_id, ack_idx)
                        .await?
                        .ok_or(anyhow::anyhow!("couldn't find ack"))?;

                    let ack_state = PacketState {
                        port_id: request.port_id.clone(),
                        channel_id: request.channel_id.clone(),
                        sequence: ack_idx,
                        data: ack.clone(),
                    };

                    acks.push(ack_state);
                }

                let res_value = QueryPacketAcknowledgementsResponse {
                    acknowledgements: acks,
                    pagination: None,
                    height: None,
                }
                .encode_to_vec();

                Ok(abci::response::Query {
                    code: 0.into(),
                    key: query.data,
                    log: "".to_string(),
                    value: res_value.into(),
                    proof: None,
                    height: height.try_into().unwrap(),
                    codespace: "".to_string(),
                    info: "".to_string(),
                    index: 0,
                })
            }

            // docstring from ibc-go:
            //
            // UnreceivedPackets implements the Query/UnreceivedPackets gRPC method. Given
            // a list of counterparty packet commitments, the querier checks if the packet
            // has already been received by checking if a receipt exists on this
            // chain for the packet sequence. All packets that haven't been received yet
            // are returned in the response
            // Usage: To use this method correctly, first query all packet commitments on
            // the sending chain using the Query/PacketCommitments gRPC method.
            // Then input the returned sequences into the QueryUnreceivedPacketsRequest
            // and send the request to this Query/UnreceivedPackets on the **receiving**
            // chain. This gRPC method will then return the list of packet sequences that
            // are yet to be received on the receiving chain.
            //
            // NOTE: The querier makes the assumption that the provided list of packet
            // commitments is correct and will not function properly if the list
            // is not up to date. Ideally the query height should equal the latest height
            // on the counterparty's client which represents this chain
            "/ibc.core.channel.v1.Query/UnreceivedPackets" => {
                let (snapshot, height) = self
                    .get_snapshot_for_height(u64::from(query.height))
                    .await?;

                let request = QueryUnreceivedPacketsRequest::decode(query.data.clone())
                    .context("failed to decode QueryUnreceivedPacketsRequest")?;

                let chan_id: ChannelId =
                    ChannelId::from_str(&request.channel_id).context("invalid channel id")?;
                let port_id: PortId =
                    PortId::from_str(&request.port_id).context("invalid port id")?;

                let mut unreceived_seqs = vec![];

                for seq in request.packet_commitment_sequences {
                    if seq == 0 {
                        return Err(anyhow::anyhow!("packet sequence {} cannot be 0", seq));
                    }

                    if snapshot
                        .seen_packet_by_channel(&chan_id, &port_id, seq)
                        .await?
                    {
                        unreceived_seqs.push(seq);
                    }
                }

                let res_value = QueryUnreceivedPacketsResponse {
                    sequences: unreceived_seqs,
                    height: None,
                }
                .encode_to_vec();

                Ok(abci::response::Query {
                    code: 0.into(),
                    key: query.data,
                    log: "".to_string(),
                    value: res_value.into(),
                    proof: None,
                    height: height.try_into().unwrap(),
                    codespace: "".to_string(),
                    info: "".to_string(),
                    index: 0,
                })
            }

            "/ibc.core.channel.v1.Query/UnreceivedAcks" => {
                let (snapshot, height) = self
                    .get_snapshot_for_height(u64::from(query.height))
                    .await?;

                let request = QueryUnreceivedAcksRequest::decode(query.data.clone())
                    .context("failed to decode QueryUnreceivedAcksRequest")?;

                let chan_id: ChannelId =
                    ChannelId::from_str(&request.channel_id).context("invalid channel id")?;
                let port_id: PortId =
                    PortId::from_str(&request.port_id).context("invalid port id")?;

                let mut unreceived_seqs = vec![];

                for seq in request.packet_ack_sequences {
                    if seq == 0 {
                        return Err(anyhow::anyhow!("packet sequence {} cannot be 0", seq));
                    }

                    if snapshot
                        .get_packet_commitment_by_id(&chan_id, &port_id, seq)
                        .await?
                        .is_some()
                    {
                        unreceived_seqs.push(seq);
                    }
                }

                let res_value = QueryUnreceivedAcksResponse {
                    sequences: unreceived_seqs,
                    height: None,
                }
                .encode_to_vec();

                Ok(abci::response::Query {
                    code: 0.into(),
                    key: query.data,
                    log: "".to_string(),
                    value: res_value.into(),
                    proof: None,
                    height: height.try_into().unwrap(),
                    codespace: "".to_string(),
                    info: "".to_string(),
                    index: 0,
                })
            }

            _ => Err(anyhow::anyhow!(
                "requested unrecognized path in ABCI query: {}",
                query.path
            )),
        }
    }
}

impl tower_service::Service<InfoRequest> for Info {
    type Response = InfoResponse;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<InfoResponse, BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: InfoRequest) -> Self::Future {
        let span = req.create_span();
        let self2 = self.clone();

        async move {
            match req {
                InfoRequest::Info(info) => self2
                    .info(info)
                    .await
                    .map(InfoResponse::Info)
                    .map_err(Into::into),
                InfoRequest::Query(query) => match self2.query(query).await {
                    Ok(rsp) => Ok(InfoResponse::Query(rsp)),
                    Err(e) => Ok(InfoResponse::Query(abci::response::Query {
                        code: 1.into(),
                        log: e.to_string(),
                        ..Default::default()
                    })),
                },
                InfoRequest::Echo(echo) => Ok(InfoResponse::Echo(Echo {
                    message: echo.message,
                })),
                InfoRequest::SetOption(_) => todo!(),
            }
        }
        .instrument(span)
        .boxed()
    }
}
