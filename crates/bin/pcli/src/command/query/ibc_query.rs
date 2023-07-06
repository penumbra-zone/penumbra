use anyhow::{Context, Result};
use colored_json::ToColoredJson;
use ibc_types::core::channel::ChannelEnd;
use ibc_types::core::connection::ConnectionEnd;
use ibc_types::lightclients::tendermint::client_state::ClientState as TendermintClientState;

use penumbra_proto::client::v1alpha1::KeyValueRequest;
use penumbra_proto::DomainType;

use crate::App;

/// Queries the chain for IBC data
#[derive(Debug, clap::Subcommand)]
pub enum IbcCmd {
    /// Queries for client info
    Client { client_id: String },
    /// Queries for connection info
    Connection { connection_id: String },
    /// Queries for channel info
    Channel { port: String, channel_id: String },
}

impl IbcCmd {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let mut client = app.specific_client().await?;
        match self {
            IbcCmd::Client { client_id } => {
                let key = format!("clients/{client_id}/clientState");
                let value = client
                    .key_value(KeyValueRequest {
                        key,
                        ..Default::default()
                    })
                    .await
                    .context(format!("Error finding client {client_id}"))?
                    .into_inner()
                    .value
                    .context("Client {client_id} not found")?;

                let client_state = TendermintClientState::decode(value.value.as_ref())?;
                let client_state_json = serde_json::to_string_pretty(&client_state)?;
                println!("{}", client_state_json.to_colored_json_auto()?);
            }
            IbcCmd::Connection { connection_id } => {
                let key = format!("connections/{connection_id}");
                let value = client
                    .key_value(KeyValueRequest {
                        key,
                        ..Default::default()
                    })
                    .await
                    .context(format!("error finding {connection_id}"))?
                    .into_inner()
                    .value
                    .context(format!("Connection {connection_id} not found"))?;

                let connection = ConnectionEnd::decode(value.value.as_ref())?;
                let connection_json = serde_json::to_string_pretty(&connection)?;
                println!("{}", connection_json.to_colored_json_auto()?);
            }
            IbcCmd::Channel { port, channel_id } => {
                let key = format!("channelEnds/ports/{port}/channels/{channel_id}");
                let value = client
                    .key_value(KeyValueRequest {
                        key,
                        ..Default::default()
                    })
                    .await
                    .context(format!("Error finding channel: {port}:{channel_id}"))?
                    .into_inner()
                    .value
                    .context(format!("Channel {port}:{channel_id} not found"))?;

                let channel = ChannelEnd::decode(value.value.as_ref())?;
                let channel_json = serde_json::to_string_pretty(&channel)?;
                println!("{}", channel_json.to_colored_json_auto()?);
            }
        }

        Ok(())
    }
}
