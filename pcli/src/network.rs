use penumbra_proto::{
    light_wallet::light_wallet_client::LightWalletClient,
    thin_wallet::thin_wallet_client::ThinWalletClient, Protobuf,
};
use penumbra_transaction::Transaction;
use tonic::transport::Channel;
use tracing::instrument;

use crate::Opt;

impl Opt {
    /// Submits a transaction to the network, returning `Ok` only when the remote
    /// node has accepted the transaction, and erroring otherwise.
    #[instrument(skip(self, transaction))]
    pub async fn submit_transaction(&self, transaction: &Transaction) -> Result<(), anyhow::Error> {
        tracing::info!("broadcasting transaction...");

        let rsp: serde_json::Value = reqwest::get(format!(
            r#"http://{}:{}/broadcast_tx_sync?tx=0x{}"#,
            self.node,
            self.rpc_port,
            hex::encode(&transaction.encode_to_vec())
        ))
        .await?
        .json()
        .await?;

        tracing::info!("{}", rsp);

        // Sometimes the result is in a result key, and sometimes it's bare? (??)
        let result = rsp.get("result").unwrap_or(&rsp);

        let code = result
            .get("code")
            .and_then(|c| c.as_i64())
            .ok_or_else(|| anyhow::anyhow!("could not parse JSON response"))?;

        if code == 0 {
            Ok(())
        } else {
            let log = result
                .get("log")
                .and_then(|l| l.as_str())
                .ok_or_else(|| anyhow::anyhow!("could not parse JSON response"))?;

            Err(anyhow::anyhow!(
                "Error submitting transaction: code {}, log: {}",
                code,
                log
            ))
        }
    }

    pub async fn thin_wallet_client(&self) -> Result<ThinWalletClient<Channel>, anyhow::Error> {
        ThinWalletClient::connect(format!("http://{}:{}", self.node, self.thin_wallet_port))
            .await
            .map_err(Into::into)
    }

    pub async fn light_wallet_client(&self) -> Result<LightWalletClient<Channel>, anyhow::Error> {
        LightWalletClient::connect(format!("http://{}:{}", self.node, self.light_wallet_port))
            .await
            .map_err(Into::into)
    }
}
