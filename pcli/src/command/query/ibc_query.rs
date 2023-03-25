use anyhow::Result;
use colored_json::prelude::*;
use ibc::core::ics03_connection::connection::ConnectionEnd;
use penumbra_proto::client::v1alpha1::KeyValueRequest;
use penumbra_proto::DomainType;

use crate::App;

/// Queries the chain for IBC data
#[derive(Debug, clap::Subcommand)]
pub enum IbcCmd {
    /// Queries for connection info
    Connection { connection_id: String },
}

impl IbcCmd {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let mut client = app.specific_client().await?;
        match self {
            IbcCmd::Connection { connection_id } => {
                let key = format!("connections/{connection_id}");
                let value = client
                    .key_value(KeyValueRequest {
                        key,
                        ..Default::default()
                    })
                    .await?
                    .into_inner()
                    .value;

                let connection = ConnectionEnd::decode(value.as_ref())?;
                let connection_json = serde_json::to_string_pretty(&connection)?;
                println!("{}", connection_json.to_colored_json_auto()?);
            }
        }

        Ok(())
    }
}
