use anyhow::{Context, Result};
use penumbra_sdk_proto::{util::tendermint_proxy::v1::GetTxRequest, DomainType};
use penumbra_sdk_transaction::Transaction;
use penumbra_sdk_view::{TransactionInfo, ViewClient};

use crate::App;

/// Queries the chain for a transaction by hash.
#[derive(Debug, clap::Args)]
pub struct TxCmd {
    /// The hex-formatted transaction hash to query.
    hash: String,
    /// If set, print the raw transaction view rather than a formatted table.
    #[clap(long)]
    raw: bool,
}

impl TxCmd {
    pub fn offline(&self) -> bool {
        false
    }
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let hash = self
            .hash
            // We have to convert to uppercase because `tendermint::Hash` only accepts uppercase :(
            .to_uppercase()
            .parse()
            .context("invalid transaction hash")?;

        // Retrieve Transaction from the view service first, or else the fullnode
        let tx_info = if let Ok(tx_info) = app.view().transaction_info_by_hash(hash).await {
            tx_info
        } else {
            if !self.raw {
                println!("Transaction not found in view service, fetching from fullnode...");
            } else {
                tracing::info!("Transaction not found in view service, fetching from fullnode...");
            }
            // Fall back to fetching from fullnode
            let mut client = app.tendermint_proxy_client().await?;
            let rsp = client
                .get_tx(GetTxRequest {
                    hash: hex::decode(self.hash.clone())?,
                    prove: false,
                })
                .await?;

            let rsp = rsp.into_inner();
            let tx = Transaction::decode(rsp.tx.as_slice())?;
            let txp = Default::default();
            let txv = tx.view_from_perspective(&txp);
            let summary = txv.summary();

            TransactionInfo {
                height: rsp.height,
                id: hash,
                transaction: tx,
                perspective: txp,
                view: txv,
                summary: summary,
            }
        };

        if self.raw {
            use colored_json::prelude::*;
            println!(
                "{}",
                serde_json::to_string_pretty(&tx_info.view)?.to_colored_json_auto()?
            );
        } else {
            use crate::transaction_view_ext::TransactionViewExt;
            tx_info.view.render_terminal();
        }

        Ok(())
    }
}
