use anyhow::Result;
use colored_json::prelude::*;
use penumbra_proto::Protobuf;
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
        use tendermint_rpc::{Client, HttpClient};

        // generic bounds on HttpClient::new are not well-constructed, so we have to
        // render the URL as a String, then borrow it, then re-parse the borrowed &str
        let client = HttpClient::new(app.pd_url.to_string().as_ref()).unwrap();

        let rsp = client.tx(self.hash.parse()?, false).await?;

        let tx = Transaction::decode(rsp.tx.as_bytes())?;
        let tx_json = serde_json::to_string_pretty(&tx)?;

        println!("{}", tx_json.to_colored_json_auto()?);
        println!("\nresult code: {:?}", rsp.tx_result.code);

        Ok(())
    }
}
