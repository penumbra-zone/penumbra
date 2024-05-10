use std::time::SystemTime;

use anyhow::Result;
use colored_json::ToColoredJson;
use comfy_table::Table;
use ibc_proto::ibc::core::channel::v1::query_client::QueryClient as ChannelQueryClient;
use ibc_proto::ibc::core::channel::v1::{
    IdentifiedChannel, QueryChannelConsensusStateRequest, QueryChannelRequest, QueryChannelsRequest,
};
use ibc_proto::ibc::core::client::v1::query_client::QueryClient as ClientQueryClient;
use ibc_proto::ibc::core::client::v1::{QueryClientStateRequest, QueryClientStatesRequest};
use ibc_proto::ibc::core::connection::v1::query_client::QueryClient as ConnectionQueryClient;
use ibc_proto::ibc::core::connection::v1::{
    ConnectionEnd, QueryConnectionRequest, QueryConnectionsRequest,
};
use ibc_types::core::channel::channel::State;
use ibc_types::lightclients::tendermint::client_state::ClientState as TendermintClientState;
use ibc_types::lightclients::tendermint::consensus_state::ConsensusState as TendermintConsensusState;

use crate::App;

/// Queries the chain for IBC data. Results will be printed in JSON.
/// The singular subcommands require identifiers, whereas the plural subcommands
/// return all results.
#[derive(Debug, clap::Subcommand)]
pub enum IbcCmd {
    /// Queries for info on a specific IBC client.
    /// Requires client identifier string, e.g. "07-tendermint-0".
    Client { client_id: String },
    /// Queries for info on all IBC clients.
    Clients {},
    /// Queries for info on a specific IBC connection.
    /// Requires the numeric identifier for the connection, e.g. "0".
    Connection { connection_id: u64 },
    /// Queries for info on all IBC connections.
    Connections {},
    /// Queries for info on a specific IBC channel.
    /// Requires the numeric identifier for the channel, e.g. "0".
    Channel {
        /// The designation of the ICS port used for channel.
        /// In the context of IBC, this is usually "transfer".
        #[clap(long, default_value = "transfer")]
        port: String,

        /// The numeric id of the ICS channel to query for. This number was assigned
        /// during channel creation by a relaying client. Refer to the documentation
        /// for the relayer provider to understand which counterparty chain the
        /// channel id refers to.
        channel_id: u64,
    },
    /// Queries for info on all IBC channels.
    Channels {},
}

struct ChannelInfo {
    channel: IdentifiedChannel,
    connection: ConnectionEnd,
    client: TendermintClientState,
    consensus_state: TendermintConsensusState,
}

