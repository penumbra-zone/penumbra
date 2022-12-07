use anyhow::Result;
use colored_json::prelude::*;
use penumbra_proto::{client::v1alpha1::GetTxRequest, Protobuf};
use penumbra_transaction::Transaction;

use crate::App;

/// Queries the chain for a transaction by hash.
#[derive(Debug, clap::Args)]
pub struct Tx {
    /// The hex-formatted transaction hash to query.
    hash: String,
}

impl Tx {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let mut client = app.tendermint_proxy_client().await?;

        let rsp = client
            .get_tx(GetTxRequest {
                hash: self.hash.to_string().as_bytes().to_vec(),
                prove: false,
            })
            .await?;

        let tx = Transaction::decode(&*rsp.into_inner().tx)?;
        let tx_json = serde_json::to_string_pretty(&tx)?;

        println!("{}", tx_json.to_colored_json_auto()?);

        Ok(())
    }
}
