//! Query commands for token factory on-chain state.

use anyhow::{Context, Result};
use colored_json::ToColoredJson;
use penumbra_sdk_proto::core::component::token_factory::v1::{
    query_service_client::QueryServiceClient as TokenFactoryQueryServiceClient,
    TokenMetadataRequest,
};

use crate::App;

/// Query token factory state.
#[derive(Debug, clap::Subcommand)]
pub enum TokenFactoryCmd {
    /// Get metadata for a factory token by its ID.
    Token {
        /// The token factory ID (hex-encoded 32 bytes).
        token_id: String,
    },
}

impl TokenFactoryCmd {
    pub fn offline(&self) -> bool {
        false
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            TokenFactoryCmd::Token { token_id } => {
                let token_id_bytes: [u8; 32] = hex::decode(token_id)
                    .context("invalid hex for token_id")?
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("token_id must be exactly 32 bytes"))?;

                let mut client = TokenFactoryQueryServiceClient::new(app.pd_channel().await?);

                let response = client
                    .token_metadata(TokenMetadataRequest {
                        token_id: token_id_bytes.to_vec(),
                    })
                    .await?
                    .into_inner();

                if let Some(metadata) = response.metadata {
                    println!("Token Metadata:");
                    let json = serde_json::to_string_pretty(&metadata)?;
                    println!("{}", json.to_colored_json_auto()?);
                } else {
                    println!("Token not found: {}", token_id);
                }

                Ok(())
            }
        }
    }
}