impl IbcCmd {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            IbcCmd::Client { client_id } => {
                let mut ibc_client = ClientQueryClient::new(app.pd_channel().await?);
                let req = QueryClientStateRequest {
                    client_id: client_id.to_string(),
                };
                let client_state = match ibc_client
                    .client_state(req)
                    .await?
                    .into_inner()
                    .client_state
                {
                    Some(c) => TendermintClientState::try_from(c)?,
                    None => {
                        anyhow::bail!("Client id not found: {}", client_id);
                    }
                };
                let client_state_json = serde_json::to_string_pretty(&client_state)?;
                println!("{}", client_state_json.to_colored_json_auto()?);
            }
            IbcCmd::Clients {} => {
                let mut ibc_client = ClientQueryClient::new(app.pd_channel().await?);
                let req = QueryClientStatesRequest {
                    // TODO: support pagination
                    pagination: None,
                };
                let client_states: Vec<_> = ibc_client
                    .client_states(req)
                    .await?
                    .into_inner()
                    .client_states
                    .into_iter()
                    .filter_map(|s| s.client_state)
                    .map(TendermintClientState::try_from)
                    .collect::<Result<Vec<_>, _>>()?;

                let clients_json = serde_json::to_string_pretty(&client_states)?;
                println!("{}", clients_json.to_colored_json_auto()?);
            }
            IbcCmd::Connection { connection_id } => {
                let mut ibc_client = ConnectionQueryClient::new(app.pd_channel().await?);
                let c = format!("connection-{}", connection_id);
                let req = QueryConnectionRequest {
                    connection_id: c.to_owned(),
                };
                let connection = ibc_client.connection(req).await?.into_inner().connection;
                if connection.is_none() {
                    anyhow::bail!("Could not find '{c}'");
                }
                let connection_json = serde_json::to_string_pretty(&connection)?;
                println!("{}", connection_json.to_colored_json_auto()?);
            }
            IbcCmd::Connections {} => {
                let mut ibc_client = ConnectionQueryClient::new(app.pd_channel().await?);
                let req = QueryConnectionsRequest {
                    // TODO: support pagination
                    pagination: None,
                };
                let connections = ibc_client.connections(req).await?.into_inner().connections;
                let connections_json = serde_json::to_string_pretty(&connections)?;
                println!("{}", connections_json.to_colored_json_auto()?);
            }
            IbcCmd::Channel { port, channel_id } => {
                let mut channel_client = ChannelQueryClient::new(app.pd_channel().await?);
                let mut connection_client = ConnectionQueryClient::new(app.pd_channel().await?);
                let mut client_client = ClientQueryClient::new(app.pd_channel().await?);

                let channel = channel_client
                    .channel(QueryChannelRequest {
                        port_id: port.to_string(),
                        channel_id: format!("channel-{channel_id}"),
                    })
                    .await?
                    .into_inner()
                    .channel
                    .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
                let connection = connection_client
                    .connection(QueryConnectionRequest {
                        connection_id: channel.connection_hops[0].clone(),
                    })
                    .await?
                    .into_inner()
                    .connection
                    .ok_or_else(|| anyhow::anyhow!("connection for channel not found"))?;
                let client_state = client_client
                    .client_state(QueryClientStateRequest {
                        client_id: connection.client_id.clone(),
                    })
                    .await?
                    .into_inner()
                    .client_state
                    .ok_or_else(|| anyhow::anyhow!("client state not found"))?;
                let client_state = TendermintClientState::try_from(client_state)?;
                let channel_consensus_state = channel_client
                    .channel_consensus_state(QueryChannelConsensusStateRequest {
                        port_id: port.to_string(),
                        channel_id: format!("channel-{}", channel_id),
                        revision_height: client_state.latest_height().revision_height,
                        revision_number: client_state.latest_height().revision_number,
                    })
                    .await?
                    .into_inner()
                    .consensus_state
                    .ok_or_else(|| anyhow::anyhow!("consensus state not found for channel"))?;

                let tendermint_consensus_state =
                    TendermintConsensusState::try_from(channel_consensus_state)?;

                let mut table = Table::new();
                table.set_header(vec![
                    "Channel ID",
                    "Port",
                    "Counterparty",
                    "Counterparty Channel ID",
                    "State",
                    "Client ID",
                    "Client Height",
                ]);
                let mut state_str = State::from_i32(channel.state)
                    .expect("invalid state value")
                    .to_string();

                let current_time: time::OffsetDateTime = SystemTime::now().into();
                let current_time_tm: tendermint::Time = current_time.try_into()?;

                let time_elapsed =
                    current_time_tm.duration_since(tendermint_consensus_state.timestamp)?;
                if client_state.expired(time_elapsed) {
                    state_str = "CLIENT EXPIRED".to_string();
                }
                table.add_row(vec![
                    channel_id.to_string(),
                    port.to_string(),
                    client_state.chain_id.to_string(),
                    channel
                        .counterparty
                        .ok_or_else(|| anyhow::anyhow!("counterparty not found"))?
                        .channel_id
                        .to_string(),
                    state_str,
                    connection.client_id.to_string(),
                    client_state.latest_height.to_string(),
                ]);

                println!("{table}")
            }
            IbcCmd::Channels {} => {
                let mut channel_client = ChannelQueryClient::new(app.pd_channel().await?);
                let mut connection_client = ConnectionQueryClient::new(app.pd_channel().await?);
                let mut client_client = ClientQueryClient::new(app.pd_channel().await?);

                let req = QueryChannelsRequest {
                    // TODO: support pagination
                    pagination: None,
                };

                let mut channel_infos = vec![];
                let channels = channel_client.channels(req).await?.into_inner().channels;
                for channel in channels {
                    let connection = connection_client
                        .connection(QueryConnectionRequest {
                            connection_id: channel.connection_hops[0].clone(),
                        })
                        .await?
                        .into_inner()
                        .connection
                        .ok_or_else(|| anyhow::anyhow!("connection for channel not found"))?;
                    let client_state = client_client
                        .client_state(QueryClientStateRequest {
                            client_id: connection.client_id.clone(),
                        })
                        .await?
                        .into_inner()
                        .client_state
                        .ok_or_else(|| anyhow::anyhow!("client state not found"))?;
                    let client_state = TendermintClientState::try_from(client_state.clone())?;
                    let channel_consensus_state = channel_client
                        .channel_consensus_state(QueryChannelConsensusStateRequest {
                            port_id: channel.clone().port_id.to_string(),
                            channel_id: channel.clone().channel_id,
                            revision_height: client_state.latest_height().revision_height,
                            revision_number: client_state.latest_height().revision_number,
                        })
                        .await?
                        .into_inner()
                        .consensus_state
                        .ok_or_else(|| anyhow::anyhow!("consensus state not found for channel"))?;

                    let tendermint_consensus_state =
                        TendermintConsensusState::try_from(channel_consensus_state)?;

                    channel_infos.push(ChannelInfo {
                        channel,
                        connection,
                        client: client_state,
                        consensus_state: tendermint_consensus_state,
                    });
                }

                let mut table = Table::new();
                table.set_header(vec![
                    "Channel ID",
                    "Port",
                    "Counterparty",
                    "Counterparty Channel ID",
                    "State",
                    "Client ID",
                    "Client Height",
                ]);

                for info in channel_infos {
                    let mut state_str = State::from_i32(info.channel.state)
                        .expect("invalid state value")
                        .to_string();
                    let current_time: time::OffsetDateTime = SystemTime::now().into();
                    let current_time_tm: tendermint::Time = current_time.try_into()?;

                    let time_elapsed =
                        current_time_tm.duration_since(info.consensus_state.timestamp)?;
                    if info.client.expired(time_elapsed) {
                        state_str = "CLIENT EXPIRED".to_string();
                    }
                    table.add_row(vec![
                        info.channel.channel_id.to_string(),
                        info.channel.port_id,
                        info.client.chain_id.to_string(),
                        info.channel
                            .counterparty
                            .ok_or_else(|| anyhow::anyhow!("counterparty not found"))?
                            .channel_id
                            .to_string(),
                        state_str,
                        info.connection.client_id.to_string(),
                        info.client.latest_height.to_string(),
                    ]);
                }

                println!("{table}")
            }
        }

        Ok(())
    }
}
