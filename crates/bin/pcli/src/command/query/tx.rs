use anyhow::Result;
use colored_json::prelude::*;
use penumbra_proto::{client::v1alpha1::GetTxRequest, DomainType};
use penumbra_transaction::Transaction;

use crate::App;

use super::OutputFormat;

/// Queries the chain for a transaction by hash.
#[derive(Debug, clap::Args)]
pub struct Tx {
    /// The format to output the transaction in.
    #[clap(short, long, value_enum, default_value_t)]
    output: OutputFormat,
    /// The hex-formatted transaction hash to query.
    hash: String,
}

impl Tx {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let mut client = app.tendermint_proxy_client().await?;

        let rsp = client
            .get_tx(GetTxRequest {
                hash: hex::decode(self.hash.clone())?,
                prove: false,
            })
            .await?;

        let rsp = rsp.into_inner();
        let tx = Transaction::decode(rsp.tx.as_slice())?;

        match self.output {
            OutputFormat::Json => {
                let tx_json = serde_json::to_string_pretty(&tx)?;
                println!("{}", tx_json.to_colored_json_auto()?);
            }
            OutputFormat::Base64 => {
                use base64::{display::Base64Display, engine::general_purpose::STANDARD};
                println!("{}", Base64Display::new(&rsp.tx, &STANDARD));
            }
        }

        Ok(())
    }
}
