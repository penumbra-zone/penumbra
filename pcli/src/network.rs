use anyhow::{Context as _, Result};
use penumbra_component::ActionHandler;
use penumbra_crypto::Nullifier;
use penumbra_proto::{
    client::v1alpha1::{
        oblivious_query_service_client::ObliviousQueryServiceClient,
        specific_query_service_client::SpecificQueryServiceClient,
        tendermint_proxy::service_client::ServiceClient as TendermintServiceClient,
    },
    Protobuf,
};
use penumbra_transaction::{plan::TransactionPlan, Transaction};
use penumbra_view::ViewClient;
use rand::Rng;
use rand_core::OsRng;
use std::future::Future;
use tonic::transport::Channel;
use tracing::instrument;

use crate::App;

impl App {
    pub async fn build_and_submit_transaction(
        &mut self,
        plan: TransactionPlan,
    ) -> anyhow::Result<()> {
        let await_detection_of_nullifier = plan.spend_plans().next().map(|spend_plan| {
            // If we spend at least one note, then we should await detecting it (it doesn't matter
            // which nullifier we wait for, since any will work)
            self.fvk
                .derive_nullifier(spend_plan.position, &spend_plan.note.commit())
        });

        let tx = self.build_transaction(plan).await?;

        self.submit_transaction(&tx, await_detection_of_nullifier)
            .await
    }

    pub fn build_transaction(
        &mut self,
        plan: TransactionPlan,
    ) -> impl Future<Output = Result<Transaction>> + '_ {
        penumbra_wallet::build_transaction(
            &self.fvk,
            self.view.as_mut().unwrap(),
            &mut self.custody,
            OsRng,
            plan,
        )
    }

    /// Submits a transaction to the network.
    ///
    /// # Returns
    ///
    /// - if `await_detection_of_nullifier` is `Some`, returns `Ok` after the specified note has been detected by the view service, implying transaction finality.
    /// - if `await_detection_of_nullifier` is `None`, returns `Ok` after the transaction has been accepted by the node it was sent to.
    #[instrument(skip(self, transaction, await_detection_of_nullifier))]
    pub async fn submit_transaction(
        &mut self,
        transaction: &Transaction,
        await_detection_of_nullifier: Option<Nullifier>,
    ) -> Result<(), anyhow::Error> {
        println!("pre-checking transaction...");
        transaction
            .check_stateless(std::sync::Arc::new(transaction.clone()))
            .await
            .context("transaction pre-submission checks failed")?;

        println!("broadcasting transaction...");

        let client = reqwest::Client::new();
        let req_id: u8 = rand::thread_rng().gen();
        let tx_encoding = &transaction.encode_to_vec();

        // 499974 is the observed maximum byte size of a transaction before the "request body too large" error appears.
        if tx_encoding.len() > 499974 {
            return Err(anyhow::anyhow!(
                "Transaction is too large to be broadcasted to the network. Please run `pcli tx sweep` and then re-try the transaction."
            ));
        }

        println!("post to {}", self.tendermint_url.clone());
        let rsp: serde_json::Value = client
            .post(self.tendermint_url.clone())
            .json(&serde_json::json!(
                {
                    "method": "broadcast_tx_sync",
                    "params": [&tx_encoding],
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

        // Just in case something changes in the wrapping JSON resulting in a different byte-size triggering the
        // "request body too large" error, check any error message:
        if let Some(error) = result.get("error") {
            let error = error
                .as_object()
                .ok_or_else(|| anyhow::anyhow!("could not parse JSON response"))?;
            let code = error
                .get("code")
                .and_then(|c| c.as_i64())
                .ok_or_else(|| anyhow::anyhow!("could not parse JSON response"))?;
            let data = error
                .get("data")
                .and_then(|c| c.as_str())
                .ok_or_else(|| anyhow::anyhow!("could not parse JSON response"))?;
            if data.contains("request body too large") {
                return Err(anyhow::anyhow!(
                "Transaction is too large to be broadcasted to the network. Please run `pcli tx sweep` and then re-try the transaction."
            ));
            } else {
                return Err(anyhow::anyhow!(
                    "Error submitting transaction: code {}, data: {}",
                    code,
                    data
                ));
            }
        }

        // Some other errors are structured differently in the JSON:
        let code = result
            .get("code")
            .and_then(|c| c.as_i64())
            .ok_or_else(|| anyhow::anyhow!("could not parse JSON response"))?;

        if code != 0 {
            let log = result
                .get("log")
                .and_then(|l| l.as_str())
                .ok_or_else(|| anyhow::anyhow!("could not parse JSON response"))?;

            return Err(anyhow::anyhow!(
                "Error submitting transaction: code {}, log: {}",
                code,
                log
            ));
        }

        if await_detection_of_nullifier.is_none() {
            println!("transaction submitted successfully");
            return Ok(());
        }

        // We are waiting for a nullifier to be detected.
        //
        // putting two spaces in makes the ellipsis line up with the above
        println!("confirming transaction  ...");

        let account_id = self.fvk.hash();
        if let Some(nullifier) = await_detection_of_nullifier {
            tokio::time::timeout(
                std::time::Duration::from_secs(20),
                self.view().await_nullifier(account_id, nullifier),
            )
            .await
            .context("timeout waiting to detect nullifier of submitted transaction")?
            .context("error while waiting for detection of submitted transaction")?;
        }

        println!("transaction confirmed and detected");

        Ok(())
    }

    /// Submits a transaction to the network, returning `Ok` as soon as the
    /// transaction has been submitted, rather than waiting to learn whether the
    /// node accepted it.
    #[instrument(skip(self, transaction))]
    pub async fn submit_transaction_unconfirmed(
        &self,
        transaction: &Transaction,
    ) -> Result<(), anyhow::Error> {
        println!("broadcasting transaction...");

        let client = reqwest::Client::new();
        let req_id: u8 = rand::thread_rng().gen();
        println!("post to {}", self.tendermint_url.clone());
        let rsp: serde_json::Value = client
            .post(self.tendermint_url.clone())
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

    pub async fn specific_client(
        &self,
    ) -> Result<SpecificQueryServiceClient<Channel>, anyhow::Error> {
        SpecificQueryServiceClient::connect(self.pd_url.as_ref().to_owned())
            .await
            .map_err(Into::into)
    }

    pub async fn oblivious_client(
        &self,
    ) -> Result<ObliviousQueryServiceClient<Channel>, anyhow::Error> {
        ObliviousQueryServiceClient::connect(self.pd_url.as_ref().to_owned())
            .await
            .map_err(Into::into)
    }

    pub async fn tendermint_client(
        &self,
    ) -> Result<TendermintServiceClient<Channel>, anyhow::Error> {
        TendermintServiceClient::connect(self.tendermint_url.as_ref().to_owned())
            .await
            .map_err(Into::into)
    }
}
