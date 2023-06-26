use anyhow::{Context, Result};
use ibc_types2::core::channel::ChannelEnd;
use ibc_types2::core::connection::ConnectionEnd;
use ibc_types2::lightclients::tendermint::client_state::ClientState as TendermintClientState;

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
                /*
                let client_state_json = serde_json::to_string_pretty(&client_state)?;
                restore this once we have serde impls
                */
                println!("{:?}", client_state);
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
                // restore this once we have serde impls
                // let connection_json = serde_json::to_string_pretty(&connection)?;
                println!("{:?}", connection);
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
                // restore this once we have serde impls
                // let connection_json = serde_json::to_string_pretty(&connection)?;
                println!("{:?}", channel);
            }
        }

        Ok(())
    }
}
