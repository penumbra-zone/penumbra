use penumbra_proto::{
    client::{
        oblivious::oblivious_query_client::ObliviousQueryClient,
        specific::specific_query_client::SpecificQueryClient,
    },
    Protobuf,
};
use penumbra_transaction::Transaction;
use rand::Rng;
use tonic::transport::Channel;
use tracing::instrument;

use crate::Opt;

impl Opt {
    /// Submits a transaction to the network, returning `Ok` only when the remote
    /// node has accepted the transaction, and erroring otherwise.
    #[instrument(skip(self, transaction))]
    pub async fn submit_transaction(&self, transaction: &Transaction) -> Result<(), anyhow::Error> {
        tracing::info!("broadcasting transaction...");

        let client = reqwest::Client::new();
        let req_id: u8 = rand::thread_rng().gen();
        let rsp: serde_json::Value = client
            .post(format!(r#"http://{}:{}"#, self.node, self.rpc_port))
            .json(&serde_json::json!(
                {
                    "method": "broadcast_tx_sync",
                    "params": [&transaction.encode_to_vec()],
                    "id": req_id,
                }
            ))
            .send()
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

    /// Submits a transaction to the network, returning `Ok` as soon as the
    /// transaction has been submitted, rather than waiting to learn whether the
    /// node accepted it.
    #[instrument(skip(self, transaction))]
    pub async fn submit_transaction_unconfirmed(
        &self,
        transaction: &Transaction,
    ) -> Result<(), anyhow::Error> {
        tracing::info!("broadcasting transaction...");

        let client = reqwest::Client::new();
        let req_id: u8 = rand::thread_rng().gen();
        let rsp: serde_json::Value = client
            .post(format!(r#"http://{}:{}"#, self.node, self.rpc_port))
            .json(&serde_json::json!(
                {
                    "method": "broadcast_tx_async",
                    "params": [&transaction.encode_to_vec()],
                    "id": req_id,
                }
            ))
            .send()
            .await?
            .json()
            .await?;

        tracing::info!("{}", rsp);

        Ok(())
    }

    pub async fn specific_client(&self) -> Result<SpecificQueryClient<Channel>, anyhow::Error> {
        SpecificQueryClient::connect(format!("http://{}:{}", self.node, self.specific_query_port))
            .await
            .map_err(Into::into)
    }

    pub async fn oblivious_client(&self) -> Result<ObliviousQueryClient<Channel>, anyhow::Error> {
        ObliviousQueryClient::connect(format!(
            "http://{}:{}",
            self.node, self.oblivious_query_port
        ))
        .await
        .map_err(Into::into)
    }
}
