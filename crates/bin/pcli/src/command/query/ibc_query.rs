use anyhow::{Context, Result};
use colored_json::ToColoredJson;
use ibc_proto::ibc::core::channel::v1::query_client::QueryClient as ChannelQueryClient;
use ibc_proto::ibc::core::channel::v1::QueryChannelsRequest;
use ibc_proto::ibc::core::client::v1::query_client::QueryClient as ClientQueryClient;
use ibc_proto::ibc::core::client::v1::{QueryClientStateRequest, QueryClientStatesRequest};
use ibc_proto::ibc::core::connection::v1::query_client::QueryClient as ConnectionQueryClient;
use ibc_proto::ibc::core::connection::v1::{QueryConnectionRequest, QueryConnectionsRequest};
use ibc_types::core::channel::ChannelEnd;
use ibc_types::lightclients::tendermint::client_state::ClientState as TendermintClientState;

use penumbra_proto::cnidarium::v1alpha1::{
    query_service_client::QueryServiceClient as StorageQueryServiceClient, KeyValueRequest,
};
use penumbra_proto::DomainType;

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

impl IbcCmd {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let mut client = StorageQueryServiceClient::new(app.pd_channel().await?);
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
                // TODO channel lookup should be updated to use the ibc query logic.
                // https://docs.rs/ibc-proto/0.36.1/ibc_proto/ibc/core/channel/v1/query_client/struct.QueryClient.html#method.channel
                let key =
                    format!("ibc-data/channelEnds/ports/{port}/channels/channel-{channel_id}");
                let value = client
                    .key_value(KeyValueRequest {
                        key,
                        ..Default::default()
                    })
                    .await
                    .context(format!(
                        "Error finding channel: {port}:channel-{channel_id}"
                    ))?
                    .into_inner()
                    .value
                    .context(format!("Channel {port}:channel-{channel_id} not found"))?;

                let channel = ChannelEnd::decode(value.value.as_ref())?;

                let channel_json = serde_json::to_string_pretty(&channel)?;
                println!("{}", channel_json.to_colored_json_auto()?);
            }
            IbcCmd::Channels {} => {
                let mut ibc_client = ChannelQueryClient::new(app.pd_channel().await?);
                let req = QueryChannelsRequest {
                    // TODO: support pagination
                    pagination: None,
                };
                let channels = ibc_client.channels(req).await?.into_inner().channels;
                let channels_json = serde_json::to_string_pretty(&channels)?;
                println!("{}", channels_json.to_colored_json_auto()?);
            }
        }

        Ok(())
    }
}
